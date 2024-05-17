use std::path::PathBuf;

use simple_excel_writer::{Column, Row, Workbook};

use crate::{config_store::ConfigStore, data::{self, Data, DataVal}};

/// A convenience struct, defined here simply to avoid
/// returning complex tuples from some functions.
/// 
/// In sample_row, each element represents a sample_id,
/// paired with a row of data corresponding to that sample
pub struct SampleOutput {
    pub headers: Vec<String>,
    pub sample_row: Vec<(String, Vec<DataVal>)>,
}//end struct SampleOutput


/// Processes the data provided, using the config provided,
/// to get csv stat columns for the data.  
/// Also uses config options to filter and split the data.
pub fn proc_csv_stat_cols(data: &Data, config: &ConfigStore) -> Result<SampleOutput,String> {
    if !config.csv_stat_columns_enabled {return Err(format!("CSV Stat columns are disabled in config!"));}
    if config.csv_stat_columns_columns.len() < 1 {return Err(format!("No columns set in config to calculate stats on!"));}

    let base_data = data.get_records();
    let filtered_data = match config.csv_class_filter_enabled {
        false => base_data,
        true => {
            let mut multi_filter_holding_vec = Vec::new();
            let filter_col_idx = data.get_header_index("raw-filtered-as").unwrap_or_else(|| 5);
            for filter in config.csv_class_filter_filters.iter() {
                match data::get_filtered_records(&base_data, filter_col_idx,DataVal::String(filter.clone())) {
                    Ok(mut single_filtered_rows) => multi_filter_holding_vec.append(&mut single_filtered_rows),
                    Err(msg) => return Err(format!("Couldn't filter records for some reason. Err msg below:\n{}", msg)),
                };
            }//end filtering to data for each class filter
            // edge case of zero filters
            if config.csv_class_filter_filters.len() == 0 {base_data}
            else {multi_filter_holding_vec}
        },
    };
    // split data up based on reading in column external-sample-id, prob index 2
    let split_data = {
        let sample_id_col_idx = data.get_header_index("external-sample-id").unwrap_or_else(|| 2);
        match data::get_split_records(&filtered_data,sample_id_col_idx) {
            Ok(split_data_ok) => split_data_ok,
            Err(msg) => return Err(format!("Couldn't split records based on \"external-sample-id\", which we think has 0-based col index {}. More info below:\n{}",sample_id_col_idx,msg)),
        }//end matching whether we can get split data properly
    };

    // create struct to hold the data we'll put in
    let mut output = SampleOutput {
        headers: Vec::new(),
        sample_row: Vec::new(),
    };
    // pre-fill output.headers with values
    for col_label in config.csv_stat_columns_columns.iter() {
        output.headers.push(format!("Avg {}", col_label));
        output.headers.push(format!("Std {}", col_label));
    }//end adding each header we'll use to output

    // process data for each group, then add to output
    for (sample_id_val, rows) in split_data {
        let mut output_row = Vec::new();

        for stat_col_header in config.csv_stat_columns_columns.iter() {
            if let Some(col_idx) = data.get_header_index(&stat_col_header) {
                let col_avg = match data::get_col_avg_sngl(&rows, col_idx) {
                    Ok(avg) => avg,
                    Err(msg) => return Err(format!("Encountered an error while trying to find the average value in column {} for rows with sample id {}:\n{}",stat_col_header, sample_id_val.to_string(), msg)),
                };
                let col_std = match data::get_col_stdev_sngl(&rows, col_idx) {
                    Ok(stdev) => stdev,
                    Err(msg) => {
                        match msg {
                            s if s.starts_with("Encountered a string where there should be a number") => {
                                println!("\nCouldn't calculate standard deviation for column {} and sample id {} because of a string being present in the data.",stat_col_header,sample_id_val.to_string());
                                println!("Standard deviation will be skipped for that column in that sample, instead listed as -1000.0. More information on how this happened:\n{}\n",s);
                                -1000.0
                            },
                            _ => return Err(format!("Encountered an error while trying to find the standard deviation of column {} for rows with sample id {}:\n{}",stat_col_header,sample_id_val.to_string(),msg)),
                        }//end matching behavior based on contents of error message
                    },
                };
                output_row.push(data::DataVal::Float(col_avg));
                output_row.push(data::DataVal::Float(col_std));
            }//end if we can find the stat column for that header
        }//end looping over each base col header

        output.sample_row.push((sample_id_val.to_string(), output_row));
    }//end looping over each sample split
    
    Ok(output)
}//end proc_csv_stat_cols(data, config)

/// Creates an excel workbook at the specified path, allowing it
/// to be used in later functions.  
/// The primary reason for this function to fail is the inability to
/// convert [output_path] to a string. The most likely cause for that
/// is invalid unicode characters.
pub fn get_workbook(output_path: &PathBuf) -> Result<Workbook,String> {
    match output_path.as_path().to_str() {
        Some(path) => {
            let wb = Workbook::create(path);
            return Ok(wb);
        },
        None => Err(format!("Unabled to convert path {} to string when creating workbook.", output_path.to_string_lossy()))
    }//end matching whether we can get the path correctly
}//end get_workbook()

pub fn write_output_to_sheet(workbook: &mut Workbook, sheet_data: &SampleOutput, sheet_name: &str) -> Result<(),String> {
    let mut sheet = workbook.create_sheet(sheet_name);

    // add column for external-sample-id, plus other headers
    sheet.add_column( Column { width: 18.0 } );
    for header in sheet_data.headers.iter() {
        // type conversions are needed because of underlying excel type
        let int_size = i16::try_from(header.len()).unwrap_or(10);
        let float_size = f32::try_from(int_size).unwrap_or(10.);
        sheet.add_column( Column { width: float_size } );
    }//end adding column for each header

    // convert sheet data into rows
    let header_row = {
        let mut cur_row = Row::new();
        cur_row.add_cell("external-sample-id");
        for header in sheet_data.headers.iter() {
            cur_row.add_cell(header.as_str());
        }//end adding each header to row
        cur_row
    };
    let excel_rows = {
        let mut cur_rows: Vec<Row> = Vec::new();
        for (sample_id, data_cells) in sheet_data.sample_row.iter() {
            let mut row = Row::new();
            row.add_cell(sample_id.as_str());
            for data_cell in data_cells {
                match data_cell {
                    DataVal::Int(i) => {
                        // need to do jank conversion due to limitations of trait impl
                        match i32::try_from(*i) {
                            Ok(i32_v) => {
                                match f64::try_from(i32_v) {
                                    Ok(f64_v) => row.add_cell(data::precision_f64(f64_v,0)),
                                    Err(_) => row.add_cell(data_cell.to_string()),
                                }}, Err(_) => row.add_cell(data_cell.to_string())
                        }},
                    DataVal::String(s) => row.add_cell(s.as_str()),
                    DataVal::Float(f) => row.add_cell(data::precision_f64(*f, 2)),
                }//end matching type of data_cell
            }//end adding each data_cell to row
            cur_rows.push(row);
        }//end adding each row of data to cur_rows
        cur_rows
    };

    if let Err(error) = workbook.write_sheet(&mut sheet, |sheet_writer| {
        let sw = sheet_writer;
        sw.append_row(header_row)?;
        for row in excel_rows { sw.append_row(row)?; }
        Ok(())
    }) {return Err(format!("Encountered an error while trying to write to excel sheet {}!\n{}", sheet_name, error.to_string()))};

    if let Err(error) = workbook.close() {return Err(format!("Encountered an error while trying to close excel sheet {}!\n{}", sheet_name, error.to_string()));}

    Ok(())
}//end write_output_to_sheet()
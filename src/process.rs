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

/// Does processing to find the percentage of each sample that belong to 
/// each class. 
pub fn proc_csv_class_per(data: &Data, config: &ConfigStore) -> Result<SampleOutput,String> {
    if !config.csv_class_percent_enabled {return Err(format!("CSV Class Percents are disabled in config!"));}
    
    let base_data = data.get_records();
    let split_data = {
        let sample_id_col_idx = data.get_header_index("external-sample-id").unwrap_or(2);
        match data::get_split_records(&base_data, sample_id_col_idx) {
            Ok(split_data_ok) => split_data_ok,
            Err(msg) => return Err(format!("Couldn't split records based on \"external-sample-id\", which we think has 0-based col index {}. More info below:\n{}",sample_id_col_idx,msg)),
        }//end matching whether we can get split data properly
    };

    // use cor-filtered-as, expected col 6 for class
    let class_idx = data.get_header_index("cor-filtered-as").unwrap_or(6);

    // (sample-id, vec<(class_name, count of class)>)
    let sample_class_totals: Vec<(&DataVal, Vec<(&DataVal, usize)>)> = {
        let mut s_c_t = Vec::new();
        for (sample_id, sample_data) in split_data {
            // s_c_t.push((sample_id, Vec::new()));
            let mut this_sample_count: Vec<(&DataVal,usize)> = Vec::new();
            for sample_row in sample_data {
                match sample_row.get_data(class_idx) {
                    Some(cell) => {
                        let this_val = cell.get_data();
                        if this_sample_count.iter().filter(|elem| (*elem.0).eq(this_val)).count() == 0 {this_sample_count.push((this_val, 0))}
                        for (class_name, class_count) in &mut this_sample_count {
                            if (*class_name).eq(this_val) {*class_count += 1; break;}
                        }//end looping to find classes with this name
                    },
                    None => println!("Couldn't access cell in 0-based col index {}, row {:?}!",class_idx,sample_row),
                }//end matching whether we can access index
            }//end looping over each row in data for this sample
            s_c_t.push((sample_id, this_sample_count));
        }//end looping over each sample
        s_c_t
    };

    let all_class_options = {
        let mut running_class_options = Vec::new();
        for (_, class_totals) in sample_class_totals.iter() {
            for (class_name, _) in class_totals.iter() {
                if !running_class_options.contains(class_name) {
                    running_class_options.push(class_name);
                }//end if we don't already know this one
            }//end looping over classes within this sample
        }//end looping over class counts for each sample
        running_class_options
    };

    let mut output = SampleOutput {
        headers: Vec::new(),
        sample_row: Vec::new(),
    };

    for class_option in all_class_options.iter() {
        output.headers.push(format!("%{}",class_option.to_string()));
    }//end adding each class option as a header

    for (sample_id, class_counts) in sample_class_totals {
        let all_classes_count = class_counts.iter().fold(0, |accum, elem| accum + elem.1);
        let mut this_sample_row = Vec::new();
        for class_name in all_class_options.iter() {
            let count_for_class = class_counts.iter().filter(|elem| (*elem.0).eq(class_name)).fold(0, |accum, elem| accum + elem.1);
            let class_percent = count_for_class as f64 / all_classes_count as f64 * 100.;
            this_sample_row.push(DataVal::Float(class_percent));
        }//end adding percent for each class option
        output.sample_row.push((sample_id.to_string(), this_sample_row));
    }//end looping over each sample's class counts

    return Ok(output);
}//end proc_csv_class_per(data, config)

/// Converts Data from xml into a SampleOutput.  
/// It is assumed that any necessary processing has already been done,
/// and the sample id is called "external-sample-id" or has index 1.
pub fn proc_xml_sieve_data(data: &Data, config: &ConfigStore) -> Result<SampleOutput,String> {
    if !config.xml_sieve_cols_enabled {return Err(format!("XML Sieve Data is disabled in the config!"));}

    let base_data = data.get_records();

    let mut output = SampleOutput {
        headers: Vec::new(),
        sample_row: Vec::new(),
    };

    let sample_id_col_idx = data.get_header_index("external-sample-id").unwrap_or(1);
    
    for (col_idx, header) in data.get_headers().iter().enumerate() {
        if col_idx <= sample_id_col_idx {continue;}
        output.headers.push(header.to_string());
    }//end filling output with headers from sample_id_col_idx onwards

    // just add the raw data to output, we assume it was processed already
    for row in base_data {
        match row.get_data(sample_id_col_idx) {
            Some(sample_id) => {
                let mut datavals: Vec<DataVal> = Vec::new();
                for (col_idx, datacell) in row.get_row_data().iter().enumerate() {
                    if col_idx <= sample_id_col_idx {continue;}
                    datavals.push(datacell.get_data().clone());
                }//end looping over each data cell in the row
                output.sample_row.push((sample_id.get_data().to_string(),datavals));
            },
            None => println!("\nSkipping a row during XML Output!: {:?}\nCouldn't get the sample_id for row idx {}.\nExpected 0-based col-idx of {} for header \"external-sample-id\", but row data has length of {}.\n",row,row.get_row_idx(),sample_id_col_idx,row.get_row_data().len()),
        }//en dmatching whether we can get the row data
    }//end looping over each row

    return Ok(output);
}//end proc_xml_sieve_data(data,config)

/// Creates an excel workbook at the specified path, allowing it
/// to be used in later functions.  
/// The primary reason for this function to fail is the inability to
/// convert output_path to a string. The most likely cause for that
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

/// Should be called after done working with a workbook.  
/// It is not clear what the `Option<Vec<u8>>` refers to.
pub fn close_workbook(workbook: &mut Workbook) -> Result<Option<Vec<u8>>,String> {
    match workbook.close() {
        Ok(a) => Ok(a),
        Err(error) => Err(format!("Encountered an error while trying to close excel sheet!\n{:?}",error))
    }//end matching result of closing workbook
}//end close_workbook(workbook)

/// Writes output from another function to a workbook that has already
/// been created. After you're done calling this function (however many times),  
/// make sure to call process::close_workbook().
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

    Ok(())
}//end write_output_to_sheet()
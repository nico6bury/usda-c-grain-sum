use std::path::PathBuf;

use rust_xlsxwriter::{Format, Workbook, XlsxError};

use crate::{config_store::ConfigStore, data::{self, Data, DataRow, DataVal}};

/// A convenience struct, defined here simply to avoid
/// returning complex tuples from some functions.
/// 
/// The primary intention is that data processing methods can
/// export this as one format, and then functions which write
/// to files can simply take this as input.
/// 
/// Each element of headers corresponds to the name of
/// a column header, and the number of decimal places
/// that should be displayed for values in that header.
/// Each header element also contains a boolean that
/// says whether that column contains percent values.
/// True for percents, false for regular numbers.
/// 
/// In sample_row, each element represents a sample_id,
/// paired with a row of data corresponding to that sample
pub struct SampleOutput {
    pub headers: Vec<(String, usize, bool)>,
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
            let filter_col_idx = data.get_header_index(&config.csv_class_filter_class).unwrap_or_else(|| { println!("Couldn't find class filter header \"{}\"!\nResorting to Default!", &config.csv_class_filter_class); return 5;});
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
        let sample_id_col_idx = data.get_header_index(&config.csv_sample_id_header).unwrap_or_else(|| {println!("Couldn't find sample id header \"{}\"!\nResorting to Default!",&config.csv_sample_id_header); return 2;});
        match data::get_split_records(&filtered_data,sample_id_col_idx) {
            Ok(split_data_ok) => split_data_ok,
            Err(msg) => return Err(format!("Couldn't split records based on \"{}\", which we think has 0-based col index {}. More info below:\n{}",config.csv_sample_id_header,sample_id_col_idx,msg)),
        }//end matching whether we can get split data properly
    };

    // create struct to hold the data we'll put in
    let mut output = SampleOutput {
        headers: Vec::new(),
        sample_row: Vec::new(),
    };
    // pre-fill output.headers with values
    for col_label in config.csv_stat_columns_columns.iter() {
        let decimal_places = match col_label.as_str() {
            "Weight" | "Light" | "Saturation" => 4,
            "Hue" | "Red" | "Green" | "Blue" => 1,
            _ => 2,
        };//end matching col_label to decimal places
        output.headers.push((format!("Avg {}", col_label),decimal_places,false));
        output.headers.push((format!("Std {}", col_label),decimal_places,false));
    }//end adding each header we'll use to output

    // process data for each group, then add to output
    for (sample_id_val, rows) in split_data {
        let mut output_row = Vec::new();

        for stat_col_header in config.csv_stat_columns_columns.iter() {
            if let Some(col_idx) = data.get_header_index(&stat_col_header) {
                let col_avg = match get_col_avg_sngl(&rows, col_idx) {
                    Ok(avg) => avg,
                    Err(msg) => return Err(format!("Encountered an error while trying to find the average value in column {} for rows with sample id {}:\n{}",stat_col_header, sample_id_val.to_string(), msg)),
                };
                let col_std = match get_col_stdev_sngl(&rows, col_idx) {
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
        let sample_id_col_idx = data.get_header_index(&config.csv_sample_id_header).unwrap_or_else(|| {println!("Couldn't find sample id header \"{}\"!\nResorting to Default!",&config.csv_sample_id_header); return 2;});
        match data::get_split_records(&base_data, sample_id_col_idx) {
            Ok(split_data_ok) => split_data_ok,
            Err(msg) => return Err(format!("Couldn't split records based on \"{}\", which we think has 0-based col index {}. More info below:\n{}",&config.csv_sample_id_header,sample_id_col_idx,msg)),
        }//end matching whether we can get split data properly
    };

    // use cor-filtered-as, expected col 6 for class
    let class_idx = data.get_header_index(&config.csv_class_filter_class).unwrap_or_else(|| {println!("Couldn't find class header \"{}\"!\nResorting to Default!",&config.csv_class_filter_class); return 6;});

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
        // data_format: Format::new().set_num_format("0.0%"),
    };

    for class_option in all_class_options.iter() {
        output.headers.push((format!("%{}",class_option.to_string()), 1, true));
    }//end adding each class option as a header

    for (sample_id, class_counts) in sample_class_totals {
        let all_classes_count = class_counts.iter().fold(0, |accum, elem| accum + elem.1);
        let mut this_sample_row = Vec::new();
        for class_name in all_class_options.iter() {
            let count_for_class = class_counts.iter().filter(|elem| (*elem.0).eq(class_name)).fold(0, |accum, elem| accum + elem.1);
            let class_percent = count_for_class as f64 / all_classes_count as f64;// * 100.;
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
        // data_format: Format::new().set_num_format("0.00"),
    };

    let sample_id_col_idx = data.get_header_index(&config.xml_sample_id_header).unwrap_or_else(|| {println!("Couldn't find xml sample-id header \"{}\"!\nResorting to Default!",&config.xml_sample_id_header); return 0;});
    
    for (col_idx, header) in data.get_headers().iter().enumerate() {
        if col_idx <= sample_id_col_idx {continue;}
        output.headers.push((header.to_string(),2,false));
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

/// Creates an excel workbook, which can then be used in
/// further funtions.
pub fn get_workbook() -> Workbook {
    Workbook::new()
}//end get_workbook()

/// Should be called after done working with a workbook, for performance reasons.
pub fn close_workbook(workbook: &mut Workbook, output_path: &PathBuf) -> Result<(),XlsxError> {
    workbook.save(output_path)?;
    Ok(())
}//end close_workbook(workbook)

/// Writes output from another function to a workbook that has already
/// been created. After you're done calling this function (however many times),  
/// make sure to call process::close_workbook().
pub fn write_output_to_sheet(workbook: &mut Workbook, sheet_data: &SampleOutput, sheet_name: &str) -> Result<(),XlsxError> {
    let sheet = workbook.add_worksheet();//workbook.create_sheet(sheet_name);
    sheet.set_name(sheet_name)?;

    // write the header row
    let bold = Format::new().set_bold();
    sheet.write_with_format(0,0,"external-sample-id", &bold)?;
    for (index,header) in sheet_data.headers.iter().enumerate() {
        let index = index as u16;
        sheet.write_with_format(0,index + 1,header.0.clone(),&bold)?;
    }//end adding column headers

    // create formats for each header row
    let mut formats = Vec::new();
    for (_,decimals,is_percent) in sheet_data.headers.iter() {
        let mut num_format = String::from("0.");
        for _ in 0..*decimals {num_format.push_str("0")}
        if *is_percent {num_format.push_str("%")}
        let this_format = Format::new().set_num_format(num_format);
        formats.push(this_format);
    }//end creating format for each header

    let default_format = Format::new().set_num_format("0.00");
    let mut row_num = 1;
    for (sample_id, data_cells) in sheet_data.sample_row.iter() {
        sheet.write(row_num, 0, sample_id)?;
        for (col_offset, data_cell) in data_cells.iter().enumerate() {
            // let col_idx = col_offset;
            let format = formats.get(col_offset).unwrap_or(&default_format);
            let col_offset = col_offset as u16;
            match data_cell {
                DataVal::Float(f) => sheet.write_number_with_format(row_num,1 + col_offset,*f, format)?,
                DataVal::Int(i) => sheet.write_number_with_format(row_num,1 + col_offset,*i as f64, format)?,
                DataVal::String(s) => sheet.write(row_num,1 + col_offset,s)?,
            };
        }//end adding each data cell to output
        row_num += 1;
    }//end looping over each line of data to write

    Ok(())
}//end write_output_to_sheet()

/// Gets information on sum and counts of different data types within columns.
/// This is formatted as (sum_info, count_info).
/// sum_info contains the sum of ints and sum of floats.
/// count_info contains the number of ints, floats, and strings.
/// 
/// # Examples
/// 
/// ```
/// use usda_c_grain_sum::data::DataVal;
/// use usda_c_grain_sum::data::DataCell;
/// use usda_c_grain_sum::data::DataRow;
/// use usda_c_grain_sum::data::Data;
/// use usda_c_grain_sum::process::get_sum_count;
/// 
/// // set up headers
/// let mut column_headers: Vec<String> = Vec::new();
/// let header_0 = String::from("Class");
/// let header_1 = String::from("Area");
/// let header_2 = String::from("Red");
/// column_headers.push(header_0.clone());
/// column_headers.push(header_1.clone());
/// column_headers.push(header_2.clone());
/// 
/// // set up rows of DataCells
/// let mut cell_row_0: Vec<DataCell> = Vec::new();
/// let mut cell_row_1: Vec<DataCell> = Vec::new();
/// cell_row_0.push(DataCell::new_from_val(&header_0, DataVal::String("Sound".to_string())));
/// cell_row_0.push(DataCell::new_from_val(&header_1, DataVal::Float(7.8)));
/// cell_row_0.push(DataCell::new_from_val(&header_2, DataVal::Int(55)));
/// cell_row_1.push(DataCell::new_from_val(&header_0, DataVal::String("Sound".to_string())));
/// cell_row_1.push(DataCell::new_from_val(&header_1, DataVal::Float(5.6)));
/// cell_row_1.push(DataCell::new_from_val(&header_2, DataVal::Int(60)));
/// 
/// // set up DataRows and add them to vec
/// let mut datarow_vec: Vec<DataRow> = Vec::new();
/// let datarow_0 = DataRow::new(0, cell_row_0);
/// let datarow_1 = DataRow::new(1, cell_row_1);
/// datarow_vec.push(datarow_0.clone());
/// datarow_vec.push(datarow_1.clone());
/// 
/// // create the data struct from everything
/// let data = Data::from_row_data(column_headers, datarow_vec);
/// 
/// let base_records = data.get_records();
/// // test string count
/// let class_sum_count = get_sum_count(&base_records, 0).unwrap();
/// assert_eq!(class_sum_count, ((0,0.0),(0,0.0,2))); // two strs, none else
/// // test int count and sum
/// let red_sum_count = get_sum_count(&base_records, 2).unwrap();
/// assert_eq!(red_sum_count, ((115,0.0),(2,0.0,0)))
/// ```
pub fn get_sum_count(records: &Vec<&DataRow>, col_idx: usize) -> Result<((i64,f64),(i64,f64,usize)), String> {
    let mut running_sums: (i64, f64) = (0,0.0);
    // int, float, string
    let mut running_counts: (i64, f64, usize) = (0,0.0,0);
    for row in records {
        if let Some(this_cell_at_col) = row.get_data(col_idx) {
            match this_cell_at_col.get_data() {
                DataVal::Int(i) => {running_sums.0 += i; running_counts.0 += 1;},
                DataVal::Float(f) => {running_sums.1 += f; running_counts.1 += 1.0;},
                DataVal::String(_) => {running_counts.2 += 1;},
            }//end matching type of cell data
        } else { return Err(format!("Couldn't get data at col idx {} for row data {:?}", col_idx, row.get_row_data())); }
    }//end looping over each row
    return Ok((running_sums, running_counts));
}//end get_sum_count

/// Gets the average value from a single column of the grid made up of DataRows.
/// Returns avg of integer values found, number of strings found, and avg of floats found.
/// This is ordered as int, float, string in the output.
/// 
/// # Examples
/// 
/// ```
/// use usda_c_grain_sum::data::DataVal;
/// use usda_c_grain_sum::data::DataCell;
/// use usda_c_grain_sum::data::DataRow;
/// use usda_c_grain_sum::data::Data;
/// use usda_c_grain_sum::process::get_sum_count;
/// use usda_c_grain_sum::process::get_col_avg;
/// 
/// // set up headers
/// let mut column_headers: Vec<String> = Vec::new();
/// let header_0 = String::from("Area");
/// column_headers.push(header_0.clone());
/// 
/// // set up rows of DataCells
/// let mut cell_row_0: Vec<DataCell> = Vec::new();
/// let mut cell_row_1: Vec<DataCell> = Vec::new();
/// let mut cell_row_2: Vec<DataCell> = Vec::new();
/// let mut cell_row_3: Vec<DataCell> = Vec::new();
/// 
/// cell_row_0.push(DataCell::new_from_val(&header_0, DataVal::Float(5.6)));
/// cell_row_1.push(DataCell::new_from_val(&header_0, DataVal::Float(7.8)));
/// cell_row_2.push(DataCell::new_from_val(&header_0, DataVal::Int(7)));
/// cell_row_3.push(DataCell::new_from_val(&header_0, DataVal::Int(5)));
/// 
/// // set up DataRows and add them to vec
/// let mut datarow_vec: Vec<DataRow> = Vec::new();
/// let datarow_0 = DataRow::new(0, cell_row_0);
/// let datarow_1 = DataRow::new(1, cell_row_1);
/// let datarow_2 = DataRow::new(2, cell_row_2);
/// let datarow_3 = DataRow::new(3, cell_row_3);
/// datarow_vec.push(datarow_0);
/// datarow_vec.push(datarow_1);
/// datarow_vec.push(datarow_2);
/// datarow_vec.push(datarow_3);
/// 
/// // create the data struct from everything
/// let data = Data::from_row_data(column_headers, datarow_vec);
/// 
/// let base_records = data.get_records();
/// let avg_info = get_col_avg(&base_records, 0).unwrap();
/// // test average for ints
/// assert_eq!(avg_info.0, 6.0);
/// // test average for floats
/// let float_diff = (avg_info.1 - 6.7).abs();
/// assert!(float_diff < 0.000001);
/// // test count for strs
/// assert_eq!(avg_info.2, 0);
/// ```
pub fn get_col_avg(records: &Vec<&DataRow>, col_idx: usize) -> Result<(f64, f64, usize), String> {
    match get_sum_count(records, col_idx) {
        Ok((sum_info, count_info)) => {
            let int_avg; if count_info.0 != 0 {
                int_avg = sum_info.0 as f64 / count_info.0 as f64;
            } else {int_avg = 0.0;}
            let flt_avg; if count_info.1 != 0.0 {
                flt_avg = sum_info.1 / count_info.1;
            } else {flt_avg = 0.0;}
        
            return Ok((int_avg, flt_avg, count_info.2));
        },
        Err(msg) => return Err(format!("While attempting to get sum and count, we encountered an error:\n{}", msg)),
    }//end matching whether we could get sum and count
}//end get_col_avg

/// Gets the average value from a single column of the grid made up of DataRows.  
/// Will combine all integer and float values found into a single average.  
/// If the count of floats and ints is 0 or less, thsi function will return 0.  
/// If the col_idx provided is invalid for records, this function will return an Err.
pub fn get_col_avg_sngl(records: &Vec<&DataRow>, col_idx: usize) -> Result<f64, String> {
    // make sure that col_idx is valid
    if col_idx >= records.len() { return Err(format!("The column index {} is not valid for records, which has length {}.", col_idx, records.len())); }
    match get_sum_count(records, col_idx) {
        Ok((sum_info, count_info)) => {
            // get sum and count for everything
            let sum_combined: f64 = sum_info.0 as f64 + sum_info.1;
            let count_combined: f64 = count_info.0 as f64 + count_info.1;
            if count_combined >= 0.0 {
                let avg_combined = sum_combined / count_combined;
                return Ok(avg_combined);
            } else { return Ok(0.0); }
        },
        Err(msg) => return Err(format!("While trying to get sum and count, we encountered an error:\n{}", msg)),
    }//end matching whether we could get sum and count
}//end get_col_avg_sngl

/// Gets standard deviation from a single column.
/// We don't assume that every row in that column has same type, so we return
/// the stdev of all integers found, stdev of all floats found, and the number
/// of strings found.
pub fn get_col_stdev(records: &Vec<&DataRow>, col_idx: usize) -> Result<(f64,f64,usize), String> {
    match get_sum_count(records, col_idx) {
        Ok((_, count_info)) => {
            match get_col_avg(records, col_idx) {
                Ok(avg_info) => {
                    let mut running_sq_diff_sum: (f64, f64) = (0.0, 0.0);
                    for row in records {
                        if let Some(this_cell_at_col) = row.get_data(col_idx) {
                            match &this_cell_at_col.get_data() {
                                DataVal::Int(i) => {
                                    let mean_diff = *i as f64 - avg_info.0;
                                    let sq_mean_diff = mean_diff.powf(2.0);
                                    running_sq_diff_sum.0 += sq_mean_diff;
                                },
                                DataVal::Float(f) => {
                                    let mean_diff = f - avg_info.1;
                                    let sq_mean_diff = mean_diff.powf(2.0);
                                    running_sq_diff_sum.1 += sq_mean_diff;
                                },
                                DataVal::String(_) => {},
                            }//end matching based on cell data type
                        } else {println!("Couldn't get data at col idx {} for row data {:?}", col_idx, row.get_row_data())}
                    }//end looping over each row
                
                    let mut variance_info: (f64, f64) = (0.0, 0.0);
                    if count_info.0 != 0 { variance_info.0 =  running_sq_diff_sum.0 / count_info.0 as f64; }
                    if count_info.1 != 0.0 { variance_info.1 = running_sq_diff_sum.1 / count_info.1 as f64; }
                
                    let int_stdev = variance_info.0.sqrt();
                    let flt_stdev = variance_info.1.sqrt();
                
                    return Ok((int_stdev, flt_stdev, count_info.2));
                },
                Err(msg) => return Err(format!("While trying to get averages, encountered an error:\n{}", msg)),
            }//end matching whether we can get averages
        },
        Err(msg) => return Err(format!("While trying to get sum and count, we encountered an error:\n{}", msg)),
    }//end matching whether we can get sum and count
}//end get_col_stdev

/// Gets standard deviation from a single column.  
/// Will combine all integers and floats together, returning the stdev of the whole column.  
/// An error will be returned in any of the following cases:
/// - A string is encountered as a record
/// - The column index provided is invalid for the records provided
/// - The count of all numbers among the records is 0
/// 
pub fn get_col_stdev_sngl(records: &Vec<&DataRow>, col_idx: usize) -> Result<f64, String> {
    match get_sum_count(records, col_idx) {
        Ok((_, count_info)) => {
            let full_count: f64 = count_info.0 as f64 + count_info.1;
            match get_col_avg_sngl(records, col_idx) {
                Ok(avg) => {
                    let mut running_sq_diff_sum: f64 = 0.0;
                    for row in records {
                        if let Some(this_cell_at_col) = row.get_data(col_idx) {
                            let val_at_cell = match &this_cell_at_col.get_data() {
                                DataVal::Int(i) => *i as f64,
                                DataVal::Float(f) => *f,
                                DataVal::String(s) => return Err(format!("Encountered a string where there should be a number. Row idx {}, col idx {}. Data in cell is \"{}\"", row.get_row_idx(), col_idx, s)),
                            };//end matching based on cell data type
                            let mean_diff = val_at_cell - avg;
                            let sq_mean_diff = mean_diff.powf(2.0);
                            running_sq_diff_sum += sq_mean_diff;
                        } else { return Err(format!("Couldn't get data at col idx {} and row idx {}. Row data is {:?}", col_idx, row.get_row_idx(), row.get_row_data())); }
                    }//end adding sq diff of each row

                    if full_count > 0.0 {
                        let variance = running_sq_diff_sum / full_count;
                        let stdev = variance.sqrt();
                        return Ok(stdev);
                    } else { return Err(format!("We couldn't do calculations because the count, {}, was 0 or less.", full_count)); }
                },
                Err(msg) => return Err(format!("While trying to get average, we encountered an error:\n{}", msg)),
            }//end matching whether we can get average
        },
        Err(msg) => return Err(format!("While trying to get sum and count, we encountered an error:\n{}", msg)),
    }//end matching whether we could get sum and count
}//end get_col_stdev_sngl()

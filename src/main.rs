use std::{path::PathBuf, str::FromStr};

use simple_excel_writer::*;
use usda_c_grain_sum::{config_store::{self, ConfigStore}, data};
use {usda_c_grain_sum::data::{Data, DataVal}, gui::GUI};

mod gui;

fn main() {
    // set up main application and window
    // let mut gui = GUI::initialize();
    // gui.ux_main_window.show();

    // while gui.app.wait() {

    // }
    
    // setup gui
    let mut gui = GUI::initialize();
    
    // get config information
    let config_name = "config";
    let mut config_path: Option<PathBuf> = None;
    let mut config_store: Option<ConfigStore> = None;
    match config_store::try_read_config_path(config_name, false) {
        Ok(config_path_tmp) => {
            if !config_path_tmp.exists() {
                let should_create_personal_file = GUI::show_yes_no_message("It seems that a configuration hasn't been set up yet. Perhaps this is your first time starting this application?\nIf a personalized config is not created, we'll just save a default configuration.\n\nYou can change your config at any time, but would you like to choose a preset configuration now?");
                let mut new_conf_stor = ConfigStore::default();
                if should_create_personal_file {
                    match GUI::show_three_choice("Please choose the config preset you want, or None/Default if you want to just stick with the default config.", "Scott", "None/Default", "Rhett") {
                        Some(0) => new_conf_stor = config_store::get_scott_config(),
                        Some(1) => new_conf_stor = ConfigStore::default(),
                        Some(2) => new_conf_stor = config_store::get_rhett_config(),
                        _ => GUI::show_message("Guided configuration setting cancelled (somehow?). We'll just use the default then."),
                    }//end matching dialog result for config preset
                }//end if we should create a personalized config file
                match config_store::try_write_config(&config_path_tmp, &new_conf_stor) {
                    Ok(_) => {
                        config_store = Some(new_conf_stor);
                        GUI::show_message("Your configuration was successfully written and set.\nIf you continue seeing messages about the config file when opening the program, please contact the developer.");
                    },
                    Err(msg) => GUI::show_alert(&format!("It seems we were unable to write the new configuration to a file,\nthough you should still be able to the program for now with the config you selected.\nError message was \"{}\".\nIf this operation keeps failing, please contact the developer.", msg))
                }//end matching whether or not we successfully wrote a new config file
            }//end if config_path_tmp doesn't point to a real file
            else {
                match config_store::try_read_config(&config_path_tmp) {
                    Ok(config_store_tmp) => {
                        gui.set_config_store(&config_store_tmp);
                        config_store = Some(config_store_tmp);
                    },
                    Err(msg) => {
                        GUI::show_alert(&format!("Could not read config file at path \"{}\".\nReceived error msg {}", config_path_tmp.to_string_lossy(), msg));
                        let should_create_new = GUI::show_yes_no_message("Problems with the config file might occur when changing versions.\nWhen the config file is deleted, the program will automatically create a new one by default.\nEven if a config file is not loaded, you can always set the config yourself using the section in the bottom right.\n\nWould you like to be delete the old config file and create a personalized one now?");
                        if should_create_new {
                            let mut new_conf_stor: Option<ConfigStore> = None;
                            match GUI::show_three_choice("Do you want a personalized config file?\nIf so, choose which preset you want:", "Scott", "None/Default", "Rhett") {
                                Some(0) => new_conf_stor = Some(config_store::get_scott_config()),
                                Some(1) => new_conf_stor = Some(ConfigStore::default()),
                                Some(2) => new_conf_stor = Some(config_store::get_rhett_config()),
                                _ => GUI::show_message("Guided configuration setting cancelled."),
                            }//end matching dialog result
                            if let Some(new_conf_stor) = new_conf_stor {
                                match config_store::try_write_config(&config_path_tmp, &new_conf_stor) {
                                    Ok(_) => {
                                        GUI::show_message("Congrats, we successfully wrote your changes to the config file.\nWhatever the problem was, it should be fixed.\nIf you continue seeing messages about this everytime you open the application, please contact the developer.");
                                        config_store = Some(new_conf_stor);
                                    },
                                    Err(msg) => GUI::show_alert(&format!("We couldn't write your config to the file, though you should still be able\nto use the program for now with the config you selected.\nError message was \"{}\".\nIf this operation keeps failing, please contact the developer.", msg))
                                }//end matching whether or not we can write to file
                            }//end if we have a new config store to write
                        }//end if we get the ok to make a new config file
                    }//end case of not being able to parse file at config_path_tmp
                }//end matching whether we can read file at config_path_tmp
                config_path = Some(config_path_tmp);
            }//end else the config file already exists
        },
        Err(msg) => GUI::show_alert(&format!("Could not determine the path to the config.\nReceived error msg {}", msg))
    }//end matching whether or not we can get config path

    // set up data containers for use during app loop
    let recv = gui.get_receiver();
    let mut input_data = None;
    let mut csv_input_file = None;
    let mut output_file = None;

    while gui.wait() {
        if let Some(msg) = recv.recv() {
            let msg_parts: Vec<&str> = msg.split("::").collect();
            // general location, sorta
            let msg_loc = *msg_parts.get(0).unwrap_or(&"None");
            // more specific of what message is being sent
            let msg_fun = *msg_parts.get(1).unwrap_or(&"None");
            // any value sent over 
            let msg_stf = *msg_parts.get(2).unwrap_or(&"None");
            match msg_loc {
                "IO" => {
                    match msg_fun {
                        "CSVInputFile" => {
                            if msg_stf != "None" {
                                // try to get csv file
                                gui.start_wait();
                                let path_buf = PathBuf::from(msg_stf);
                                match csv::Reader::from_path(path_buf.clone()) {
                                    Ok(reader) => {
                                        println!("We got the csv reader");
                                        let data = Data::from_csv_reader(reader).unwrap();
                                        println!("We finished reading {} records from the csv", data.get_records().len());
                                        input_data = Some(data);
                                        csv_input_file = Some(path_buf);
                                        // format_csv_sum(&data);
                                    },
                                    Err(_) => GUI::show_message("Couldn't get csv reader."),
                                }//end matching result of getting csv reader
                                gui.end_wait();
                            }
                        },
                        "XMLInputFile" => {
                            GUI::show_message("XML Support not yet added...");
                        },
                        "OutputFile" => {
                            if msg_stf != "None" {
                                // we got an output file
                                match PathBuf::from_str(msg_stf) {
                                    Ok(path_buf) => {
                                        output_file = Some(path_buf);
                                        println!("Got output file path: \"{}\"", msg_stf);
                                    },
                                    Err(_) => {
                                        println!("Somehow we couldn't get a path_buf even though the conversion is infallible. This should never happen.");
                                    },
                                }//end matching whether we can get pathbuf
                            } else { println!("We got a message about OutputFile, but nothing was sent? This should not happen."); }
                        },
                        "None" => println!("No message function for msg {} ???", msg),
                        _ => println!("Unrecognized msg_fun {} in msg {}", msg_fun, msg),
                    }//end matching message function
                },
                "Proc" => {
                    match msg_fun {
                        "Sum" => {
                            match &input_data {
                                Some(input) => {
                                    // make sure user entered output file affects processing
                                    let output_txt = gui.get_output_text();
                                    if output_txt != "" && output_file.is_none() {
                                        let input_stem = match csv_input_file {
                                            Some(ref pathbuf) => match pathbuf.parent() {
                                                Some(stem) => match stem.to_str() {
                                                    Some(str) => str,
                                                    None => "", }, None => "", }, None => "",
                                        };
                                        if input_stem != "" {
                                            let mut output_pathbuf = PathBuf::new();
                                            output_pathbuf.push(input_stem);
                                            output_pathbuf.push(output_txt.clone());
                                            output_pathbuf.set_extension("xlsx");
                                            if !output_pathbuf.exists() || GUI::show_yes_no_message("The output file you specified already exists.\nAre you sure you want to replace it?") {
                                                output_file = Some(output_pathbuf);
                                            }//end if file doesn't exist OR user is fine with overwriting it
                                        }//end if we got the input file stem
                                    }//end if we need to update output file name from user entered text
                                    match &mut output_file {
                                        Some(output) => {
                                            gui.start_wait();
                                            output.set_file_name(gui.get_output_text());
                                            output.set_extension("xlsx");
                                            println!("Started processing and outputing file.");
                                            // output_csv_sum(input, output);
                                            let config = gui.get_config_store();
                                            if let Err(msg) = output_excel_sum(input, output, config) {
                                                GUI::show_message(&format!("Encountered errors while processing:\n{}", msg));
                                            } else {
                                                println!("Finished outputing processed file.");
                                                gui.clear_output_text();
                                                if GUI::show_yes_no_message("Processing complete. Would you like to open the folder where the output file is located?") {
                                                    opener::reveal(output).unwrap();
                                                }//end if user wants to open folder
                                                input_data = None;
                                                output_file = None;
                                            }//end else everything was find
                                            gui.end_wait();
                                        },
                                        None => GUI::show_message("No Output File Selected")
                                    }//end matching existence of output file
                                },
                                None => GUI::show_message("No Input File Loaded")
                            }//end matching existence of input file
                        },
                        "None" => {
                            println!("None message recieved for Proc?");
                        },
                        _ => println!("Unrecognized msg_fun {} in msg {}", msg_fun, msg),
                    }//end matching message function
                },
                "App" => {
                    match msg_fun {
                        "Closing" => {
                            match config_path {
                                Some(ref config_path_tmp) => {
                                    if config_store.is_some() {config_store = Some(gui.get_config_store())}
                                    match config_store {
                                        Some(ref config_store_tmp) => {
                                            match config_store::try_write_config(config_path_tmp, config_store_tmp) {
                                                Ok(_) => println!("Config file updated!"),
                                                Err(msg) => println!("Couldn't write config to file!\nReceived message \"{}\"!", msg),
                                            }//end matching whether or not we can write the config to file
                                        },
                                        None => println!("Config Store not Initialized!"),
                                    }//end matching whether we have config store
                                },
                                None => println!("Config Path not Found!"),
                            };

                            GUI::quit();
                        },
                        _ => println!("Unrecognized msg_fun {} in msg {}", msg_fun, msg),
                    }//end matching message function
                }
                _ => println!("Unrecognized msg_loc {} in msg {}", msg_loc, msg),
            }//end matching message location
        }//end if we recieved a message
    }//end main application loop

    println!("Program Exiting!");
}

fn output_excel_sum(data: &Data, output_path: &PathBuf, config: ConfigStore) -> Result<(), String> {
    let base_data = data.get_records();
    let filtered_data = match config.csv_class_filter_enabled {
        true => {
            // get filtered rows for each filter
            let mut multi_filter_holding_vec = Vec::new();
            let filter_col_idx = data.get_header_index("raw-filtered-as").unwrap_or_else(|| 5);
            for filter in config.csv_class_filter_filters.iter() {
                match data::get_filtered_records(&base_data, filter_col_idx, DataVal::String(filter.clone())) {
                    Ok(mut single_filtered_rows) => multi_filter_holding_vec.append(&mut single_filtered_rows),
                    Err(msg) => return Err(format!("Couldn't filter records for some reason. Err msg below:\n{}", msg)),
                };
            }//end looping over each filter we're using
            // handle edge case of zero filters
            if config.csv_class_filter_filters.len() == 0 { base_data }
            else { multi_filter_holding_vec }
        },
        false => base_data,
    };
    // split data up based on reading in column external-sample-id, probably index 2
    let split_data = {
        let sample_id_col_idx = data.get_header_index("external-sample-id").unwrap_or_else(|| 2);
        match data::get_split_records(&filtered_data, sample_id_col_idx) {
            Ok(split_data_ok) => split_data_ok,
            Err(msg) => return Err(format!("Couldn't split records based on \"external-sample-id\", which we think has 0-based col index {}. More information below:\n{}", sample_id_col_idx, msg)),
        }//end matching whether we can get split data properly
    };
    println!("We split the data into {} groups.", split_data.len());
    // get all the excel writer stuff ready
    let mut wb = Workbook::create(output_path.as_path().to_str().unwrap());
    let mut stat_sheet = wb.create_sheet("Stats");

    // get whole string of all headers we'll output
    let headers = {
        let mut tmp_header_vec = Vec::new();
        tmp_header_vec.push("external-sample-id".to_string());
        if config.csv_stat_columns_enabled {
            for col_label in config.csv_stat_columns_columns.iter() {
                // make sure we can find that header
                match data.get_header_index(&col_label) {
                    Some(_) => {
                        tmp_header_vec.push(format!("Avg {}", col_label));
                        tmp_header_vec.push(format!("Std {}", col_label));
                    }, None => println!("Couldn't find column header \"{}\". Skipping that column!", col_label),
                }//end matching whether we can find the specified column header
            }//end adding label for each col
        }//end if we're outputting csv stat columns
        tmp_header_vec
    };
    for header in headers.iter() {
        stat_sheet.add_column(Column {width: header.len() as f32});
    }//end adding column for each header

    let excel_rows = {
        let mut tmp_vec = Vec::new();
        for (sample_id_val, rows) in split_data {
            let row = {
                let mut tmp_row = Row::new();
    
                let sample_id = match sample_id_val {
                    DataVal::String(s) => s.to_string(),
                    DataVal::Int(i) => format!("{}",i),
                    DataVal::Float(f) => format!("{}",f),
                }; tmp_row.add_cell(sample_id.clone()); //tmp_vals.push(sample_id.clone());

                if config.csv_stat_columns_enabled {
                    for stat_col_header in config.csv_stat_columns_columns.iter() {
                        if let Some(col_idx) = data.get_header_index(stat_col_header) {
                            let col_avg = match data::get_col_avg_sngl(&rows, col_idx) {
                                Ok(avg) => avg,
                                Err(msg) => return Err(format!("Encountered an error while trying to find the average value in for column {} for rows with sample id {}:\n{}", stat_col_header, sample_id, msg)),
                            };
                            let col_stdev = match data::get_col_stdev_sngl(&rows, col_idx) {
                                Ok(stdev) => stdev,
                                Err(msg) => {
                                    if msg.starts_with("Encountered a string where there should be a number") {
                                        println!("\nCouldn't calculate standard deviation for column {} and sample id {} because of a string being present in the data.", stat_col_header, sample_id);
                                        println!("Standard deviation will be skipped for that column in that sample, instead listed as -1000.0. More information on how this happened:\n{}\n", msg);
                                        -1000.0
                                    } else { return Err(format!("Encountered an error while trying to find the standard deviation of column {} for rows with sample id {}:\n{}", stat_col_header, sample_id, msg)) }
                                },
                            };
                            // tmp_vals.push(format!("{:.2}", col_avg));
                            tmp_row.add_cell(data::precision_f64(col_avg, 2));
                            // tmp_vals.push(format!("{:.2}", col_stdev));
                            tmp_row.add_cell(data::precision_f64(col_stdev, 2));
                        }//end if we can find the col_idx for that header
                    }//end looping over each col in the stat columns
                }//end if we're printing csv stat columns
    
                tmp_row
            };
            tmp_vec.push(row);
        }//end looping over each sample split
        tmp_vec.into_iter()
    };
    
    // write all the rows out to the stat sheet
    wb.write_sheet(&mut stat_sheet, |sheet_writer| {
        let sw = sheet_writer;
        sw.append_row(Row::from_iter(headers.into_iter()))?;
        for row in excel_rows { sw.append_row(row)?; }
        Ok(())
    }).expect("write excel error!");

    wb.close().expect("close excel error!");

    return Ok(());
}//end output_excel_sum

#[allow(dead_code)]
fn output_csv_sum(data: &Data, output_path: &PathBuf) {
    let base_data = data.get_records();
    // filter so that we only have Sound data
    let sound_data = data::get_filtered_records(&base_data, 5, DataVal::String(String::from("Sound"))).unwrap();
    // split data up based on reading in column 2, external-sample-id
    let split_data = data::get_split_records(&sound_data, 2).unwrap();
    println!("We split the data into {} groups.", split_data.len());
    // get our csv writer
    let mut writer = csv::Writer::from_path(output_path).unwrap();
    // write headers
    writer.write_field("ExtSampleID").unwrap();
    for (col_idx, header) in data.get_headers().iter().enumerate() {
        if col_idx >= 11 && col_idx <= 24 {
            let h1 = format!("{}Avg", header);
            let h2 = format!("{}Stdev", header);
            writer.write_field(h1).unwrap();
            writer.write_field(h2).unwrap();
        }//end if index is within desired range
    }//end writing the rest of the data headers
    writer.write_record(None::<&[u8]>).unwrap();
    // run through each grouping of external_sample_id and comput avg and stdev
    for (sample_id, records) in split_data {
        match sample_id {
            DataVal::Int(i) => writer.write_field(format!("{}", i)).unwrap(),
            DataVal::String(s) => writer.write_field(format!("{}", s)).unwrap(),
            DataVal::Float(f) => writer.write_field(format!("{}", f)).unwrap(),
        }//end matching type and printing sample id
        // loop through column indices 11-24
        for col_idx in 11..=24 {
            let avgs = data::get_col_avg(&records, col_idx).unwrap();
            let stdevs = data::get_col_stdev(&records, col_idx).unwrap();
            writer.write_field(format!("{:.2}", avgs.1)).unwrap();
            writer.write_field(format!("{:.2}", stdevs.1)).unwrap();
        }//end looping through column indices 11-24
        // effectively adds a newline
        writer.write_record(None::<&[u8]>).unwrap();
    }//end looping over split groups
    writer.flush().unwrap();
}//end output_csv_sum

/// Formats a csv file in the format desired
#[allow(dead_code)]
fn format_csv_sum(data: &Data) {
    /* 
    Keep columns:
    external-sample-id, 2  raw-filtered-as, 5
    area, 11 length, 12 width, 13 thickness, 14 ratio, 15 mean width, 16 volume, 17
    weight, 18 light, 19 hue, 20 saturation, 21 red, 22 green, 23 blue, 24
     */
    let base_data = data.get_records();
    // filter so that we only have Sound data
    let sound_data = data::get_filtered_records(&base_data, 5, DataVal::String("Sound".to_string())).unwrap();
    // split data up based on reading in column 2, external-sample-id
    let split_data = data::get_split_records(&sound_data, 2).unwrap();
    println!("We split the data into {} groups.", split_data.len());
    // print headers
    print!("ExtSampleID");
    for (col_idx, header) in data.get_headers().iter().enumerate() {
        if col_idx >= 11 && col_idx <= 24 {
            print!("\t{}Avg\t{}Stdev", header, header);
        }//end if index is within desired range
    }//end printing out all the data headers
    print!("\n");
    // run through each grouping of external-sample-id and compute avg and stdev for cols 11-24
    for (sample_id, records) in split_data {
        match sample_id {
            DataVal::Int(i) => print!("{}\t", i),
            DataVal::String(s) => print!("{}\t",s),
            DataVal::Float(f) => print!("{}\t", f),
        }//end printing out sample id
        // loop through column indices 11-24
        for col_idx in 11..=24 {
            let avgs = data::get_col_avg(&records, col_idx).unwrap();
            let stdevs = data::get_col_stdev(&records, col_idx).unwrap();
            print!("{:.2}\t{:.2}\t", avgs.1, stdevs.1);
        }//end looping over column indices 11-24
        print!("\n");
    }//end looping over split groups and doing avg
}//end format_csv_sum
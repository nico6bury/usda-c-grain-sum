use std::{path::PathBuf, str::FromStr};

use usda_c_grain_sum::config_store::{self, ConfigStore};
use usda_c_grain_sum::data::Data;
use usda_c_grain_sum::process::{self, SampleOutput};
use gui::GUI;

use crate::gui::InterfaceMessage;

mod gui;

fn main() {
    // setup gui
    let mut gui = GUI::initialize();
    
    // get config information
    let config_name = "config";
    let mut config_path: Option<PathBuf> = None;
    let mut config_store: Option<ConfigStore> = None;

    // make sure we get config information, update gui, walk user through fix if necessary
    ensure_config_valid(&mut gui, &mut config_store, &mut config_path, config_name);

    // set up data containers for use during app loop
    let recv = gui.get_receiver();
    let mut input_data = None;
    let mut input_xml_data = None;
    let mut csv_input_file = None;
    let mut xml_input_file = None;
    let mut output_file = None;

    while gui.wait() {
        match recv.recv() {
            Some(InterfaceMessage::CSVInputFile(file_path)) if file_path != "None" => {
                // try to get csv file
                gui.start_wait();
                let path_buf = PathBuf::from(file_path);
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
            },
            Some(InterfaceMessage::XMLInputFile(file_path)) if file_path != "None" => {
                // try to get the xml file
                gui.start_wait();
                let path_buf = PathBuf::from(file_path);
                match quick_xml::Reader::from_file(path_buf.clone()) {
                    Ok(reader) => {
                        println!("We got the xml reader");
                        let xml_data = Data::from_xml_reader(reader).unwrap();
                        println!("We finished reading {} records from the xml file.", xml_data.get_records().len());
                        input_xml_data = Some(xml_data);
                        xml_input_file = Some(path_buf);
                    },
                    Err(error) => GUI::show_alert(&format!("Error occured when trying to open xml file:\n{:?}",error)),
                }//end matching whether we can open the xml file
                gui.end_wait();
            },
            Some(InterfaceMessage::OutputFile(file_path)) if file_path != "None" => {
                // we got an output file
                match PathBuf::from_str(&file_path) {
                    Ok(path_buf) => {output_file = Some(path_buf); println!("Got output file path: \"{}\"", file_path);},
                    Err(_) => println!("Somehow we couldn't get a path_buf even though the conversion is infallible. This should never happen."),
                }//end matching whether we can get pathbuf
            },
            Some(InterfaceMessage::ProcessSum) => {
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

                                // actually call the processing functions
                                match process::get_workbook(&output) {
                                    Ok(mut wb) => {
                                        // make sure we aren't asking user to see workbook if nothing finished successfully
                                        let mut successfully_processed_at_least_once = false;
                                        // (name of sheet, data to go in that sheet)
                                        let mut output_sheets: Vec<(String, SampleOutput)> = Vec::new();
                                        
                                        // get all data we might want, based on config
                                        if config.csv_stat_columns_enabled {
                                            match process::proc_csv_stat_cols(input, &config) {
                                                Ok(sample_output) => output_sheets.push(("CSV_Stats".to_string(), sample_output)),
                                                Err(msg) => GUI::show_alert(&format!("An Error Occurred while trying to process CSV STAT Columns!\n{}",msg)),
                                            }//end matching whether or not csv stat columns were processed successfully
                                        }//end if we should output csv stat columns
                                        if config.csv_class_percent_enabled {
                                            match process::proc_csv_class_per(input, &config) {
                                                Ok(sample_output) => output_sheets.push(("Class_Percents".to_string(), sample_output)),
                                                Err(msg) => GUI::show_alert(&format!("An Error Occured while trying to process CSV Class Percent Columns!\n{}",msg)),
                                            }//end matching whether or not csv class percents were processed successfully
                                        }//end if we should output class percents
                                        if config.xml_sieve_cols_enabled {
                                            let input_xml = input_xml_data.unwrap();
                                            match process::proc_xml_sieve_data(&input_xml, &config) {
                                                Ok(sample_output) => output_sheets.push(("XML_Sieve_Data".to_string(),sample_output)),
                                                Err(msg) => GUI::show_alert(&format!("An Error occured while trying to process XML Sieve Data!\n{}", msg)),
                                            }//end matching whether or not xml sieve stuff was processed correctly
                                            input_xml_data = Some(input_xml);
                                        }//end if we should output xml sieve cols

                                        for (sheet_name, sheet_data) in output_sheets {
                                            match process::write_output_to_sheet(&mut wb, &sheet_data, &sheet_name) {
                                                Ok(_) => successfully_processed_at_least_once = true,
                                                Err(msg) => GUI::show_alert(&format!("Ecountered an error while attempting to write data to worksheet {}.\n{}", sheet_name, msg)),
                                            }//end matching whether writing to sheet was a success
                                        }//end writing data from each output sheet

                                        if let Err(error) = process::close_workbook(&mut wb) {GUI::show_alert(&format!("Encountered an error while attempting to write data to worksheet.\n{}",error));}

                                        if successfully_processed_at_least_once {
                                            println!("Finished outputing processed file.");
                                            gui.clear_output_text();
                                            if GUI::show_yes_no_message("Processing complete. Would you like to open the folder where the output file is located?") {
                                                opener::reveal(output).unwrap();
                                            }//end if user wants to open folder
                                            input_data = None;
                                            output_file = None;
                                            input_xml_data = None;
                                            xml_input_file = None;
                                        } else {
                                            GUI::show_alert("It seems that a processing routine was run without any successful outputs.\nThis shouldn't happen...");
                                        }//end else we never managed to process anything
                                    },
                                    Err(msg) => GUI::show_message(&format!("Encountered errors while creating output workbook:\n{}",msg)),
                                }//end matching whether or not we can successfully get the workbook object
                                gui.end_wait();
                            },
                            None => GUI::show_message("No Output File Selected")
                        }//end matching existence of output file
                    },
                    None => GUI::show_message("No Input File Loaded")
                }//end matching existence of input file
            },
            Some(InterfaceMessage::AppClosing) => {
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
            Some(InterfaceMessage::ConfigReset) => {
                let new_conf = match GUI::show_three_choice("Please choose the configuration preset you'd like to switch to:", "Scott", "None/Default", "Rhett") {
                    Some(0) => config_store::get_scott_config(),
                    Some(2) => config_store::get_rhett_config(),
                    _ => config_store::ConfigStore::default(),
                };
                gui.set_config_store(&new_conf);
                config_store = Some(new_conf);
            },
            Some(unrecognized_message) => GUI::show_alert(&format!("Recieved unrecognized message {:?}",unrecognized_message)),
            None => {}, 
        }//end if we recieved a message
    }//end main application loop

    println!("Program Exiting!");
}

/// Gets the config information from the config file.  
/// If we encounter issues with that, walk the user through a fix via the gui.
fn ensure_config_valid(gui: &mut GUI, config_store: &mut Option<ConfigStore>, config_path: &mut Option<PathBuf>, config_name: &str) {
    *config_path = None;
    *config_store = None;
    
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
                        gui.set_config_store(&new_conf_stor);
                        *config_store = Some(new_conf_stor);
                        GUI::show_message("Your configuration was successfully written and set.\nIf you continue seeing messages about the config file when opening the program, please contact the developer.");
                    },
                    Err(msg) => GUI::show_alert(&format!("It seems we were unable to write the new configuration to a file,\nthough you should still be able to the program for now with the config you selected.\nError message was \"{}\".\nIf this operation keeps failing, please contact the developer.", msg))
                }//end matching whether or not we successfully wrote a new config file
            }//end if config_path_tmp doesn't point to a real file
            else {
                match config_store::try_read_config(&config_path_tmp) {
                    Ok(config_store_tmp) => {
                        gui.set_config_store(&config_store_tmp);
                        *config_store = Some(config_store_tmp);
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
                                        *config_store = Some(new_conf_stor);
                                    },
                                    Err(msg) => GUI::show_alert(&format!("We couldn't write your config to the file, though you should still be able\nto use the program for now with the config you selected.\nError message was \"{}\".\nIf this operation keeps failing, please contact the developer.", msg))
                                }//end matching whether or not we can write to file
                            }//end if we have a new config store to write
                        }//end if we get the ok to make a new config file
                    }//end case of not being able to parse file at config_path_tmp
                }//end matching whether we can read file at config_path_tmp
                *config_path = Some(config_path_tmp);
            }//end else the config file already exists
        },
        Err(msg) => GUI::show_alert(&format!("Could not determine the path to the config.\nReceived error msg {}", msg))
    }//end matching whether or not we can get config path
}
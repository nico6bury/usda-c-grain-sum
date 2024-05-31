use core::str;
use std::path::PathBuf;

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
    let mut input_csv_data = None;
    let mut input_xml_data = None;
    let mut csv_input_file = None;
    let mut xml_input_file = None;
    let mut output_file = None;

    while gui.wait() {
        match recv.recv() {
            Some(InterfaceMessage::CSVInputFile(file_path)) => {
                // try to get csv file
                gui.start_wait();
                match csv::Reader::from_path(file_path.clone()) {
                    Ok(reader) => {
                        println!("We got the csv reader");
                        let data = Data::from_csv_reader(reader).unwrap();
                        println!("We finished reading {} records from the csv", data.get_records().len());
                        input_csv_data = Some(data);
                        csv_input_file = Some(file_path);
                        // format_csv_sum(&data);
                    },
                    Err(_) => gui.integrated_dialog_message("Couldn't get csv reader."),
                }//end matching result of getting csv reader
                gui.end_wait();
            },
            Some(InterfaceMessage::XMLInputFile(file_path)) => {
                // try to get the xml file
                gui.start_wait();
                match quick_xml::Reader::from_file(file_path.clone()) {
                    Ok(reader) => {
                        let mut config = gui.get_config_store();
                        println!("We got the xml reader");
                        let mut tags_to_include = vec![config.xml_sample_id_header]; tags_to_include.append(&mut config.xml_tags_to_include);
                        match Data::from_xml_reader(reader, Some(tags_to_include), Some(config.xml_sample_closing_tag.as_bytes())) {
                            Ok(xml_data) => {
                                println!("We finished reading {} records from the xml file.", xml_data.get_records().len());
                                input_xml_data = Some(xml_data);
                                xml_input_file = Some(file_path);
                            }, Err(msg) => gui.integrated_dialog_alert(&format!("Encountered an error while trying to parse xml data.\n{}",msg)),
                        }//end matching whether we can parse xml data
                    },
                    Err(error) => gui.integrated_dialog_alert(&format!("Error occured when trying to open xml file:\n{:?}",error)),
                }//end matching whether we can open the xml file
                gui.end_wait();
            },
            Some(InterfaceMessage::OutputFile(file_path)) => {
                // we got an output file
                println!("Got output file path: \"{}\"", file_path.to_string_lossy());
                output_file = Some(file_path);
            },
            Some(InterfaceMessage::ProcessSum) => {
                let config_store = Some(gui.get_config_store());
                if ensure_data_valid_for_output(&mut gui, &config_store, &input_csv_data, &input_xml_data, &mut output_file, &csv_input_file, &xml_input_file) {
                    println!("Started processing and outputing file.");
                    
                    let output = output_file.clone().unwrap();
                    let config = config_store.clone().unwrap();
                    gui.start_wait();
                    // actually call the processing functions
                    let mut wb = process::get_workbook();
                    // make sure we aren't asking user to see workbook if nothing finished successfully
                    let mut successfully_processed_at_least_once = false;
                    // (name of sheet, data to go in that sheet)
                    let mut output_sheets: Vec<(String, SampleOutput)> = Vec::new();
                    
                    // get all data we might want, based on config
                    if config.csv_stat_columns_enabled || config.csv_class_percent_enabled {
                        let input_csv = input_csv_data.unwrap();
                        if config.csv_stat_columns_enabled {
                            match process::proc_csv_stat_cols(&input_csv, &config) {
                                Ok(sample_output) => output_sheets.push(("CSV_Stats".to_string(), sample_output)),
                                Err(msg) => gui.integrated_dialog_alert(&format!("An Error Occurred while trying to process CSV STAT Columns!\n{}",msg)),
                            }//end matching whether or not csv stat columns were processed successfully
                        }//end if we should output csv stat columns
                        if config.csv_class_percent_enabled {
                            match process::proc_csv_class_per(&input_csv, &config) {
                                Ok(sample_output) => output_sheets.push(("Class_Percents".to_string(), sample_output)),
                                Err(msg) => gui.integrated_dialog_alert(&format!("An Error Occured while trying to process CSV Class Percent Columns!\n{}",msg)),
                            }//end matching whether or not csv class percents were processed successfully
                        }//end if we should output class percents
                        input_csv_data = Some(input_csv);
                    }//end if we're doing csv stuff
                    if config.xml_sieve_cols_enabled {
                        let input_xml = input_xml_data.unwrap();
                        match process::proc_xml_sieve_data(&input_xml, &config) {
                            Ok(sample_output) => output_sheets.push(("XML_Sieve_Data".to_string(),sample_output)),
                            Err(msg) => gui.integrated_dialog_alert(&format!("An Error occured while trying to process XML Sieve Data!\n{}", msg)),
                        }//end matching whether or not xml sieve stuff was processed correctly
                        input_xml_data = Some(input_xml);
                    }//end if we should output xml sieve cols

                    for (sheet_name, sheet_data) in output_sheets {
                        match process::write_output_to_sheet(&mut wb, &sheet_data, &sheet_name) {
                            Ok(_) => successfully_processed_at_least_once = true,
                            Err(msg) => gui.integrated_dialog_alert(&format!("Ecountered an error while attempting to write data to worksheet {}.\n{}", sheet_name, msg)),
                        }//end matching whether writing to sheet was a success
                    }//end writing data from each output sheet

                    if let Err(error) = process::close_workbook(&mut wb, &output) {gui.integrated_dialog_alert(&format!("Encountered an error while attempting to write data to worksheet.\n{}",error));}

                    if successfully_processed_at_least_once {
                        println!("Finished outputing processed file.");
                        gui.clear_output_text();
                        if gui.integrated_dialog_yes_no("Processing complete. Would you like to open the folder where the output file is located?") {
                            opener::reveal(output).unwrap();
                        }//end if user wants to open folder
                        input_csv_data = None;
                        output_file = None;
                        input_xml_data = None;
                    } else {
                        gui.integrated_dialog_alert("It seems that a processing routine was run without any successful outputs.\nThis shouldn't happen...");
                    }//end else we never managed to process anything
                    gui.end_wait();
                }//end if all our data is valid for the output we want to make
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
                let new_conf = match gui.integrated_dialog_message_choice("Please choose the configuration preset you'd like to switch to:", vec!["Rhett", "Scott", "Other"]) {
                    Some(0) => config_store::get_rhett_config(),
                    Some(1) => config_store::get_scott_config(),
                    _ => config_store::ConfigStore::default(),
                };
                gui.set_config_store(&new_conf);
                config_store = Some(new_conf);
            },
            Some(unrecognized_message) => gui.integrated_dialog_alert(&format!("Recieved unrecognized message {:?}",unrecognized_message)),
            None => {}, 
        }//end if we recieved a message
    }//end main application loop

    println!("Program Exiting!");
}

/// Tries to confirm that file information and data containers  
/// are appropriate for what the user wants. If things are fine,
/// returns true. Otherwise, returns false.
fn ensure_data_valid_for_output(gui: &mut GUI, config_store: &Option<ConfigStore>, input_csv_data: &Option<Data>, input_xml_data: &Option<Data>, output_file: &mut Option<PathBuf>, csv_input_file: &Option<PathBuf>, xml_input_file: &Option<PathBuf>) -> bool {
    match config_store {
        Some(config) => {
            if input_csv_data.is_none() && (config.csv_stat_columns_enabled || config.csv_class_percent_enabled) {gui.integrated_dialog_alert("You have enabled one of the CSV output columns, but you haven't loaded a CSV file!"); return false;}
            if input_xml_data.is_none() && (config.xml_sieve_cols_enabled) {gui.integrated_dialog_alert("You have enabled output based on XML input, but you haven't loaded an XML file!"); return false;}
            
            let csv_input_clone = csv_input_file.clone();
            let xml_input_clone = xml_input_file.clone();

            // lots of checking to make sure output file path is working correctly
            let output_txt = gui.get_output_text();
            if output_txt != "" && output_file.is_none() {
                // gets directory of input file, either csv or xml depending on config
                let input_dir = match config {
                    csv_conf if csv_conf.csv_class_percent_enabled && csv_conf.csv_stat_columns_enabled => {
                        match csv_input_clone{
                            Some(ref pathbuf) => match pathbuf.parent() {
                                Some(parent) => String::from(parent.to_string_lossy()),
                                None => "".to_string(),
                            },
                            None => "".to_string(),
                        }//end matching for directory of csv input file
                    },
                    xml_conf if xml_conf.xml_sieve_cols_enabled => {
                        match xml_input_clone {
                            Some(ref pathbuf) => match pathbuf.parent() {
                                Some(parent) => String::from(parent.to_string_lossy()),
                                None => "".to_string(),
                            },
                            None => "".to_string(),
                        }//end matching for directory of xml input file
                    },
                    _ => String::from(""),
                };
                if input_dir != "" {
                    let mut output_pathbuf = PathBuf::new();
                    output_pathbuf.push(input_dir);
                    output_pathbuf.push(output_txt.clone());
                    output_pathbuf.set_extension("xlsx");
                    if !output_pathbuf.exists() || gui.integrated_dialog_yes_no("The output file you specified already exists.\nAre you sure you want to replace it?") {
                        *output_file = Some(output_pathbuf);
                    }//end if the file doesn't exist OR user is fine with overwriting it
                }//end if we were able to get the input directory
            }//end if we need to update output file name from user entered text

            match output_file {
                Some(output) => {
                    output.set_file_name(output_txt);
                    output.set_extension("xlsx");
                },
                None => {gui.integrated_dialog_alert("Please select a name or path for the output file!"); return false;}
            }//end ensureing output_file is fine for use
        },
        None => {gui.integrated_dialog_alert("Couldn't Access Configuration Settings When Attempting Processing! Aborting!"); return false;}
    }//end matching whether we actually have config to go off of

    return true;
}//end ensure_data_valid_for_output()

/// Gets the config information from the config file.  
/// If we encounter issues with that, walk the user through a fix via the gui.
fn ensure_config_valid(gui: &mut GUI, config_store: &mut Option<ConfigStore>, config_path: &mut Option<PathBuf>, config_name: &str) {
    *config_path = None;
    *config_store = None;
    
    match config_store::try_read_config_path(config_name, false) {
        Ok(config_path_tmp) => {
            if !config_path_tmp.exists() {
                let should_create_personal_file = gui.integrated_dialog_yes_no("The configuration hasn't been set up yet.\nWould you like to choose a preset configuration?");
                let mut new_conf_stor = ConfigStore::default();
                if should_create_personal_file {
                    match gui.integrated_dialog_message_choice("Please choose the config preset you want.", vec!["Rhett", "Scott", "Other"]) {
                        Some(0) => new_conf_stor = config_store::get_rhett_config(),
                        Some(1) => new_conf_stor = config_store::get_scott_config(),
                        Some(2) => new_conf_stor = ConfigStore::default(),
                        _ => gui.integrated_dialog_message("Guided configuration setting cancelled (somehow?). We'll just use the default then."),
                    }//end matching dialog result for config preset
                }//end if we should create a personalized config file
                match config_store::try_write_config(&config_path_tmp, &new_conf_stor) {
                    Ok(_) => {
                        gui.set_config_store(&new_conf_stor);
                        *config_store = Some(new_conf_stor);
                        // gui.integrated_dialog_message("Your configuration was successfully written and set.\nIf you continue seeing messages about the config file when opening the program, please contact the developer.");
                    },
                    Err(msg) => gui.integrated_dialog_alert(&format!("It seems we were unable to write the new configuration to a file,\nthough you should still be able to the program for now with the config you selected.\nError message was \"{}\".\nIf this operation keeps failing, please contact the developer.", msg))
                }//end matching whether or not we successfully wrote a new config file
            }//end if config_path_tmp doesn't point to a real file
            else {
                match config_store::try_read_config(&config_path_tmp) {
                    Ok(config_store_tmp) => {
                        gui.set_config_store(&config_store_tmp);
                        *config_store = Some(config_store_tmp);
                    },
                    Err(msg) => {
                        gui.integrated_dialog_alert(&format!("Could not read config file at path \"{}\".\nReceived error msg {}", config_path_tmp.to_string_lossy(), msg));
                        let should_create_new = gui.integrated_dialog_yes_no("Problems with the config file might occur when changing versions.\nWhen the config file is deleted, the program will automatically create a new one by default.\nEven if a config file is not loaded, you can always set the config yourself using the section in the bottom right.\n\nWould you like to be delete the old config file and create a personalized one now?");
                        if should_create_new {
                            let mut new_conf_stor: Option<ConfigStore> = None;
                            match gui.integrated_dialog_message_choice("Do you want a personalized config file?\nIf so, choose which preset you want:", vec!["Rhett", "Scott", "Other"]) {
                                Some(0) => new_conf_stor = Some(config_store::get_rhett_config()),
                                Some(1) => new_conf_stor = Some(config_store::get_scott_config()),
                                Some(2) => new_conf_stor = Some(ConfigStore::default()),
                                _ => gui.integrated_dialog_message("Guided configuration setting cancelled."),
                            }//end matching dialog result
                            if let Some(new_conf_stor) = new_conf_stor {
                                match config_store::try_write_config(&config_path_tmp, &new_conf_stor) {
                                    Ok(_) => {
                                        gui.integrated_dialog_message("Congrats, we successfully wrote your changes to the config file.\nWhatever the problem was, it should be fixed.\nIf you continue seeing messages about this everytime you open the application, please contact the developer.");
                                        *config_store = Some(new_conf_stor);
                                    },
                                    Err(msg) => gui.integrated_dialog_alert(&format!("We couldn't write your config to the file, though you should still be able\nto use the program for now with the config you selected.\nError message was \"{}\".\nIf this operation keeps failing, please contact the developer.", msg))
                                }//end matching whether or not we can write to file
                            }//end if we have a new config store to write
                        }//end if we get the ok to make a new config file
                    }//end case of not being able to parse file at config_path_tmp
                }//end matching whether we can read file at config_path_tmp
                *config_path = Some(config_path_tmp);
            }//end else the config file already exists
        },
        Err(msg) => gui.integrated_dialog_alert(&format!("Could not determine the path to the config.\nReceived error msg {}", msg))
    }//end matching whether or not we can get config path
}
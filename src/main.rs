use std::path::PathBuf;

use crate::{data::{Data, DataVal}, gui::GUI};

mod gui;
mod data;

fn main() {
    // set up main application and window
    // let mut gui = GUI::initialize();
    // gui.ux_main_window.show();

    // while gui.app.wait() {

    // }
    let gui = GUI::initialize();

    let recv = gui.get_receiver();
    let mut input_data = None;
    let mut output_file = None;

    while gui.wait() {
        if let Some(msg) = recv.recv() {
            match msg.as_str() {
                "CSV::GetInputFile" => {
                    // try to get file
                    let path_buf = GUI::get_file_to_open();
                    match csv::Reader::from_path(path_buf.clone()) {
                        Ok(reader) => {
                            println!("We got the csv reader");
                            let data = Data::from_csv_reader(reader).unwrap();
                            println!("We finished reading {} records from the csv", data.get_records().len());
                            input_data = Some(data);
                            // format_csv_sum(&data);
                        },
                        Err(_) => println!("Couldn't get csv reader."),
                    }//end matching result of getting csv reader
                },
                "CSV::GetOutputFile" => {
                    let path_buf = GUI::get_file_to_save();
                    output_file = Some(path_buf);
                },
                "CSV::Process" => {
                    match &input_data {
                        Some(input) => {
                            match &output_file {
                                Some(output) => {
                                    println!("Started processing and outputing file.");
                                    output_csv_sum(input, output);
                                    input_data = None;
                                    output_file = None;
                                    println!("Finished outputing processed file.");
                                },
                                None => println!("No Output File Selected")
                            }//end matching existence of output file
                        },
                        None => println!("No Input File Loaded")
                    }//end matching existence of input file
                }//end matching CSV::process
                _ => println!("Unrecognized message {}.", msg)
            }//end matching the message we recieved
        }//end if we recieved a message
    }//end main application loop

    println!("Program Exiting!");
}

fn output_csv_sum(data: &Data, output_path: &PathBuf) {
    let base_data = data.get_records();
    // filter so that we only have Sound data
    let sound_data = data::get_filtered_records(&base_data, 5, DataVal::String(String::from("Sound")));
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
            let avgs = data::get_col_avg(&records, col_idx);
            let stdevs = data::get_col_stdev(&records, col_idx);
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
    let sound_data = data::get_filtered_records(&base_data, 5, DataVal::String("Sound".to_string()));
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
            let avgs = data::get_col_avg(&records, col_idx);
            let stdevs = data::get_col_stdev(&records, col_idx);
            print!("{:.2}\t{:.2}\t", avgs.1, stdevs.1);
        }//end looping over column indices 11-24
        print!("\n");
    }//end looping over split groups and doing avg
}//end format_csv_sum
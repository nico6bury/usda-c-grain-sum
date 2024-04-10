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

    while gui.wait() {
        if let Some(msg) = recv.recv() {
            match msg.as_str() {
                "GetFile" => {
                    // try to get file
                    let path_buf = GUI::get_file();
                    match csv::Reader::from_path(path_buf) {
                        Ok(reader) => {
                            println!("We got the csv reader");
                            let data = Data::from_csv_reader(reader).unwrap();
                            println!("We finished reading {} records from the csv", data.get_records().len());
                            format_csv_sum(&data);
                        },
                        Err(_) => println!("Couldn't get csv reader."),
                    }//end matching result of getting csv reader
                },
                _ => println!("Unrecognized message {}.", msg)
            }
        }
    }

    println!("Program Exiting!");
}

/// Formats or outputs a csv file in the format desired
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
    // for group in split_data {
    //     let avgs = data::get_col_avg(&group.1, 11);
    //     println!("Area averages for group {:?} were {:?}", group.0, avgs);
    // }//end looping over groups
}//end format_csv_sum
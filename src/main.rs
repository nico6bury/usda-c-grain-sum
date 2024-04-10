use crate::{data::Data, gui::GUI};

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
                            let data = Data::from_csv_reader(reader);
                            println!("We finished reading {} records from the csv", data.unwrap().get_records().len());
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

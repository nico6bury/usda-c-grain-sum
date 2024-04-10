
use crate::gui::GUI;

mod gui;

fn main() {
    // set up main application and window
    // let mut gui = GUI::initialize();
    // gui.ux_main_window.show();

    // while gui.app.wait() {

    // }
    let gui = GUI::initialize();

    while gui.wait() {

    }

    println!("Program Exiting!");
}

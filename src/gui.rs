use fltk::{app::{self, App, Receiver, Sender}, button::{self, Button}, dialog, frame::Frame, prelude::{DisplayExt, GroupExt, WidgetBase, WidgetExt}, text, window::{self, Window}};

#[allow(dead_code)]
pub struct GUI {
    pub app: App,
    pub ux_main_window: Window,
    msg_sender: Sender<String>,
    msg_receiver: Receiver<String>,
}//end struct GUI

#[allow(dead_code)]
impl GUI {
    /// Returns a reference to receiver so you can
    /// react to messages sent by gui.
    pub fn get_receiver(&self) -> &Receiver<String> {
        return &self.msg_receiver;
    }//end get_receiver(self)

    /// Sets up all the properties and appearances of
    /// various widgets and UI settings.
    pub fn initialize() -> GUI {
        let c_grain_app = app::App::default();
        let mut main_window = window::Window::default().with_size(600, 300).with_label("USDA C-Grain Summarizer");

        let (s,r) = app::channel();

        // set up header information
        let mut header_buf = text::TextBuffer::default();
        let mut header_box = text::TextDisplay::default().with_pos(10, 10).with_size(580, 140);
        header_box.set_buffer(header_buf.clone());

        header_buf.append("C-Grain Summarizer v##.##\n");
        header_buf.append("USDA-ARS Manhattan, KS\n");

        // get input file from user
        let mut input_file_btn = Button::default()
            .with_label("Select Input CSV")
            .with_pos(header_box.x(), header_box.y() + header_box.h() + 10)
            .with_size(125, 25);
        input_file_btn.emit(s.clone(), "GetFile".to_string());

        main_window.end();
        main_window.show();

        GUI {
            app: c_grain_app,
            ux_main_window: main_window,
            msg_sender: s,
            msg_receiver: r,
        }
    }

    /// Makes the main window visible.
    pub fn show(&mut self) {
        self.ux_main_window.show();
    }//end show(self)

    /// Wraps app.wait().  
    /// To run main app use, use while(gui.wait()){}.
    pub fn wait(&self) -> bool {
        self.app.wait()
    }//end wait(&self)

    /// Gets a file from the user
    pub fn get_file() -> std::path::PathBuf {
        let mut dialog = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseFile);
        dialog.show();
        dialog.filename()
    }//end get_file()
}//end impl for GUI
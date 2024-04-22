use fltk::{app::{self, App, Receiver, Sender}, button::Button, dialog, enums::FrameType, group::{Group, Tile}, prelude::{DisplayExt, GroupExt, WidgetExt}, text, window::{self, Window}};

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
        main_window.end();

        let (s,r) = app::channel();

        let mut tile_group = Tile::default()
            .with_pos(0, 0)
            .with_size(main_window.width(), main_window.height());
        tile_group.end();
        main_window.add(&tile_group);

        // set up header information
        let mut header_group = Group::default()
            .with_pos(0,0)
            .with_size(tile_group.width(), tile_group.height() / 3);
        header_group.end();
        tile_group.add(&header_group);

        let mut header_buf = text::TextBuffer::default();
        let mut header_box = text::TextDisplay::default()
            .with_pos(10, 10)
            .with_size(header_group.width() - 20,header_group.height() - 20);
        header_group.add(&header_box);
        header_box.set_buffer(header_buf.clone());
        header_buf.append("C-Grain Summarizer v##.##\n");
        header_buf.append("USDA-ARS Manhattan, KS\n");

        // set up group with input and output controls, processing stuff
        let mut io_controls_group = Group::default()
            .with_pos(0, header_group.y() + header_group.height())
            .with_size(tile_group.width() / 3 * 2, tile_group.height() - header_group.height());
        io_controls_group.end();
        tile_group.add(&io_controls_group);

        // get input file from user
        let mut input_file_btn = Button::default()
            .with_label("Select Input CSV")
            .with_pos(io_controls_group.x() + 10, io_controls_group.y() + 10)
            .with_size(125, 25);
        input_file_btn.emit(s.clone(), String::from("CSV::GetInputFile"));
        input_file_btn.set_frame(FrameType::GtkRoundUpFrame);
        io_controls_group.add(&input_file_btn);

        // get output file from user
        let mut output_file_btn = Button::default()
            .with_label("Select Output CSV")
            .with_pos(input_file_btn.x(), input_file_btn.y() + input_file_btn.h() + 10)
            .with_size(125, 25);
        output_file_btn.emit(s.clone(), String::from("CSV::GetOutputFile"));
        output_file_btn.set_frame(FrameType::GtkRoundUpFrame);
        io_controls_group.add(&output_file_btn);

        // process the data we have
        let mut process_file_btn = Button::default()
            .with_label("Process Data")
            .with_pos(output_file_btn.x(), output_file_btn.y() + output_file_btn.h() + 10)
            .with_size(125, 25);
        process_file_btn.emit(s.clone(), String::from("CSV::Process"));
        process_file_btn.set_frame(FrameType::PlasticDownBox);
        io_controls_group.add(&process_file_btn);

        // set up group with configuration options
        let mut config_group = Group::default()
            .with_pos(io_controls_group.x() + io_controls_group.w(), io_controls_group.y())
            .with_size(tile_group.width() - io_controls_group.width(), tile_group.height() - header_group.height());
        config_group.end();
        tile_group.add(&config_group);

        // set frame type for borders between sections, make sure to use box type
        header_group.set_frame(FrameType::GtkUpBox);
        io_controls_group.set_frame(FrameType::GtkUpBox);
        config_group.set_frame(FrameType::GtkUpBox);

        main_window.make_resizable(true);
        main_window.show();

        GUI {
            app: c_grain_app,
            ux_main_window: main_window,
            msg_sender: s,
            msg_receiver: r,
        }//end struct construction
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

    /// Gets a file from the user to open.
    pub fn get_file_to_open() -> std::path::PathBuf {
        let mut dialog = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseFile);
        dialog.show();
        dialog.filename()
    }//end get_file()

    /// Gets a file from the user to save.
    pub fn get_file_to_save() -> std::path::PathBuf {
        let mut dialog = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseSaveFile);
        dialog.show();
        dialog.filename()
    }//end get_file_to_save()
}//end impl for GUI
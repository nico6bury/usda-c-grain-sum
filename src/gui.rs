use fltk::{app::{self, App, Receiver, Sender}, button::{Button, CheckButton}, dialog, enums::{Align, FrameType}, frame::Frame, group::{Group, Tile}, prelude::{DisplayExt, GroupExt, WidgetExt}, text::{TextBuffer, TextDisplay, TextEditor}, window::{self, Window}};

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
        let mut main_window = window::Window::default().with_size(700, 400).with_label("USDA C-Grain Summarizer");
        main_window.end();

        // define some constants to be used repeatedly for sizing and styling
        let io_btn_width = 150;
        let io_btn_height = 30;
        let io_btn_padding = 10;
        let io_btn_frame = FrameType::GtkRoundUpFrame;
        // let io_box_width = 240; boxes are centered between btn and rest of space in tile
        let io_box_height = 30;
        let io_box_padding = 10;
        let io_box_frame = FrameType::GtkDownFrame;
        let cf_padding = 5;
        let cf_chck_height = 20;
        let cf_chck_frame = FrameType::GtkUpFrame;
        let cf_box_frame = FrameType::GtkDownFrame;

        let (s,r) = app::channel();

        let mut tile_group = Tile::default()
            .with_pos(0, 0)
            .with_size(main_window.w(), main_window.h());
        tile_group.end();
        main_window.add(&tile_group);

        // set up header information
        let mut header_group = Group::default()
            .with_pos(0,0)
            .with_size(tile_group.w(), tile_group.h() / 9 * 4);
        header_group.end();
        tile_group.add(&header_group);

        let mut header_buf = TextBuffer::default();
        let mut header_box = TextDisplay::default()
            .with_pos(10, 10)
            .with_size(header_group.w() - 20,header_group.h() - 20);
        header_group.add_resizable(&header_box);
        header_box.set_buffer(header_buf.clone());
        let version = option_env!("CARGO_PKG_VERSION");
        let format_des = time::macros::format_description!("[month repr:long] [year]");
        let date = compile_time::date!();
        header_buf.append("USDA-ARS Manhattan, KS\tC-Grain Summarizer\n");
        header_buf.append(&format!("{}\tv{}\t\tNicholas Sixbury/Dan Brabec\n", date.format(format_des).unwrap_or(String::from("unknown compile time")) ,version.unwrap_or("unknown version")));
        header_buf.append("Processes CSV and XML Data from C-Grain into Sum Files\n");
        header_buf.append("\nCurrent Config Info:\n");
        header_buf.append("Filtering for Classification: Any | Sound | Sorghum\n");
        header_buf.append("Stat Columns: Area, Length, Width, Thickness, Ratio, Mean Width, HSV, RGB\n");
        header_buf.append("Classification Percent Columns: Yes\n");
        header_buf.append("XML Sieve Data: Yes");

        // set up group with input and output controls, processing stuff
        let mut io_controls_group = Group::default()
            .with_pos(0, header_group.y() + header_group.h())
            .with_size(tile_group.w() / 7 * 4, tile_group.h() - header_group.h());
        io_controls_group.end();
        tile_group.add(&io_controls_group);

        let io_controls_label = Frame::default()
            .with_pos(io_controls_group.x(), io_controls_group.y() + 10)
            .with_size(io_controls_group.w(), 20)
            .with_label("Input and Output Controls")
            .with_align(Align::Center);
        io_controls_group.add(&io_controls_label);

        // get input file from user
        let mut input_csv_btn = Button::default()
            .with_label("Select Input CSV")
            .with_pos(io_controls_label.x() + io_btn_padding, io_controls_label.y() +  io_controls_label.h() + io_btn_padding)
            .with_size(io_btn_width, io_btn_height);
        input_csv_btn.emit(s.clone(), String::from("CSV::GetInputFile"));
        input_csv_btn.set_frame(io_btn_frame);
        io_controls_group.add(&input_csv_btn);

        let input_csv_buf = TextBuffer::default();
        let mut input_csv_box = TextEditor::default()
            .with_pos(input_csv_btn.x() + input_csv_btn.w() + io_box_padding, input_csv_btn.y())
            .with_size(io_controls_group.w() - (input_csv_btn.w() + (3 * io_box_padding)), io_box_height);
        input_csv_box.set_frame(io_box_frame);
        input_csv_box.set_buffer(input_csv_buf);
        io_controls_group.add_resizable(&input_csv_box);

        let mut input_xml_btn = Button::default()
            .with_label("Select Input XML")
            .with_pos(input_csv_btn.x(), input_csv_btn.y() + input_csv_btn.h() + io_btn_padding)
            .with_size(io_btn_width, io_btn_height);
        input_xml_btn.emit(s.clone(), String::from("XML::GetInputFile"));
        input_xml_btn.set_frame(io_btn_frame);
        io_controls_group.add(&input_xml_btn);

        let input_xml_buf = TextBuffer::default();
        let mut input_xml_box = TextEditor::default()
            .with_pos(input_xml_btn.x() + input_xml_btn.w() + io_box_padding, input_xml_btn.y())
            .with_size(io_controls_group.w() - (input_xml_btn.w() + (3 * io_box_padding)), io_box_height);
        input_xml_box.set_frame(io_box_frame);
        input_xml_box.set_buffer(input_xml_buf);
        io_controls_group.add_resizable(&input_xml_box);

        // get output file from user
        let mut output_file_btn = Button::default()
            .with_label("Select Output CSV")
            .with_pos(input_xml_btn.x(), input_xml_btn.y() + input_xml_btn.h() + io_btn_padding)
            .with_size(io_btn_width, io_btn_height);
        output_file_btn.emit(s.clone(), String::from("CSV::GetOutputFile"));
        output_file_btn.set_frame(io_btn_frame);
        io_controls_group.add(&output_file_btn);

        let output_file_buf = TextBuffer::default();
        let mut output_file_box = TextEditor::default()
            .with_pos(output_file_btn.x() + output_file_btn.w() + io_box_padding, output_file_btn.y())
            .with_size(io_controls_group.w() - (output_file_btn.w() + (3 * io_box_padding)), io_box_height);
        output_file_box.set_frame(io_box_frame);
        output_file_box.set_buffer(output_file_buf);
        io_controls_group.add_resizable(&output_file_box);

        // process the data we have
        let mut process_file_btn = Button::default()
            .with_label("Process Data")
            .with_pos(output_file_btn.x() + 60, output_file_btn.y() + output_file_btn.h() + 10)
            .with_size(250, 50);
        process_file_btn.emit(s.clone(), String::from("CSV::Process"));
        process_file_btn.set_frame(FrameType::PlasticDownBox);
        io_controls_group.add_resizable(&process_file_btn);

        // set up group with configuration options
        let mut config_group = Group::default()
            .with_pos(io_controls_group.x() + io_controls_group.w(), io_controls_group.y())
            .with_size(tile_group.width() - io_controls_group.width(), tile_group.height() - header_group.height());
        config_group.end();
        tile_group.add(&config_group);

        let config_label = Frame::default()
            .with_pos(config_group.x(), config_group.y() + 10)
            .with_size(config_group.width(), 20)
            .with_label("Configuration Options")
            .with_align(Align::Center);
        config_group.add(&config_label);

        let mut class_filter_chck = CheckButton::default()
            .with_pos(config_label.x() + cf_padding, config_label.y() + config_label.h() + cf_padding)
            .with_size(180,cf_chck_height)
            .with_label("Filter to Classification of:");
        class_filter_chck.set_checked(true);
        class_filter_chck.set_frame(cf_chck_frame);
        config_group.add(&class_filter_chck);

        let mut class_filter_buf = TextBuffer::default();
        let mut class_filter_box = TextEditor::default()
            .with_pos(class_filter_chck.x() + class_filter_chck.w() + cf_padding, class_filter_chck.y())
            .with_size(config_group.width() - (class_filter_chck.w() + (cf_padding * 3)), 25);
        class_filter_buf.set_text("Sound");
        class_filter_box.set_frame(cf_box_frame);
        class_filter_box.set_buffer(class_filter_buf);
        config_group.add_resizable(&class_filter_box);

        let mut stat_cols_chck = CheckButton::default()
            .with_pos(class_filter_chck.x(), class_filter_chck.y() + class_filter_chck.h() + cf_padding)
            .with_size(config_group.w() - cf_padding * 2, cf_chck_height)
            .with_label("Output Stat Columns from CSV Columns:");
        stat_cols_chck.set_checked(true);
        stat_cols_chck.set_frame(cf_chck_frame);
        config_group.add(&stat_cols_chck);

        let mut stat_cols_buf = TextBuffer::default();
        let mut stat_cols_box = TextEditor::default()
            .with_pos(stat_cols_chck.x(), stat_cols_chck.y() + stat_cols_chck.h() + cf_padding)
            .with_size(stat_cols_chck.w(), 75);
        stat_cols_buf.set_text("Area\nLength\nWidth Thickness Ratio\nHue Brightness Saturation Red Green Blue");
        stat_cols_box.set_buffer(stat_cols_buf);
        stat_cols_box.set_frame(cf_box_frame);
        config_group.add_resizable(&stat_cols_box);

        let mut class_perc_chck = CheckButton::default()
            .with_pos(stat_cols_chck.x(), stat_cols_box.y() + stat_cols_box.h() + cf_padding)
            .with_size(stat_cols_chck.w(), cf_chck_height)
            .with_label("Outut Classification Percentages from CSV");
        class_perc_chck.set_checked(true);
        class_perc_chck.set_frame(cf_chck_frame);
        config_group.add(&class_perc_chck);

        let mut xml_sieve_chck = CheckButton::default()
            .with_pos(class_perc_chck.x(), class_perc_chck.y() + class_perc_chck.h() + cf_padding)
            .with_size(stat_cols_chck.w(), cf_chck_height)
            .with_label("Output XML Sieve Data if Found");
        xml_sieve_chck.set_checked(true);
        xml_sieve_chck.set_frame(cf_chck_frame);
        config_group.add(&xml_sieve_chck);

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
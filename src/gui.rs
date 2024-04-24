use std::{cell::RefCell, rc::Rc};

use fltk::{app::{self, App, Receiver, Sender}, button::{Button, CheckButton}, dialog::{self}, enums::{Align, Event, FrameType}, frame::Frame, group::{Group, Tile}, prelude::{DisplayExt, GroupExt, WidgetBase, WidgetExt, WindowExt}, text::{TextBuffer, TextDisplay, TextEditor}, window::{self, Window}};

use crate::config_store::ConfigStore;

#[allow(dead_code)]
/// This struct represents a graphical user interface for the program.
/// The program is meant to be written in an MVC way, without the GUI
/// having a lot of control over processing, instead just letting the
/// main file/controller handle things by reacting to the Receiver
/// retrieved from get_receiver().
pub struct GUI {
    /// The main app struct. Used for event handling stuff later.
    app: App,
    /// The main window struct. Holds all the other controls.
    ux_main_window: Window,
    /// Holds debug messages sent by main
    debug_log: Vec<String>,
    /// Message Sender, used to send button events to main, essentially.
    msg_sender: Sender<String>,
    /// Message Receiver, we give a reference to this to main, 
    /// allowing it to receive our messages.
    msg_receiver: Receiver<String>,
    /// Buffer holding text in the header display
    ux_header_buf: TextBuffer,
    /// Buffer holding the filename/path for input csv file.
    ux_input_csv_txt: Rc<RefCell<TextEditor>>,
    /// Buffer holding the filename/path for input xml file.
    ux_input_xml_txt: Rc<RefCell<TextEditor>>,
    /// Buffer holding the filename/path for the output file.
    ux_output_file_txt: Rc<RefCell<TextEditor>>,
    /// Check button in config section.  
    /// Tells whether or not we should be filtering input
    /// csv data to only include rows with a specific classification.
    ux_cf_class_filter_chck: CheckButton,
    /// Text buffer in config section.  
    /// If we're filtering input csv data to only inlclude rows
    /// with a specific classification, this tells us what
    /// classification we're filtering for, such as "Sound".
    ux_cf_class_filter_buf: TextBuffer,
    /// Check button in config section.  
    /// Tells us whether we should include columns in output
    /// that are essentially statistics about certain columns
    /// in the input csv.
    ux_cf_stat_cols_chck: CheckButton,
    /// Text buffer in config section.  
    /// If we're including columns in the output that are essentially
    /// statistics about certain columns in the input csv, this tells
    /// us which columns in the input csv to do statistics on.
    ux_cf_stat_cols_buf: TextBuffer,
    /// Check button in config section.  
    /// Tells us whether we should include columns in the output
    /// about what percentage of each sample has each classification.  
    /// So, %Sound, %Sorghum, etc.
    ux_cf_class_perc_chck: CheckButton,
    /// Check button in config section.  
    /// Tells us whether we should include columns in the output
    /// that are pulled from sieve data in the xml file. If no
    /// xml file is loaded, then this is meaningless.
    ux_cf_xml_sieve_chck: CheckButton,
}//end struct GUI

#[allow(dead_code)]
impl GUI {
    /// Returns a clone of the receiver so you can
    /// react to messages sent by gui.
    pub fn get_receiver(&self) -> Receiver<String> {
        return self.msg_receiver.clone();
    }//end get_receiver(self)

    /// Sets up all the properties and appearances of
    /// various widgets and UI settings.
    pub fn initialize() -> GUI {
        let c_grain_app = app::App::default();
        let mut main_window = window::Window::default().with_size(700, 325).with_label("USDA C-Grain Summarizer");
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
            .with_size(tile_group.w(), tile_group.h() / 13 * 4);
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
        // header_buf.append("\nCurrent Config Info:\n");
        // header_buf.append("Filtering for Classification: Any | Sound | Sorghum\n");
        // header_buf.append("Stat Columns: Area, Length, Width, Thickness, Ratio, Mean Width, HSV, RGB\n");
        // header_buf.append("Classification Percent Columns: Yes\n");
        // header_buf.append("XML Sieve Data: Yes");
        header_box.set_scrollbar_align(Align::Right);

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
        // input_csv_btn.emit(s.clone(), String::from("CSV::GetInputFile"));
        input_csv_btn.set_frame(io_btn_frame);
        io_controls_group.add(&input_csv_btn);

        let input_csv_buf = TextBuffer::default();
        let mut input_csv_box = TextEditor::default()
            .with_pos(input_csv_btn.x() + input_csv_btn.w() + io_box_padding, input_csv_btn.y())
            .with_size(io_controls_group.w() - (input_csv_btn.w() + (3 * io_box_padding)), io_box_height);
        input_csv_box.set_frame(io_box_frame);
        input_csv_box.set_scrollbar_align(Align::Bottom);
        input_csv_box.set_scrollbar_size(7);
        input_csv_box.set_buffer(input_csv_buf.clone());
        io_controls_group.add_resizable(&input_csv_box);
        let input_csv_ref = Rc::from(RefCell::from(input_csv_box));
        // let input_csv_pathbuf = Rc::from(RefCell::from(None));

        input_csv_btn.handle({
            let input_csv_ref_clone = input_csv_ref.clone();
            let sender_clone = s.clone();
            move |_, ev| {
                let input_csv_ref = input_csv_ref_clone.as_ref().borrow();
                match ev {
                    Event::Released => {
                        if let Err(err_message) = GUI::create_io_dialog(&sender_clone, "IO::CSVInputFile", &input_csv_ref, dialog::NativeFileChooserType::BrowseFile, dialog::NativeFileChooserOptions::UseFilterExt, "*.csv", "Please select a csv input file") {
                            println!("Encountered an error when attempting to show file dialog:\n{}", err_message);
                        }//end if we got an error
                        true
                    },
                    _ => false,
                }//end matching events
            }//end moving closure for button event
        });

        let mut input_xml_btn = Button::default()
            .with_label("Select Input XML")
            .with_pos(input_csv_btn.x(), input_csv_btn.y() + input_csv_btn.h() + io_btn_padding)
            .with_size(io_btn_width, io_btn_height);
        // input_xml_btn.emit(s.clone(), String::from("IO::XMLInputFile"));
        input_xml_btn.set_frame(io_btn_frame);
        io_controls_group.add(&input_xml_btn);

        let input_xml_buf = TextBuffer::default();
        let mut input_xml_box = TextEditor::default()
            .with_pos(input_xml_btn.x() + input_xml_btn.w() + io_box_padding, input_xml_btn.y())
            .with_size(io_controls_group.w() - (input_xml_btn.w() + (3 * io_box_padding)), io_box_height);
        input_xml_box.set_frame(io_box_frame);
        input_xml_box.set_scrollbar_align(Align::Bottom);
        input_xml_box.set_scrollbar_size(7);
        input_xml_box.set_buffer(input_xml_buf.clone());
        io_controls_group.add_resizable(&input_xml_box);
        let input_xml_ref = Rc::from(RefCell::from(input_xml_box));
        // let input_xml_pathbuf = Rc::from(RefCell::from(None));

        input_xml_btn.handle({
            let input_xml_ref_clone = input_xml_ref.clone();
            let sender_clone = s.clone();
            move |_, ev| {
                let input_xml_ref = input_xml_ref_clone.as_ref().borrow();
                match ev {
                    Event::Released => {
                        if let Err(err_message) = GUI::create_io_dialog(&sender_clone, "IO::XMLInputFile", &input_xml_ref, dialog::NativeFileChooserType::BrowseFile, dialog::NativeFileChooserOptions::UseFilterExt, "*.xml", "Please select an xml input file") {
                            println!("Encountered an error when attempting to show file dialog:\n{}", err_message);
                        }//end if we got an error
                        true
                    },
                    _ => false,
                }//end matching events
            }//end moving closure for button event
        });

        // get output file from user
        let mut output_file_btn = Button::default()
            .with_label("Select Output CSV")
            .with_pos(input_xml_btn.x(), input_xml_btn.y() + input_xml_btn.h() + io_btn_padding)
            .with_size(io_btn_width, io_btn_height);
        // output_file_btn.emit(s.clone(), String::from("IO::OutputFile"));
        output_file_btn.set_frame(io_btn_frame);
        io_controls_group.add(&output_file_btn);

        let output_file_buf = TextBuffer::default();
        let mut output_file_box = TextEditor::default()
            .with_pos(output_file_btn.x() + output_file_btn.w() + io_box_padding, output_file_btn.y())
            .with_size(io_controls_group.w() - (output_file_btn.w() + (3 * io_box_padding)), io_box_height);
        output_file_box.set_frame(io_box_frame);
        output_file_box.set_scrollbar_align(Align::Bottom);
        output_file_box.set_scrollbar_size(7);
        output_file_box.set_buffer(output_file_buf.clone());
        io_controls_group.add_resizable(&output_file_box);
        let output_file_ref = Rc::from(RefCell::from(output_file_box));
        // let output_file_pathbuf = Rc::from(RefCell::from(None));

        output_file_btn.handle({
            let output_file_ref_clone = output_file_ref.clone();
            let sender_clone = s.clone();
            move |_, ev| {
                let output_file_ref = output_file_ref_clone.as_ref().borrow();
                match ev {
                    Event::Released => {
                        if let Err(err_message) = GUI::create_io_dialog(&sender_clone, "IO::OutputFile", &output_file_ref, dialog::NativeFileChooserType::BrowseSaveFile, dialog::NativeFileChooserOptions::SaveAsConfirm, "", "Please specify the output file.") {
                            println!("Encountered an error when attempting to show file dialog:\n{}", err_message);
                        }//end if we got an error
                        true
                    },
                    _ => false,
                }//end matching events
            }//end moving closure for button event
        });

        // process the data we have
        let mut process_file_btn = Button::default()
            .with_label("Process Data")
            .with_pos(output_file_btn.x() + 60, output_file_btn.y() + output_file_btn.h() + 10)
            .with_size(250, 50);
        process_file_btn.emit(s.clone(), String::from("Proc::Sum"));
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
        class_filter_chck.set_tooltip("If checked, processing will only consider rows in csv data matching the given classification(s).");
        config_group.add(&class_filter_chck);

        let mut class_filter_buf = TextBuffer::default();
        let mut class_filter_box = TextEditor::default()
            .with_pos(class_filter_chck.x() + class_filter_chck.w() + cf_padding, class_filter_chck.y())
            .with_size(config_group.width() - (class_filter_chck.w() + (cf_padding * 3)), 25);
        class_filter_box.set_buffer(class_filter_buf.clone());
        class_filter_buf.set_text("Sound");
        class_filter_box.set_frame(cf_box_frame);
        class_filter_box.set_tooltip("Separate values by a comma or |. When separating by comma, include 1 or 0 spaces after the comma. When separating by |, include 1 space on either side or no space on either side.");
        class_filter_box.set_scrollbar_align(Align::Clip);
        config_group.add_resizable(&class_filter_box);

        let mut stat_cols_chck = CheckButton::default()
            .with_pos(class_filter_chck.x(), class_filter_chck.y() + class_filter_chck.h() + cf_padding)
            .with_size(config_group.w() - cf_padding * 2, cf_chck_height)
            .with_label("Output Stat Columns from CSV Columns:");
        stat_cols_chck.set_checked(true);
        stat_cols_chck.set_frame(cf_chck_frame);
        stat_cols_chck.set_tooltip("If checked, then columns will be added to the output with the Avg and Stdev per sample of certain columns in the CSV data.");
        config_group.add(&stat_cols_chck);

        let mut stat_cols_buf = TextBuffer::default();
        let mut stat_cols_box = TextEditor::default()
            .with_pos(stat_cols_chck.x(), stat_cols_chck.y() + stat_cols_chck.h() + cf_padding)
            .with_size(stat_cols_chck.w(), 75);
        stat_cols_box.set_buffer(stat_cols_buf.clone());
        stat_cols_buf.set_text("Area, Length, Width, Thickness, \nRatio, Mean Width, Volume, Weight\nLight, Hue, Saturation\nRed, Green, Blue");
        stat_cols_box.set_frame(cf_box_frame);
        stat_cols_box.set_tooltip("Columns in CSV input to do statistics on. Separate values by a new line or comma. When separating by comma, include 1 or 0 spaces after the comma. To get a list of potential column headers, click this box and press F1.");
        stat_cols_box.set_scrollbar_align(Align::Right);
        stat_cols_box.set_scrollbar_size(12);
        config_group.add_resizable(&stat_cols_box);

        stat_cols_box.add_key_binding(fltk::enums::Key::F1, fltk::enums::Shortcut::None, |_, _| {
            dialog::message_title("Some Potential Column Headings");
            dialog::message(0, 0, "Some of the possible column headers are: \nArea, Length, Thickness, Mean Width, Ratio, Volume, Weight, \nBrightness, Hue, Saturation, Red, Green, Blue, Severity.");
            0
        });
        

        let mut class_perc_chck = CheckButton::default()
            .with_pos(stat_cols_chck.x(), stat_cols_box.y() + stat_cols_box.h() + cf_padding)
            .with_size(stat_cols_chck.w(), cf_chck_height)
            .with_label("Outut Classification Percentages from CSV");
        class_perc_chck.set_checked(true);
        class_perc_chck.set_frame(cf_chck_frame);
        class_perc_chck.set_tooltip("If checked, then columns will be added to the output giving the percentage of each sample of each possible classification. These percentages are calculated independently of any other classification fitlering.");
        config_group.add(&class_perc_chck);

        let mut xml_sieve_chck = CheckButton::default()
            .with_pos(class_perc_chck.x(), class_perc_chck.y() + class_perc_chck.h() + cf_padding)
            .with_size(stat_cols_chck.w(), cf_chck_height)
            .with_label("Output XML Sieve Data if Found");
        xml_sieve_chck.set_checked(true);
        xml_sieve_chck.set_frame(cf_chck_frame);
        xml_sieve_chck.set_tooltip("If checked, then columns will be added to the output giving sieve data for each sample. Since this data is only found in the xml file, columns will only be added if an xml input file is loaded.");
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
            debug_log: Vec::new(),
            msg_sender: s,
            msg_receiver: r,
            ux_header_buf: header_buf,
            ux_input_csv_txt: input_csv_ref,
            ux_input_xml_txt: input_xml_ref,
            ux_output_file_txt: output_file_ref,
            ux_cf_class_filter_chck: class_filter_chck,
            ux_cf_class_filter_buf: class_filter_buf,
            ux_cf_stat_cols_chck: stat_cols_chck,
            ux_cf_stat_cols_buf: stat_cols_buf,
            ux_cf_class_perc_chck: class_perc_chck,
            ux_cf_xml_sieve_chck: xml_sieve_chck,
        }//end struct construction
    }

    /// Helper method used in initialize to share code between handlers
    /// of io buttons.
    fn create_io_dialog(sender: &Sender<String>, msg_header: &str, txt: &TextEditor, dialog_type: dialog::NativeFileChooserType, dialog_option: dialog::NativeFileChooserOptions, dialog_filter: &str, dialog_title: &str ) -> Result<(), String> {
        // make sure textbuffer is accessible
        let mut txt_buf = match txt.buffer() {
            Some(buf) => buf,
            None => {
                return Err(format!("For some reason we couldn't access teh textbuffer. Oops. This should never happen."));
            }};
        // set up dialog with all the settings
        let mut dialog = dialog::NativeFileChooser::new(dialog_type);
        dialog.set_option(dialog_option);
        dialog.set_filter(dialog_filter);
        dialog.set_title(dialog_title);
        dialog.show();
        // make sure the dialog didn't have an error
        let dialog_error = dialog.error_message().unwrap_or_else(|| "".to_owned()).replace("No error", "");
        if dialog_error != "" {
            return Err(format!("We encountered a dialog error somehow. Details below:\n{}", dialog_error));
        }//end if dialog had an error
        // make sure we can get the file from the dialog
        match dialog.filename().as_os_str().to_str() {
            Some(filepath) => {
                match dialog.filename().file_name() {
                    Some(filename_os) => {
                        match filename_os.to_str() {
                            Some(filename) => {
                                txt_buf.set_text(filename);
                                sender.send(format!("{}::{}", msg_header, filepath));
                            }, None => return Err(format!("Couldn't get filename for some reason. Failed at to_str()."))
                        }}, None => return Err(format!("Couldn't get filename for some reason. Maybe dialog was cancelled."))}
            }, None => return Err(format!("Couldn't get filepath for some reason."))}

        return Ok(());
    }//end create_io_dialog()

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

    /// Simply displays a message to the user.
    pub fn show_message(txt: &str) {
        dialog::message(0, 0, txt);
    }//end show_message(txt)

    /// Creates a ConfigStore from the current config settins, as
    /// chosen by the user.
    pub fn get_config_store(&self) -> ConfigStore {
        let class_filter_txt = self.ux_cf_class_filter_buf.text();
        let stat_columns_txt = self.ux_cf_stat_cols_buf.text();
        // replace multi-char instance we want to split with single chars, then split on '|', ',', or '\n', as needed
        let class_filters: Vec<String> = class_filter_txt.replace(" | ", "|").replace(", ", ",").split(['|',',']).map(|el| el.to_owned()).collect();
        let stat_columns: Vec<String> = stat_columns_txt.replace(", ", ",").split([',','\n']).map(|el| el.to_owned()).collect();

        ConfigStore {
            csv_class_filter_enabled: self.ux_cf_class_filter_chck.is_checked(),
            csv_class_filter_filters: class_filters,
            csv_stat_columns_enabled: self.ux_cf_stat_cols_chck.is_checked(),
            csv_stat_columns_columns: stat_columns,
            csv_class_percent_enabled: self.ux_cf_class_perc_chck.is_checked(),
            xml_sieve_cols_enabled: self.ux_cf_xml_sieve_chck.is_checked(),
        }//end struct construction
    }//end get_config_store

    /// Updates the current configuration widgets in the interface to match
    /// the given ConfigStore.
    pub fn set_config_store(&mut self, config: &ConfigStore) {
        self.ux_cf_class_filter_chck.set_checked(config.csv_class_filter_enabled);
        self.ux_cf_class_filter_buf.set_text(&config.csv_class_filter_filters.join("|"));
        self.ux_cf_stat_cols_chck.set_checked(config.csv_stat_columns_enabled);
        self.ux_cf_stat_cols_buf.set_text(&config.csv_stat_columns_columns.join("\n"));
        self.ux_cf_class_perc_chck.set_checked(config.csv_class_percent_enabled);
        self.ux_cf_xml_sieve_chck.set_checked(config.xml_sieve_cols_enabled);
    }//end set_config_store(self, config)

    /// Gives a small visual indication that the program is doing something in the background.
    pub fn start_wait(&mut self) {
        self.ux_main_window.set_cursor(fltk::enums::Cursor::Wait);
    }//end start_wait(self)

    /// Clears the visual indication from start_wait()
    pub fn end_wait(&mut self) {
        self.ux_main_window.set_cursor(fltk::enums::Cursor::Default);
    }
}//end impl for GUI
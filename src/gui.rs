use std::{cell::RefCell, path::PathBuf, rc::Rc};

use fltk::{app::{self, App, Receiver, Sender}, button::{Button, CheckButton}, dialog::{self, BeepType}, enums::{Align, Color, Event, FrameType}, frame::Frame, group::{Flex, FlexType, Group, Tile}, prelude::{DisplayExt, GroupExt, WidgetBase, WidgetExt, WindowExt}, text::{TextBuffer, TextDisplay, TextEditor}, window::{self, Window}};

use usda_c_grain_sum::config_store::ConfigStore;

/// This enum is specifically intended for message passing
/// from the GUI to the main function. This is done
/// with Sender and Receiver objects created in initialize()
#[derive(Clone,PartialEq,Debug)]
pub enum InterfaceMessage {
    /// Indicates that the user has selected a CSV Input File.
    /// The filepath selected by the user is returned in the message.
    CSVInputFile(PathBuf),
    /// Indicates that the user has selected an XML Input File.
    /// The filepath selected by the user is returned in the message.
    XMLInputFile(PathBuf),
    /// Indicates that the user has selected an Output File.
    /// The filepath selected by the user is returned in the message.
    OutputFile(PathBuf),
    /// Indicates that the user has clicked the Process Button,
    /// so they wish for the output file to be produced.
    ProcessSum,
    /// Indicates that the app is currently closing.
    AppClosing,
    /// Indicates that the user has requested for the current
    /// configuration preset to be reselected, as if they
    /// had started the program for the first time.
    ConfigReset,
    /// Indicates that some other, unidentified message has been
    /// passed. In most cases, this is likely to be a mistake
    /// on the part of the sender.
    Other(String),
}//end enum InterfaceMessage

impl InterfaceMessage {
    /// Generates a file-based InterfaceMessage from the header and
    /// content. Specifically, this function returns one of:
    /// - CSVInputFile
    /// - XMLInputFile
    /// - OutputFile
    /// - Other
    /// depending on the header.
    pub fn file_message_from_header(header: &str, content: PathBuf) -> InterfaceMessage {
        match header {
            "CSVInputFile" => InterfaceMessage::CSVInputFile(content),
            "XMLInputFile" => InterfaceMessage::XMLInputFile(content),
            "OutputFile" => InterfaceMessage::OutputFile(content),
            _ => InterfaceMessage::Other(content.to_string_lossy().into_owned()),
        }//end matching header to type.
    }//end file_message_from_header(header,content)
}//end InterfaceMessage

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
    msg_sender: Sender<InterfaceMessage>,
    /// Message Receiver, we give a reference to this to main, 
    /// allowing it to receive our messages.
    msg_receiver: Receiver<InterfaceMessage>,
    /// Buffer holding text in the header display
    ux_header_buf: TextBuffer,
    /// The group holding all the configuration controls.  
    /// This is stored here in order to disable during dialog.
    ux_config_group: Group,
    /// The group holding all the input and output controls.
    /// This is stored here in order to disable during dialog
    ux_io_controls_group: Group,
    /// The group holding the custom dialog controls.
    /// This is stored here to enable during dialog
    ux_dialog_group: Group,
    /// The display which shows dialog messages to the user.
    ux_dialog_box: TextDisplay,
    /// The flex which holds buttons corresponding to the
    /// dialog choices available to a user.
    ux_dialog_btns_flx: Flex,
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
    /// The frame holding the text displayed to indicate
    /// which named setting preset is currently active.
    ux_cf_setting_preset_buf: Frame,
    /// Stores the last config_store we've got.  
    /// It is initialized as ConfigStore::default().  
    /// It should be noted that this field is not updated automatically
    /// by changes to the GUI; instead, the primary purpose of always
    /// storing this as a field is to ensure that information contained
    /// in the config_store sent by main will be preserved, even if some
    /// of that information is not directly represented by a widget.
    config_store: Rc<RefCell<ConfigStore>>,
}//end struct GUI

#[allow(dead_code)]
impl GUI {
    /// Returns a clone of the receiver so you can
    /// react to messages sent by gui.
    pub fn get_receiver(&self) -> Receiver<InterfaceMessage> {
        return self.msg_receiver.clone();
    }//end get_receiver(self)

    /// Constructs the String holding default header information, aside from
    /// config preset information. This function builds in the pkg version,
    /// the time at which the program was compiled, and some other common
    /// header information.
    /// 
    /// This function was originally created in order to make it easier to
    /// add or remove text from the header in response to configuration preset
    /// changes.
    fn default_header_info() -> String {
        let version = option_env!("CARGO_PKG_VERSION");
        let format_des = time::macros::format_description!("[month repr:long] [year]");
        let date = compile_time::date!();
        let date_str = date.format(format_des).unwrap_or(String::from("unknown compile time"));
        let mut output = String::new();
        output.push_str(&format!("USDA C-Grain Sum\tv{}\t{}\n",version.unwrap_or("unknown version"),date_str));
        output.push_str("Processes CSV and XML Data from C-Grain into Sum Files\n\n");
        output.push_str(&format!("Nicholas Sixbury/Dan Brabec\tUSDA Manhattan,KS\n"));
        return output;
    }//end default_header_info()

    /// Closes the application.
    pub fn quit() {
        app::App::default().quit();
    }//end show(self)

    /// Wraps app.wait().  
    /// To run main app loop, use while(gui.wait()){}.
    pub fn wait(&self) -> bool {
        self.app.wait()
    }//end wait(&self)

    #[deprecated(since="0.3.4", note="please use integrated_dialog_message instead")]
    /// Simply displays a message to the user.
    pub fn show_message(txt: &str) {
        dialog::message_default(txt);
    }//end show_message(txt)

    #[deprecated(since="0.3.4", note="please use integrated_dialog_alert instead")]
    /// Simply displays an error message to the user.
    pub fn show_alert(txt: &str) {
        dialog::alert_default(txt);
    }//end show_alert(txt)

    // #[deprecated(since="0.3.4", note="please use integrated_dialog_yes_no instead")]
    /// Asks user a yes or no question. Returns true if
    /// user didn't close the dialog and clicked yes.
    pub fn show_yes_no_message(txt: &str) -> bool {
        match dialog::choice2_default(txt, "yes", "no", "") {
            Some(index) => index == 0,
            None => false,
        }//end matching dialog result
    }//end show_yes_no_message

    #[deprecated(since="0.3.4", note="please use integrated_dialog_message_choice instead")]
    /// Asks the user to choose between three options.  
    /// If this is successful, returns index of choice, 0, 1, or 2
    pub fn show_three_choice(txt: &str, c0: &str, c1: &str, c2: &str) -> Option<u8> {
        match dialog::choice2_default(txt, c0, c1, c2) {
            Some(index) => {
                match u8::try_from(index) {
                    Ok(val) => Some(val),
                    Err(_) => None,
                }//end matching whether we can convert properly
            },
            None => None,
        }//end matching dialog result
    }//end show_three_choice()

    /// Resets group activations to ensure user can
    /// interact with gui after dialog has eneded.
    pub fn clear_integrated_dialog(&mut self) {
        self.ux_io_controls_group.activate();
        self.ux_config_group.activate();
        self.ux_dialog_group.deactivate();
        self.ux_dialog_box.buffer().unwrap_or_else(|| TextBuffer::default()).set_text("");
        self.ux_dialog_btns_flx.clear();
        self.ux_dialog_btns_flx.redraw();
    }//end clear_integrated_dialog()

    /// Deactivates most of the gui so that user
    /// is forced to interact with dialog
    fn activate_dialog(&mut self) {
        self.ux_io_controls_group.deactivate();
        self.ux_config_group.deactivate();
        self.ux_dialog_group.activate();
    }//end activate_dialog()

    /// Creates a modal dialog message that is integrated into
    /// the main window of the application.
    pub fn integrated_dialog_message(&mut self, txt: &str) {
        self.integrated_dialog_message_choice(txt, vec!["Ok"]);
    }//end integrated_dialog_message()

    /// Creates a modal error message that is integrated into the
    /// main window of the application.
    pub fn integrated_dialog_alert(&mut self, txt: &str) {
        dialog::beep(BeepType::Error);
        self.integrated_dialog_message(txt);
    }//end integrated_dialog_alert()

    /// Creates a modal dialog message which forces the user
    /// to ask a yes or no question.
    pub fn integrated_dialog_yes_no(&mut self, txt: &str) -> bool {
        match self.integrated_dialog_message_choice(txt, vec!["yes","no"]) {
            Some(idx) => idx == 0,
            None => false,
        }//end matching whether selection was yes or no
    }//end integrated_dialog_yes_no()

    /// Creates a modal dialog message which forces the user to choose
    /// between the options specified.  
    /// The buttons for options have auto-generated sizes, so if there are too
    /// many options, or they are too wordy, text might not be readable.  
    /// If this function is passed an empty vec for options, it will immediately
    /// return None. Without any options to end dialog, the user wouldn't be able
    /// to continue.
    pub fn integrated_dialog_message_choice(&mut self, txt: &str, options: Vec<&str>) -> Option<usize> {
        self.activate_dialog();
        // input validation for options being empty
        if options.len() == 0 {return None;}
        // update text based on parameter
        let mut dialog_buffer = self.ux_dialog_box.buffer().unwrap_or_else(|| TextBuffer::default());
        dialog_buffer.set_text(txt);
        self.ux_dialog_box.set_buffer(dialog_buffer);
        // update buttons based on type
        let button_pressed_index = Rc::from(RefCell::from(None));

        self.ux_dialog_btns_flx.clear();
        for (idx, option) in options.iter().enumerate() {
            let mut button = Button::default().with_label(option);
            button.set_callback({
                let button_index_ref = (&button_pressed_index).clone();
                move |_| {
                    let mut button_index = button_index_ref.borrow_mut();
                    *button_index = Some(idx);
                }//end closure
            });
            self.ux_dialog_btns_flx.add(&button);
        }//end creating each button and handler
        self.ux_dialog_btns_flx.redraw();

        // wait for user to click a button
        let button_pressed_index_ref = (&button_pressed_index).clone();
        let mut button_index_to_return = None;
        while self.app.wait() {
            if let Ok(pushed_index) = button_pressed_index_ref.try_borrow() {
                if pushed_index.is_some() {button_index_to_return = pushed_index.clone(); break;}
            }
        }//end continuing application while we wait for button to be pressed

        self.clear_integrated_dialog();
        return button_index_to_return;
    }//end integrated_dialog_message(self, txt)

    /// Returns the text shown in the output file box.
    /// This box is meant to display the file name (without the directory)
    /// of the output file the user has chosen. 
    pub fn get_output_text(&self) -> String {
        let output_text = self.ux_output_file_txt.as_ref().borrow().buffer().unwrap_or_default().text();
        output_text
    }//end get_io_inputs(self)

    /// Clears text from io area.
    /// This includes the text boxes displaying the csv input filename,
    /// the xml input filename, and the output filename.
    pub fn clear_output_text(&mut self) {
        self.ux_input_csv_txt.borrow().buffer().unwrap_or_default().set_text("");
        self.ux_input_xml_txt.borrow().buffer().unwrap_or_default().set_text("");
        self.ux_output_file_txt.borrow().buffer().unwrap_or_default().set_text("");
    }//end clear_output_text()

    /// Creates a ConfigStore from the current config settings, as
    /// chosen by the user. 
    pub fn get_config_store(&self) -> ConfigStore {
        let config_ref = &self.config_store;
        let config_ref = config_ref.clone();
        let config_ref = config_ref.as_ref().borrow();
        let mut config_clone = config_ref.clone();
        
        let class_filter_txt = self.ux_cf_class_filter_buf.text();
        let stat_columns_txt = self.ux_cf_stat_cols_buf.text();
        // replace multi-char instance we want to split with single chars, then split on '|', ',', or '\n', as needed
        let class_filters: Vec<String> = class_filter_txt.replace(" | ", "|").replace(", ", ",").split(['|',',']).map(|el| el.to_owned()).collect();
        let stat_columns: Vec<String> = stat_columns_txt.replace(", ", ",").split([',','\n']).filter(|el| el.trim() != "").map(|el| el.to_owned()).collect();

        config_clone.csv_class_filter_enabled = self.ux_cf_class_filter_chck.is_checked();
        config_clone.csv_class_filter_filters = class_filters;
        config_clone.csv_stat_columns_enabled = self.ux_cf_stat_cols_chck.is_checked();
        config_clone.csv_stat_columns_columns = stat_columns;
        config_clone.csv_class_percent_enabled = self.ux_cf_class_perc_chck.is_checked();
        config_clone.xml_sieve_cols_enabled = self.ux_cf_xml_sieve_chck.is_checked();
        
        return config_clone;
    }//end get_config_store

    /// Updates the current configuration widgets in the interface to match
    /// the given ConfigStore.
    pub fn set_config_store(&mut self, config: &ConfigStore) {
        let config_ref = &self.config_store;
        let config_ref = config_ref.clone();
        let mut config_ref = config_ref.as_ref().borrow_mut();
        
        *config_ref = config.clone();

        self.ux_cf_class_filter_chck.set_checked(config.csv_class_filter_enabled);
        self.ux_cf_class_filter_buf.set_text(&config.csv_class_filter_filters.join(" | "));
        self.ux_cf_stat_cols_chck.set_checked(config.csv_stat_columns_enabled);
        self.ux_cf_stat_cols_buf.set_text(&config.csv_stat_columns_columns.join("\n"));
        self.ux_cf_class_perc_chck.set_checked(config.csv_class_percent_enabled);
        self.ux_cf_xml_sieve_chck.set_checked(config.xml_sieve_cols_enabled);

        match config.personalized_config_name.as_str() {
            "Scott" | "Rhett"=> {
                let new_header = GUI::default_header_info();
                self.ux_header_buf.set_text(&new_header);
                self.ux_cf_setting_preset_buf.set_label(&format!("Config Preset for {}",&config.personalized_config_name));
                // if config.personalized_config_name.eq("Scott") { self.ux_config_group.set_color(Color::from_rgb(220,239,220)) }
                // if config.personalized_config_name.eq("Rhett") { self.ux_config_group.set_color(Color::from_rgb(220,220,239)) }
            },
            _ => {
                self.ux_header_buf.set_text(&GUI::default_header_info());
                self.ux_cf_setting_preset_buf.set_label("No Named Preset Active");
                // self.ux_config_group.set_color(Color::Light1);
            },
        }//end matching personalized configuration stuff
        self.ux_config_group.redraw();
    }//end set_config_store(self, config)

    /// Gives a small visual indication that the program is doing something in the background.
    pub fn start_wait(&mut self) {
        self.ux_main_window.set_cursor(fltk::enums::Cursor::Wait);
    }//end start_wait(self)

    /// Clears the visual indication from start_wait()
    pub fn end_wait(&mut self) {
        self.ux_main_window.set_cursor(fltk::enums::Cursor::Default);
    }//end end_wait(self)

    /// Sets up all the properties and appearances of
    /// various widgets and UI settings.
    pub fn initialize() -> GUI {
        let c_grain_app = app::App::default();
        let mut main_window = window::Window::default().with_size(700, 435).with_label("USDA C-Grain Summarizer");
        main_window.end();

        let config_ref = Rc::from(RefCell::from(ConfigStore::default()));
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

        let (s, r): (Sender<InterfaceMessage>, Receiver<InterfaceMessage>) = app::channel();

        let mut tile_group = Tile::default()
            .with_pos(0, 0)
            .with_size(main_window.w(), main_window.h());
        tile_group.end();
        main_window.add(&tile_group);

        // set up header information
        let mut header_group = Group::default()
            .with_pos(0,0)
            .with_size(tile_group.w(), 90);
        header_group.end();
        tile_group.add(&header_group);

        let mut header_buf = TextBuffer::default();
        let mut header_box = TextDisplay::default()
            .with_pos(10, 10)
            .with_size(header_group.w() - 20,header_group.h() - 20);
        header_group.add_resizable(&header_box);
        header_box.set_buffer(header_buf.clone());
        header_buf.append(&GUI::default_header_info());
        header_box.set_scrollbar_align(Align::empty());

        // set up group with input and output controls, processing stuff
        let mut io_controls_group = Group::default()
            .with_pos(0, header_group.y() + header_group.h())
            .with_size(tile_group.w() / 7 * 4, tile_group.h() - header_group.h() - 125);
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
        input_csv_btn.set_frame(io_btn_frame);
        input_csv_btn.set_tooltip("Left Click this button to choose a csv input file.\nRight Click this button to configure advanced csv input options.");
        io_controls_group.add(&input_csv_btn);

        let input_csv_buf = TextBuffer::default();
        let mut input_csv_box = TextEditor::default()
            .with_pos(input_csv_btn.x() + input_csv_btn.w() + io_box_padding, input_csv_btn.y())
            .with_size(io_controls_group.w() - (input_csv_btn.w() + (3 * io_box_padding)), io_box_height);
        input_csv_box.set_frame(io_box_frame);
        input_csv_box.set_scrollbar_align(Align::Bottom);
        input_csv_box.set_scrollbar_size(7);
        input_csv_box.deactivate();
        input_csv_box.set_buffer(input_csv_buf.clone());
        io_controls_group.add_resizable(&input_csv_box);
        let input_csv_ref = Rc::from(RefCell::from(input_csv_box));

        input_csv_btn.set_callback({
            let input_csv_ref_clone = (&input_csv_ref).clone();
            let sender_clone = s.clone();
            let config_ref_clone = (&config_ref).clone();
            move |_| {
                if app::event_button() == 3 {
                    let mut config = config_ref_clone.as_ref().borrow_mut();
                    if let Some(choice) = dialog::input_default("Please enter the name of the column which indicates the sample id in the csv.\nThe default is external-sample-id", &config.csv_sample_id_header) {
                        config.csv_sample_id_header = choice;
                    }//end matching whether we got response from user
                } else {
                    let input_csv_ref = input_csv_ref_clone.as_ref().borrow();
                    if let Err(err_message) = GUI::create_io_dialog(&sender_clone, "CSVInputFile", &input_csv_ref, dialog::NativeFileChooserType::BrowseFile, dialog::NativeFileChooserOptions::UseFilterExt, "*.csv", "Please select a csv input file") {
                        println!("Encountered an error when attempting to show file dialog:\n{}", err_message);
                    }//end if we got an error
                }//end else user didn't right-click
            }//end moving for closure
        });

        let mut input_xml_btn = Button::default()
            .with_label("Select Input XML")
            .with_pos(input_csv_btn.x(), input_csv_btn.y() + input_csv_btn.h() + io_btn_padding)
            .with_size(io_btn_width, io_btn_height);
        input_xml_btn.set_frame(io_btn_frame);
        input_xml_btn.set_tooltip("Left Click this button to choose an xml input file.\nRight click this button to configure advanced xml input options.");
        io_controls_group.add(&input_xml_btn);

        let input_xml_buf = TextBuffer::default();
        let mut input_xml_box = TextEditor::default()
            .with_pos(input_xml_btn.x() + input_xml_btn.w() + io_box_padding, input_xml_btn.y())
            .with_size(io_controls_group.w() - (input_xml_btn.w() + (3 * io_box_padding)), io_box_height);
        input_xml_box.set_frame(io_box_frame);
        input_xml_box.set_scrollbar_align(Align::Bottom);
        input_xml_box.set_scrollbar_size(7);
        input_xml_box.deactivate();
        input_xml_box.set_buffer(input_xml_buf.clone());
        io_controls_group.add_resizable(&input_xml_box);
        let input_xml_ref = Rc::from(RefCell::from(input_xml_box));

        input_xml_btn.set_callback({
            let input_xml_ref_clone = input_xml_ref.clone();
            let sender_clone = s.clone();
            let config_clone = (&config_ref).clone();
            move |_| {
                if app::event_button() == 3 {
                    let clicked_ok = Rc::from(RefCell::from(false));
                    let config = {config_clone.borrow().clone()};
                    // create a basic window in order to show custom dialog
                    // need sample id, custom tags, and closing tag
                    let mut dialog_window = Window::default()
                        .with_size(470,160)
                        .with_label("Advanced XML Options");
                    dialog_window.make_resizable(true);
                    dialog_window.make_modal(true);
                    let mut ok_button = Button::default()
                        .with_size(50,30)
                        .with_pos(60,115)
                        .with_label("Ok");
                    ok_button.set_frame(FrameType::GtkRoundUpFrame);
                    ok_button.clear_visible_focus();
                    let mut cancel_button = Button::default()
                        .with_size(70,30)
                        .with_pos(120,115)
                        .with_label("Cancel");
                    cancel_button.set_frame(FrameType::GtkRoundUpFrame);
                    cancel_button.clear_visible_focus();
                    let mut xml_sample_id_header_buf = TextBuffer::default();
                    xml_sample_id_header_buf.set_text(&config.xml_sample_id_header);
                    let mut xml_sample_id_header_box = TextEditor::default()
                        .with_size(220,30)
                        .with_pos(20,20)
                        .with_label("Tag to read as sample-id in xml:")
                        .with_align(Align::TopLeft);
                    xml_sample_id_header_box.set_tooltip("Default is \"reference\" without quotation marks.");
                    xml_sample_id_header_box.set_frame(FrameType::GtkDownFrame);
                    xml_sample_id_header_box.set_scrollbar_align(Align::Bottom);
                    xml_sample_id_header_box.set_scrollbar_size(7);
                    xml_sample_id_header_box.set_buffer(xml_sample_id_header_buf);
                    let mut xml_closing_tag_buf = TextBuffer::default();
                    xml_closing_tag_buf.set_text(&config.xml_sample_closing_tag);
                    let mut xml_closing_tag_box = TextEditor::default()
                        .with_size(220,30)
                        .with_pos(20,70)
                        .with_label("Tag to read as the end of a sample:")
                        .with_align(Align::TopLeft);
                    xml_closing_tag_box.set_tooltip("Default is \"sample-result\" without quotation marks.");
                    xml_closing_tag_box.set_frame(FrameType::GtkDownFrame);
                    xml_closing_tag_box.set_scrollbar_align(Align::Bottom);
                    xml_closing_tag_box.set_scrollbar_size(7);
                    xml_closing_tag_box.set_buffer(xml_closing_tag_buf);
                    let mut xml_extra_tags_buf = TextBuffer::default();
                    xml_extra_tags_buf.set_text(&config.xml_tags_to_include.join("\n"));
                    let mut xml_extra_tags_box = TextEditor::default()
                        .with_size(200,130)
                        .with_pos(250,20)
                        .with_label("Extra Tags to Read from XML:")
                        .with_align(Align::TopRight);
                    xml_extra_tags_box.set_tooltip("Separate tags by newlines.\nExample of a tag is \"good-images\", without quotation marks.\nDefault is empty.");
                    xml_extra_tags_box.set_frame(FrameType::GtkDownFrame);
                    xml_extra_tags_box.set_scrollbar_align(Align::Right);
                    xml_extra_tags_box.set_scrollbar_size(12);
                    xml_extra_tags_box.set_buffer(xml_extra_tags_buf);

                    dialog_window.end();

                    dialog_window.set_callback({
                        let clicked_ref = (&clicked_ok).clone();
                        let config_clone = (&config_clone).clone();
                        move |win| {
                            let clicked_ok = clicked_ref.borrow();
                            if *clicked_ok {
                                let mut config = config_clone.borrow_mut();
                                config.xml_sample_id_header = xml_sample_id_header_box.buffer().unwrap().text();
                                config.xml_sample_closing_tag = xml_closing_tag_box.buffer().unwrap().text();
                                config.xml_tags_to_include = xml_extra_tags_box
                                    .buffer().unwrap().text()
                                    .split("\n").into_iter().filter(|el| el.trim() != "")
                                    .map(|el| el.to_owned()).collect();
                                dialog::message_title("Success!");
                                dialog::message_default("Advanced XML Options have been successfully updated.");
                            }//end if user clicked ok to change their config
                            win.hide();
                        }//end closure
                    });

                    let dialog_window_ref = Rc::from(RefCell::from(dialog_window));
                    ok_button.set_callback({
                        let window_ref = (&dialog_window_ref).clone();
                        let ok_ref = (&clicked_ok).clone();
                        move |_| {
                            let mut window_ref = window_ref.borrow_mut();
                            *(ok_ref.borrow_mut()) = true;
                            window_ref.do_callback();
                        }//end moving closure
                    });
                    cancel_button.set_callback({
                        let window_ref = (&dialog_window_ref).clone();
                        move |_| {
                            let mut window_ref = window_ref.borrow_mut();
                            window_ref.do_callback();
                        }//end moving closure
                    });

                    let window_ref_clone = (&dialog_window_ref).clone();
                    let mut window_ref = window_ref_clone.borrow_mut();
                    window_ref.show();
                } else {
                    let input_xml_ref = input_xml_ref_clone.as_ref().borrow();
                    if let Err(err_message) = GUI::create_io_dialog(&sender_clone, "XMLInputFile", &input_xml_ref, dialog::NativeFileChooserType::BrowseFile, dialog::NativeFileChooserOptions::UseFilterExt, "*.xml", "Please select an xml input file") {
                        println!("Encountered an error when attempting to show file dialog:\n{}", err_message);
                    }//end if we got an error
                }//end else user didn't right-click
            }//end moving for closure
        });

        // get output file from user
        let mut output_file_btn = Button::default()
            .with_label("Select Output XLSX")
            .with_pos(input_xml_btn.x(), input_xml_btn.y() + input_xml_btn.h() + io_btn_padding)
            .with_size(io_btn_width, io_btn_height);
        output_file_btn.set_frame(io_btn_frame);
        output_file_btn.set_tooltip("Click this button to set where the output file will be located.\nOr, just type a name in the box to right.");
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

        output_file_btn.set_callback({
            let output_file_ref_clone = output_file_ref.clone();
            let sender_clone = s.clone();
            move |_| {
                let output_file_ref = output_file_ref_clone.as_ref().borrow();
                if let Err(err_message) = GUI::create_io_dialog(&sender_clone, "OutputFile", &output_file_ref, dialog::NativeFileChooserType::BrowseSaveFile, dialog::NativeFileChooserOptions::SaveAsConfirm, "", "Please specify the output file.") {
                    println!("Encountered an error when attempting to show file dialog:\n{}", err_message);
                }//end if we got an error
            }//end moving for closure
        });

        // process the data we have
        let mut process_file_btn = Button::default()
            .with_label("Process Data")
            .with_pos(output_file_btn.x() + 60, output_file_btn.y() + output_file_btn.h() + 10)
            .with_size(250, 50);
        process_file_btn.emit(s.clone(), InterfaceMessage::ProcessSum);
        process_file_btn.set_frame(FrameType::PlasticDownBox);
        io_controls_group.add_resizable(&process_file_btn);

        // set up group with configuration options
        let mut config_group = Group::default()
            .with_pos(io_controls_group.x() + io_controls_group.w(), io_controls_group.y())
            .with_size(tile_group.width() - io_controls_group.width(), tile_group.height() - header_group.height());
        config_group.end();
        config_group.set_color(Color::from_rgb(220,239,220));
        tile_group.add(&config_group);
        
        let mut config_label = Frame::default()
            .with_pos(config_group.x(), config_group.y() + 10)
            .with_size(config_group.width(), 20)
            .with_label("Configuration Settings")
            .with_align(Align::Center);
        config_group.add(&config_label);
        
        config_label.set_tooltip("Right click if you want to change config presets.");
        config_label.handle({
            let sender_clone = s.clone();
            move |_, ev| {
                match ev {
                    Event::Released => {
                        // event_button => 1 for left click, 2 for middle, 3 for right
                        if app::event_button() == 3 {
                            if GUI::show_yes_no_message("Would you like to reset the current configuration preset?") {
                                sender_clone.send(InterfaceMessage::ConfigReset);
                            }//end if we want to reset the current config preset
                        }//end if we have a right-click event
                        true
                    },
                    _ => false
                }
            }//end moving for closure
        });

        let config_preset_frm = Frame::default()
            .with_pos(config_label.x(), config_label.y() + config_label.h())
            .with_size(config_label.w(),config_label.h())
            .with_label("No Named Preset Active")
            .with_align(Align::Center);
        config_group.add(&config_preset_frm);

        let mut class_filter_chck = CheckButton::default()
            .with_pos(config_preset_frm.x() + cf_padding, config_preset_frm.y() + config_preset_frm.h() + cf_padding)
            .with_size(180,cf_chck_height)
            .with_label("Filter to Classification of:");
        class_filter_chck.set_checked(true);
        class_filter_chck.set_frame(cf_chck_frame);
        class_filter_chck.set_tooltip("If checked, processing will only consider rows in csv data matching the given classification(s).\nRight click if you want to configure which column is considered for class filtering.");
        config_group.add(&class_filter_chck);
        class_filter_chck.set_callback({
            let config_ref_clone = (&config_ref).clone();
            move |chck| {
                if app::event_button() == 3 {
                    // this is just done to cancel the toggle of checked
                    chck.set_checked(!chck.is_checked());
                    let mut config = config_ref_clone.as_ref().borrow_mut();
                    if let Some(choice) = dialog::input_default("Please indicate the name of the column holding class\ninformation, to be used in filtering.\nThe default is raw-filtered-as", &config.csv_class_filter_class) {
                        config.csv_class_filter_class = choice;
                    }//end if user chose to change setting
                }//end if user right-clicked
            }//end moving closure
        });

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
            .with_size(stat_cols_chck.w(), 184);
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

        let mut dialog_group = Group::default()
            .with_pos(io_controls_group.x(), io_controls_group.y() + io_controls_group.h())
            .with_size(io_controls_group.w(), tile_group.h() - (io_controls_group.y() + io_controls_group.h()));
        dialog_group.end();
        tile_group.add(&dialog_group);

        let mut dialog_buf = TextBuffer::default();
        let mut dialog_box = TextDisplay::default()
            .with_pos(dialog_group.x() + 5, dialog_group.y() + 5)
            .with_size(dialog_group.w() - 10, dialog_group.height() - 50)
            .with_align(Align::Inside);
        dialog_box.set_color(Color::Light1);
        dialog_box.set_frame(FrameType::GtkThinDownFrame);
        dialog_box.wrap_mode(fltk::text::WrapMode::AtBounds, 1);
        dialog_box.set_scrollbar_align(Align::Right);
        dialog_box.set_scrollbar_size(10);
        dialog_buf.set_text("");
        dialog_box.set_buffer(dialog_buf);
        dialog_group.add(&dialog_box);

        let mut dialog_btns = Flex::default()
            .with_pos(dialog_box.x(), dialog_box.y() + dialog_box.h() + 5)
            .with_size(dialog_box.w(), dialog_group.h() - dialog_box.h() - 15)
            // .with_label("button_pack")
            .with_align(Align::Right)
            .with_type(FlexType::Row);
        dialog_btns.end();
        dialog_btns.set_frame(FrameType::FlatBox);
        dialog_group.add(&dialog_btns);

        // set frame type for borders between sections, make sure to use box type
        header_group.set_frame(FrameType::GtkUpBox);
        io_controls_group.set_frame(FrameType::GtkUpBox);
        config_group.set_frame(FrameType::GtkUpBox);
        dialog_group.set_frame(FrameType::GtkUpBox);
        dialog_group.deactivate();

        main_window.make_resizable(true);
        // callback for window occurs when user tries to close it
        main_window.set_callback({
            let sender_clone = s.clone();
            move |_| {
                sender_clone.send(InterfaceMessage::AppClosing);
                println!("GUI Ready to Close!");
            }
        });
        main_window.show();

        GUI {
            app: c_grain_app,
            ux_main_window: main_window,
            debug_log: Vec::new(),
            msg_sender: s,
            msg_receiver: r,
            ux_header_buf: header_buf,
            ux_config_group: config_group,
            ux_io_controls_group: io_controls_group,
            ux_dialog_group: dialog_group,
            ux_dialog_box: dialog_box,
            ux_dialog_btns_flx: dialog_btns,
            ux_input_csv_txt: input_csv_ref,
            ux_input_xml_txt: input_xml_ref,
            ux_output_file_txt: output_file_ref,
            ux_cf_class_filter_chck: class_filter_chck,
            ux_cf_class_filter_buf: class_filter_buf,
            ux_cf_stat_cols_chck: stat_cols_chck,
            ux_cf_stat_cols_buf: stat_cols_buf,
            ux_cf_class_perc_chck: class_perc_chck,
            ux_cf_xml_sieve_chck: xml_sieve_chck,
            ux_cf_setting_preset_buf: config_preset_frm,
            config_store: config_ref,
        }//end struct construction
    }

    /// Helper method used in initialize to share code between handlers
    /// of io buttons.
    fn create_io_dialog(sender: &Sender<InterfaceMessage>, msg_header: &str, txt: &TextEditor, dialog_type: dialog::NativeFileChooserType, dialog_option: dialog::NativeFileChooserOptions, dialog_filter: &str, dialog_title: &str ) -> Result<(), String> {
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
        match dialog.filename().file_name() {
            Some(filename) => {
                txt_buf.set_text(filename.to_string_lossy().as_ref());
                sender.send(InterfaceMessage::file_message_from_header(msg_header,dialog.filename()));
            },
            None => return Err(format!("Couldn't get filename for some reason"))
        }//end matching whether we can get the filename to set the box

        return Ok(());
    }//end create_io_dialog()
}//end impl for GUI
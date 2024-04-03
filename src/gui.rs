use fltk::{app::{self, App, Receiver, Sender}, button::{self, Button}, frame::Frame, prelude::{DisplayExt, GroupExt, WidgetBase, WidgetExt}, text, window::{self, Window}};

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
        let gui_app = App::default();
        let (s,r) = app::channel();
        
        // set up main window properties
        let mut ux_main_window = Window::default()
            .with_size(900,480)
            .with_label("USDA C-Grain Summarizer");
        ux_main_window.make_resizable(true);
        ux_main_window.end();

        let ux_test_frame = Frame::default()
            .with_pos(50,50)
            .with_size(100,100);
        ux_main_window.add(&ux_test_frame);

        let ux_text_button = Button::default()
            .with_size(50,50)
            .with_pos(50,50)
            .with_label("test button");
        ux_main_window.add(&ux_text_button);

        // self.msg_sender.send("test".to_string());
        // println!("Sent test");

        ux_main_window.show();
        let _ = gui_app.run();

        GUI {
            app: gui_app,
            ux_main_window,
            msg_sender: s,
            msg_receiver: r,
        }
    }//end initialize(self)

    pub fn test() {
        let flight_mill_log_processor = app::App::default();
        let mut main_window = window::Window::default().with_size(900, 480).with_label("Rusty Flight Mill");

        // set up header information
        let mut header_buf = text::TextBuffer::default();
        let mut header_box = text::TextDisplay::default().with_pos(10, 10).with_size(880, 140);
        header_box.set_buffer(header_buf.clone());

        header_buf.append("Flight Mill Compression v.2.22\n");
        header_buf.append("USDA-ARS Manhattan, KS\tMar/2023\tSixbury/Rust/Brabec\n");

        let mut thresh_prompt_buf = text::TextBuffer::default();
        let mut prompt_threshold = text::TextDisplay::new(10,260,300,40,"");
        prompt_threshold.set_buffer(thresh_prompt_buf.clone());
        thresh_prompt_buf.set_text("Please enter threshold. Default: 1.5");

        let mut thresh_input_buf = text::TextBuffer::default();
        let mut txt_thresh_input = text::TextEditor::new(320, 260, 570, 40, "");
        txt_thresh_input.set_buffer(thresh_input_buf.clone());
        thresh_input_buf.append("1.5");

        // set up additional options
        let btn_skip_lines = button::CheckButton::new(10, 310, 400, 40, "Skip a number of lines at start of Input File");
        let mut skip_line_buf = text::TextBuffer::default();
        let mut txt_skip_line_num = text::TextEditor::new(410,310,480,40,"");
        txt_skip_line_num.set_buffer(skip_line_buf.clone());
        skip_line_buf.append("4");
        btn_skip_lines.set_checked(true);

        main_window.end();
        main_window.show();

        while flight_mill_log_processor.wait() {
            
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
}//end impl for GUI
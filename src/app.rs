use self::code_editor::CodeEditor;
use dirs::home_dir;
use egui::{Color32, RichText, TextBuffer, Vec2};
use rfd::FileDialog;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use windows_sys::w;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_F, VK_F11, VK_N, VK_O, VK_R, VK_RMENU, VK_S, VK_T,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_ICONEXCLAMATION, MB_OK, MB_YESNOCANCEL,
};
//mod gks;
mod cmdmod;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// if we add new fields, give them default values when deserializing old state
mod code_editor;
mod richpresence;
use cmdmod::{
    finder, mkdir, newcmd, openfile, rmdir, runfile, savetofile, terminalr
};
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    #[serde(skip)]
    can_save : bool,
    #[serde(skip)]
    can_run : bool,
    #[serde(skip)]
    recv: mpsc::Receiver<String>,
    #[serde(skip)]
    sender: mpsc::Sender<String>,

    unsafe_mode: bool,

    #[serde(skip)]
    to_find: String,

    #[serde(skip)]
    output: String,

    #[serde(skip)]
    output_window_is_open: bool,

    #[serde(skip)]
    spotify_window_is_open: bool,

    #[serde(skip)]
    terminal_help: bool,

    #[serde(skip)]
    window_title: String,

    #[serde(skip)]
    settings_window_is_open: bool,

    auto_save_to_ram: bool,

    auto_save: bool,

    #[serde(skip)]
    text: String,

    language: String,

    terminal_mode: bool,

    code_editor: CodeEditor,

    #[serde(skip)]
    last_save_path: Option<PathBuf>,

    auto_save_interval: u64,
    #[serde(skip)]
    rpc_sender: Option<mpsc::Sender<String>>,
    #[serde(skip)]
    autosave_sender: Option<mpsc::Sender<String>>,

    #[serde(skip)]
    session_started: chrono::DateTime<chrono::Local>,

    #[serde(skip)]
    code_editor_text_lenght: usize,

    #[serde(skip)]
    discord_presence_is_running: bool,

    #[serde(skip)]
    lines: Vec<String>,

    is_gui_development: bool,

    #[serde(skip)]
    finder_is_open: bool,

    #[serde(skip)]
    scroll_offset: Vec2,

    #[serde(skip)]
    is_found: Option<bool>,

    #[serde(skip)]
    occurences: usize,

    #[serde(skip)]
    opened_file: String,

    window_options_always_on_top: bool,
    window_options_full_screen: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let (sender, recv) = mpsc::channel::<String>();
        Self {
            can_save: false,
            can_run: false,
            recv,
            sender,
            unsafe_mode: false,
            to_find: String::new(),
            terminal_help: false,
            output: String::new(),
            output_window_is_open: false,
            spotify_window_is_open: false,
            window_title: "Marcide".into(),
            session_started: chrono::Local::now(),
            settings_window_is_open: false,
            auto_save: true,
            terminal_mode: false,
            auto_save_to_ram: false,
            text: String::new(),
            language: "py".into(),
            code_editor: CodeEditor::default(),
            is_gui_development: false,
            last_save_path: None,
            auto_save_interval: 15,
            autosave_sender: None,
            rpc_sender: None,
            code_editor_text_lenght: 0,
            discord_presence_is_running: false,
            lines: Vec::new(),
            finder_is_open: false,
            scroll_offset: (0.0, 0.0).into(),
            is_found: None,
            occurences: 0,
            opened_file: String::new(),

            window_options_always_on_top: false,
            window_options_full_screen: false,
        }
    }
}
impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}
fn count_lines(text: &str) -> usize {
    text.split('\n').count()
}
impl eframe::App for TemplateApp {
    fn on_close_event(&mut self) -> bool {
        //remove temp dir
        rmdir();
        let window_on_top_state = self.window_options_always_on_top;
        if window_on_top_state {
            self.window_options_always_on_top = false;
        }
        if !self.auto_save || self.last_save_path.is_none() {
            //implement save warnings using winapi
            unsafe {
                match MessageBoxW(
                    0,
                    w!("Do you want to save before qutting the application?"),
                    w!("Save before quitting"),
                    MB_ICONERROR | MB_YESNOCANCEL,
                ) {
                    //yes
                    6 => {
                        //save text then quit
                        let files = FileDialog::new()
                            .set_title("Save as")
                            .set_directory("/")
                            .save_file();
                        if files.clone().is_some() {
                            self.last_save_path = files.clone();
                            savetofile(files.clone(), self.text.clone());
                            return true;
                        } else {
                            self.on_close_event();
                        }
                    }
                    //no
                    7 => {
                        //quit
                        return true;
                    }
                    //cancel
                    2 => {
                        if window_on_top_state {
                            self.window_options_always_on_top = true;
                        }
                        return false;
                    }
                    _ => {}
                };
            }
        }
        true
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if !self.auto_save_to_ram {
            self.code_editor.code.clear();
        }
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.code_editor_text_lenght > self.code_editor.code.len() {
            self.code_editor_text_lenght = self.code_editor.code.len();
        }
        let mut go_to_offset: bool = false;
        match self.recv.try_recv() {
            Ok(ok) => {
                self.output = ok.clone();
            }
            Err(_) => { /*Task didnt finsih yet*/ }
        };
        let _projname: String = self.opened_file.clone();
        let starttime: String = self.session_started.format("%m-%d %H:%M:%S").to_string();
        let tx = self.rpc_sender.get_or_insert_with(|| {
            let (tx, rx) = mpsc::channel::<String>();
            std::thread::spawn(move || loop {
                //reciver, text always gets updated
                match rx.try_recv() {
                    Ok(_ /* Failed attempt to make a changing rcp based on filename */) => {
                        match richpresence::rpc(starttime.clone()) {
                            Err(err) => println!("Richpresence failed : {}", err),
                            Ok(_) => {}
                        };
                    }
                    Err(_) => {
                        
                    }
                };
            });
            tx
        });
        match tx.send(self.opened_file.clone()) {
            Ok(_) => {}
            Err(err) => {
                println!("Failed to send msg : {}", err)
            }
        };

        self.discord_presence_is_running = true;

        //title logic for * when unsaved

        //hotkeys
        if let Some(wintitle) = self.last_save_path.clone() {
            if let Some(file_name) = wintitle.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    self.window_title = format!("Marcide - {}", file_name_str.to_string());
                    self.opened_file = file_name_str.clone().to_string();
                }
            }
            if self.code_editor_text_lenght != self.code_editor.code.len() {
                self.window_title = self.window_title.clone() + &"*".as_str();
            }
        }
        //frame settings
        _frame.set_window_title(self.window_title.as_str());
        _frame.set_fullscreen(self.window_options_full_screen);
        _frame.set_always_on_top(self.window_options_always_on_top);
        //get alt input => if true ctrl => false
        let altimput = unsafe { GetAsyncKeyState(VK_RMENU as i32) };
        let alt_is_pressed = (altimput as u16 & 0x8000) != 0;
        let nimput = unsafe { GetAsyncKeyState(VK_N as i32) };
        let nis_pressed = (nimput as u16 & 0x8000) != 0;
        let oimput = unsafe { GetAsyncKeyState(VK_O as i32) };
        let ois_pressed = (oimput as u16 & 0x8000) != 0;
        let rimput = unsafe { GetAsyncKeyState(VK_R as i32) };
        let ris_pressed = (rimput as u16 & 0x8000) != 0;
        let timput = unsafe { GetAsyncKeyState(VK_T as i32) };
        let tis_pressed = (timput as u16 & 0x8000) != 0;
        let has_focus = ctx.input(|i| i.focused);
        let ctrlimput = unsafe { GetAsyncKeyState(VK_CONTROL as i32) };
        let mut ctrlis_pressed = (ctrlimput as u16 & 0x8000) != 0;
        let f11input = unsafe { (GetAsyncKeyState(VK_F11 as i32) as u16 & 0x8000) != 0 };
        //listen if ENTER key is pressed so we can send the message, except when r or l shift is pressed
        let sinp = unsafe { GetAsyncKeyState(VK_S as i32) };
        let sis_pressed = (sinp as u16 & 0x8000) != 0;
        //if f is pressed
        let fimput = unsafe { GetAsyncKeyState(VK_F as i32) };
        let fis_pressed = (fimput as u16 & 0x8000) != 0;
        //save hotkey
        if f11input {
            self.window_options_full_screen = !self.window_options_full_screen;
        }
        if alt_is_pressed {
            ctrlis_pressed = false;
        }
        if sis_pressed && ctrlis_pressed && has_focus {
            self.can_save = !self.can_save;
        }
        //finder hotkey
        if ctrlis_pressed && fis_pressed && has_focus {
            if !self.finder_is_open {
                self.finder_is_open = true;
            }
        }
        if ctrlis_pressed && ris_pressed && has_focus {
            if !self.output_window_is_open {
                self.can_run = !self.can_run
            }
        }
        if ctrlis_pressed && fis_pressed && has_focus {
            if !self.finder_is_open {
                self.finder_is_open = !self.finder_is_open;
            }
        }
        if ctrlis_pressed && ois_pressed && has_focus {
            let files = FileDialog::new()
                .set_title("Open")
                .set_directory("/")
                .pick_file();
            if files.clone().is_some() {
                self.last_save_path = files.clone();
                self.code_editor.code = openfile(files);
                self.code_editor_text_lenght = self.code_editor.code.len();
            }
        }
        if ctrlis_pressed && tis_pressed && has_focus {
            if !self.settings_window_is_open {
                self.settings_window_is_open = !self.settings_window_is_open;
            }
        }
        if ctrlis_pressed && nis_pressed && has_focus {
            let files = FileDialog::new()
                .set_title("Save as")
                .set_directory("/")
                .save_file();
            if files.clone().is_some() {
                self.last_save_path = files.clone();
                savetofile(files.clone(), self.text.clone());
                self.code_editor_text_lenght = self.code_editor.code.len();
            }
        }
        self.text = self.code_editor.code.clone();
        self.code_editor.language = self.language.clone();
        if self.output_window_is_open && !self.is_gui_development {
            egui::Window::new("Output")
                .open(&mut self.output_window_is_open)
                .show(ctx, |ui| {
                    ui.label(
                        RichText::from("Output")
                            .size(25.0)
                            .color(Color32::from_rgb(92, 92, 92)),
                    );
                    ui.label(
                        RichText::from(self.output.clone())
                            .size(20.0)
                            .color(Color32::from_rgb(255, 255, 255)),
                    );
                });
        }
        if self.finder_is_open {
            egui::Window::new("Finder")
                .open(&mut self.finder_is_open)
                .show(ctx, |ui| {
                    let occurence: usize;

                    ui.label("Finder");
                    ui.text_edit_singleline(&mut self.to_find);
                    if ui.button("Search").clicked() {
                        let occur: io::Result<Vec<usize>> =
                            finder(self.code_editor.code.clone(), self.to_find.clone());
                        if occur.as_ref().unwrap().len() == 0 {
                            self.is_found = None;
                        } else {
                            go_to_offset = true;
                            //let px = ui.fonts(|f| f.row_height(&egui::FontId { size: 10.0, family: egui::FontFamily::Monospace }));
                            occurence = occur.as_ref().unwrap()[0];
                            self.scroll_offset[1] = occurence as f32;
                            self.occurences = occur.unwrap().len();
                            self.is_found = Some(true);
                        }
                        //update scroll offset and done!
                    }
                    if self.is_found.is_none() {
                        ui.colored_label(Color32::RED, "0 Occurences in the text");
                    } else {
                        ui.colored_label(
                            Color32::GREEN,
                            format!("Appears in {} line(s)", self.occurences),
                        );
                    }
                });
        }
        /*
        if self.spotify_window_is_open{
            egui::Window::new("Spotify")
            .open(&mut self.spotify_window_is_open)
            .show(ctx, |ui|{

            });
        }
        */
        if self.terminal_help {
            egui::Window::new("Help")
                .default_size((300.0, 300.0))
                .open(&mut self.terminal_help)
                .show(ctx, |ui| {
                    ui.label("How to add a module to python?");
                    ui.text_edit_singleline(&mut "py -m pip install {your_module_name}");
                });
        }
        //autosave implementation
        if self.last_save_path.is_some() {
            //define recv sender

            let tx = self.autosave_sender.get_or_insert_with(|| {
                let (tx, rx) = mpsc::channel::<String>();

                std::thread::spawn(move || loop {
                    match rx.try_recv() {
                        Ok(text) => {
                            //println!("{}", lines[1]);
                            let lines: Vec<&str> =
                                text.split("VxpAM$616*9Y8G%tOp$en*KDJ").collect();

                            savetofile(Some(PathBuf::from(lines[1].trim())), lines[0].to_string());
                        }
                        Err(_) => {
                            //"SzeRinTetEk tuDja A hArmAdiK szAbÁLyT?"
                        }
                    };
                });
                tx
            });
            if self.auto_save {
                if let Some(path) = self.last_save_path.clone() {
                    let data_to_send: String = format!(
                        "{} VxpAM$616*9Y8G%tOp$en*KDJ {}",
                        self.code_editor.code.clone(),
                        path.to_str().unwrap_or_default().to_string()
                    );
                    if self.code_editor_text_lenght < self.code_editor.code.len() {
                        match tx.send(data_to_send) {
                            Ok(_) => {}
                            Err(_) => {}
                        };
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
                }
            }
        }
        if self.settings_window_is_open {
            egui::Window::new("Settings")
                .open(&mut self.settings_window_is_open)
                .show(ctx, |ui| {
                    ui.label(RichText::from("Window").size(20.0));
                    ui.checkbox(&mut self.window_options_always_on_top, "Always on top");
                    ui.label(egui::RichText::from("File handling").size(20.0));
                    ui.checkbox(&mut self.auto_save, "Autosave to file");
                    if self.auto_save {
                        ui.label(
                            "Autosave will only turn on after you have saved your file somewhere.",
                        );
                    }
                    //this will enable/disable the savestate feature to codeeditor.code
                    ui.checkbox(&mut self.auto_save_to_ram, "Autosave");
                    if self.auto_save_to_ram {
                        ui.label("This will save your text temporarily in the application. ");
                    }
                    ui.separator();
                    ui.label(egui::RichText::from("Programming language").size(20.0));
                    ui.text_edit_singleline(&mut self.language);
                    if self.language == "quran"
                        || self.language == "Quran"
                        || self.language == "Korán"
                        || self.language == "korán"
                    {
                        ui.hyperlink_to("Quran", "https://mek.oszk.hu/06500/06534/06534.pdf");
                    }
                    if self.language == "marci1175"
                        || self.language == "marci"
                        || self.language == "Marci"
                        || self.language == "Marcell"
                        || self.language == "Varga Marcell"
                    {
                        ui.separator();
                        ui.label("Credits");
                        ui.separator();
                        ui.label(
                            "Made by : Varga Marcell also known as marci1175 at 5111 days old",
                        );
                        ui.label("Had so much fun developing this lol.");
                        ui.hyperlink_to("Github", "https://github.com/marci1175");
                    }
                    if self.language == "py" || self.language == "lua" {
                        ui.checkbox(&mut self.is_gui_development, "Gui mode");
                        if self.is_gui_development {
                            ui.label(RichText::from("Output window wont show up when running the application, with gui mode.").color(Color32::LIGHT_YELLOW));
                            self.output_window_is_open = false;
                        }
                    }
                    else {
                        self.is_gui_development = false;
                    }
                    if self.language == "bat" || self.language == "cmd" || self.language == "ps1" || self.language == "vbs" || self.language == "wsf" || self.language == "reg" {
                        self.terminal_mode = true;
                        ui.label(RichText::from("Terminal mode is on, but syntaxing is unavailable").color(Color32::LIGHT_YELLOW));
                    }
                    else if !self.unsafe_mode {
                        self.terminal_mode = false;
                    }
                    ui.checkbox(&mut self.unsafe_mode, "Unsafe mode");
                    if self.unsafe_mode {
                        ui.label(
                            "With unsafe mode on you can try running any programming language added to PATH",
                        );
                        ui.checkbox(&mut self.terminal_mode, "Terminal mode");
                    }
                    if self.terminal_mode {
                        ui.label("You can use marcide to excecute terminal commands");
                    }
                });
        }
        egui::TopBottomPanel::top("Settings").show(ctx, |ui| {
            /*Brownie recipie
                For 1 small baking dish 1 cup (2 sticks) butter 4 medium sized eggs 2 cups
                brown sugar 3/4 cup cocoa powder (you can substitute 3.5 oz really dark chocolate) 1 cup flour 1/2 teaspoon vanilla extract
                3/4 cup chopped almonds or other nuts

                1. Melt the butter, let it cool a little
                2. Beat eggs, sugar into butter
                3. Mix in the rest of the ingredients
                4. Put into baking dish
                5. Bake ~30 minutes at 375 F
            */
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                //define buttons
                let run = ui.button("Run");
                let find = ui.button("Find");
                let save = ui.button("Save");
                let save_as = ui.button("Save as");
                let open = ui.button("Open");
                let terminal = ui.button("Terminal");
                let settings = ui.button("Settings");
                let support = ui.button("Support");
                if run.clicked()  || self.can_run {
                    if self.terminal_mode
                        || self.unsafe_mode
                        || self.language == "py"
                        || self.language == "lua"
                    {
                        //reset value
                        self.can_run = false;
                        //save to temp folder
                        if self.last_save_path.is_none() {
                            mkdir();
                            //C:\Users\%user_name%\marcide.temp
                            if let Some(mut home_dir) = home_dir() {
                                let to_push = format!("%marcide.temp%\\temp.{}", self.language);
                                home_dir.push(to_push);

                                // Set the files variable
                                let files: Option<PathBuf> = Some(home_dir);
                                //save file
                                savetofile(files.clone(), self.text.clone());
                                //run file
                                self.output_window_is_open = true;
                                let lang = self.language.clone();
                                let s = self.sender.clone();
                                let terminalm = self.terminal_mode.clone();
                                std::thread::spawn(move || {
                                    let mut _out: String = String::new();
                                    if !terminalm {
                                        _out = String::from_utf8_lossy(
                                            &runfile(files.clone(), lang).stdout,
                                        )
                                        .to_string();
                                    } else {
                                        _out = String::from_utf8_lossy(
                                            &terminalr(files.clone()).stdout,
                                        )
                                        .to_string();
                                    }

                                    s.send(_out.clone()).expect("Couldnt send msg");
                                });
                            }
                        } else {
                            let files = self.last_save_path.clone();
                            self.output_window_is_open = true;
                            let lang = self.language.clone();
                            let terminalm = self.terminal_mode.clone();
                            let s = self.sender.clone();
                            std::thread::spawn(move || {
                                let mut _out: String = String::new();
                                if !terminalm {
                                    _out = String::from_utf8_lossy(
                                        &runfile(files.clone(), lang).stdout,
                                    )
                                    .to_string();
                                } else {
                                    _out =
                                        String::from_utf8_lossy(&terminalr(files.clone()).stdout)
                                            .to_string();
                                }

                                s.send(_out.clone()).expect("Couldnt send msg");
                            });
                        }
                    } else {
                        unsafe {
                            MessageBoxW(
                                0,
                                w!("This ide can only run .lua, and .py files out of box"),
                                w!("Fatal error"),
                                MB_ICONEXCLAMATION | MB_OK,
                            );
                        }
                    }
                }
                if open.clicked() {
                    let files = FileDialog::new()
                        .set_title("Open")
                        .set_directory("/")
                        .pick_file();
                    if files.clone().is_some() {
                        self.last_save_path = files.clone();
                        self.code_editor.code = openfile(files);
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
                }
                if save_as.clicked() {
                    let files = FileDialog::new()
                        .set_title("Save as")
                        .set_directory("/")
                        .save_file();
                    if files.clone().is_some() {
                        self.last_save_path = files.clone();
                        savetofile(files.clone(), self.text.clone());
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
                };
                if save.clicked() || self.can_save {
                    //reset value
                    self.can_save = false;
                    if self.last_save_path.clone().is_none() {
                        let files = FileDialog::new()
                            .set_title("Save as")
                            .set_directory("/")
                            .save_file();
                        self.last_save_path = files.clone();
                        savetofile(self.last_save_path.clone(), self.text.clone());
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    } else if self.code_editor_text_lenght < self.code_editor.code.len() {
                        savetofile(self.last_save_path.clone(), self.text.clone());
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    } else {
                        //do nothing
                    }
                }
                if find.clicked() {
                    self.finder_is_open = !self.finder_is_open;
                }
                if settings.clicked() {
                    self.settings_window_is_open = !self.settings_window_is_open;
                }
                if terminal.clicked() {
                    newcmd();
                    self.terminal_help = !self.terminal_help;
                }
                if support.clicked() {
                    match webbrowser::open("https://discord.gg/7s3VRr4H6j") {
                        Ok(_) => {}
                        Err(_) => {}
                    };
                }
                /*
                if ui.button("Spotify").clicked(){
                    self.spotify_window_is_open = true;
                }
                */
                run.on_hover_text("You can run py and lua files\nCTRL + R");
                open.on_hover_text("CTRL + O");
                save_as.on_hover_text("CTRL + N");
                save.on_hover_text("CTRL + S");
                find.on_hover_text("CTRL + F");
                settings.on_hover_text("CTRL + T");
                support.on_hover_text("If you encounter errors make sure to contact support!");
            });
        });

        egui::TopBottomPanel::bottom("Stats").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let lenght = self.text.len();
                let lines = count_lines(&self.text);
                let final_lenght = lenght - (lines - 1);
                ui.label(lines.to_string() + " : Lines");

                //separate self.label into a vector by whitespaces, then count them
                let trimmed_string = self.text.trim();
                let words: Vec<&str> = trimmed_string.split_whitespace().collect();
                ui.label(words.len().to_string() + " : Words");
                ui.label(final_lenght.to_string() + " : Characters");
                ui.separator();
                let current_datetime = chrono::Local::now();
                let datetime_str = current_datetime.format("%H:%M:%S ").to_string();
                let sessiondate_str = self.session_started.format("%H:%M:%S ").to_string();
                ui.label(format!("Session started : {}", sessiondate_str));
                ui.label(format!("Current time : {}", datetime_str));
                ctx.request_repaint();
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    self.scroll_offset = code_editor::CodeEditor::show(
                        &mut self.code_editor,
                        "id".into(),
                        ui,
                        self.scroll_offset,
                        go_to_offset,
                    );
                },
            );
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}

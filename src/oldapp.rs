use self::code_editor::CodeEditor;
use dirs::home_dir;
use egui::{Layout, Stroke, Rounding};
use egui::{Color32, RichText, TextBuffer, Vec2};
use egui_dock::{NodeIndex, Tree, Style};
use egui_terminal::term::CommandBuilder;
use rfd::FileDialog;
use std::io;
use std::env;
use winreg::enums::*;
use winreg::RegKey;
use std::path::PathBuf;
use std::sync::mpsc;
use windows_sys::w;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_F, VK_F11, VK_N, VK_O, VK_R, VK_RMENU, VK_S, VK_T,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_ICONEXCLAMATION, MB_OK, MB_YESNOCANCEL,
};

use eframe::egui;
use egui_terminal::TermHandler;

//mod gks;
mod terminal;
mod cmdmod;
mod code_editor;
mod richpresence;

use cmdmod::{
    finder, mkdir, openfile, rmdir, runfile, savetofile, terminalr
};
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    #[serde(skip)]
    tree: Tree<usize>,
    #[serde(skip)]
    window_size: Vec2,
    #[serde(skip)]
    terminal_terminal_style: TermHandler,

    #[serde(skip)]
    run_terminal_style: TermHandler,

    #[serde(skip)]
    can_save_as : bool,
    #[serde(skip)]
    can_open : bool,
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

    #[serde(skip)]
    read_from_args: bool,
    first_run : bool,

    #[serde(skip)]
    counter: usize,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let (sender, recv) = mpsc::channel::<String>();
        Self {
            tree: Tree::new(vec![1, 2]),
            window_size: Vec2 { x: 0., y: 0. },
            terminal_terminal_style: TermHandler::new(CommandBuilder::new("powershell")),
            run_terminal_style: TermHandler::new(CommandBuilder::new("powershell")),
            can_open: false,
            can_save_as: false,
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

            read_from_args: true,
            first_run : true,

            counter: 0,
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
fn trueorfalse(arg : String) -> bool {
    if arg.to_lowercase() == "false" {
        return false;
    }
    else {
        return true;
    }
}
struct TabViewer<'a> {
    added_nodes: &'a mut Vec<NodeIndex>,
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

        self.window_size = _frame.info().window_info.size;
        const ICON_BYTES: &[u8] = include_bytes!("../icon.ico");
        let args: Vec<String> = env::args().collect();
        //[0]self
        //path
        if args.len() == 2 && self.read_from_args {
            self.read_from_args = false;
            match std::fs::metadata(args[1].clone()){
                Ok(m) => {
                    if m.is_file() && !m.is_dir(){
                        self.last_save_path = Some(args[1].clone().into());
                        self.code_editor.code = openfile(self.last_save_path.clone());
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
                },
                Err(_) => {
                    println!("File doesnt exist");
                }
            }
           
        }

        if self.code_editor_text_lenght > self.code_editor.code.len() || self.code_editor.code.len() == 0 {
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
                    self.opened_file = file_name_str.to_string();
                }
            }
            if self.code_editor.code.len() != self.code_editor_text_lenght {
                let window_title = self.window_title.clone() + &"*".as_str();
                _frame.set_window_title(window_title.as_str());
            }
            else {
                _frame.set_window_title(self.window_title.as_str());
            }
        }
        //frame settings
        
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
            self.can_open = !self.can_open;
        }
        if ctrlis_pressed && tis_pressed && has_focus {
            if !self.settings_window_is_open {
                self.settings_window_is_open = !self.settings_window_is_open;
            }
        }
        if ctrlis_pressed && nis_pressed && has_focus {
            self.can_save_as = !self.can_save_as;
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

        
        if self.terminal_help {
            egui::Window::new("Terminal")
                .fixed_size((self.window_size[0] / 1.5, self.window_size[1] / 2.0))
                .resizable(false)
                .open(&mut self.terminal_help)
                .show(ctx, |ui| {

                    ui.style_mut().visuals.window_fill = Color32::BLACK;
                    
                    let frame_rect = ui.max_rect().shrink(5.0);
                    
                        ui.allocate_space(egui::vec2(ui.available_width(), 5.));
                        ui.allocate_space(egui::vec2(ui.available_width(), ui.available_height() - 5.));

                        ui.painter().rect(
                            frame_rect,
                            Rounding::same(5.0),
                            Color32::BLACK,
                            Stroke::NONE,
                        );
                        let code_rect = frame_rect.shrink(5.0);
    
                        let mut frame_ui = ui.child_ui(code_rect, Layout::default());
    
                        egui::ScrollArea::vertical()
                            .id_source("terminal")
                            .stick_to_bottom(true)
                            .show(&mut frame_ui, |ui| {
                                ui.add(terminal::new(&mut self.terminal_terminal_style, ui.available_size()));
                            });
                    
                    
                    
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
                    if self.code_editor_text_lenght <= self.code_editor.code.len() {
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
                    ui.label(RichText::from("Application").size(20.0));
                    if ui.button("Add to windows context menu").clicked() {
                        let path = format!("{}", env::current_dir().unwrap().display());
                        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                        let environment_key = hklm.open_subkey_with_flags(
                            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
                            KEY_READ | KEY_WRITE,
                        );

                        match environment_key {
                            Ok(key) => {
                                // Read the existing PATH value
                                let mut path_value: String = key.get_value("PATH").unwrap_or_default();

                                // Append the new path to the existing value
                                if !path_value.is_empty() {
                                    path_value.push_str(";");
                                }
                                path_value.push_str(&path);

                                // Set the modified PATH value
                                key.set_value("PATH", &path_value).expect("Failed to set value");

                                // Notify the user about the updated PATH
                            }
                            Err(err) => {
                                eprintln!("Error: {}", err);
                            }
                        }
                        
                        if let Ok(exe_path) = env::current_exe() {
                            let app_path = exe_path;
                            let _ = std::process::Command::new("reg")
                                .args(&[
                                    "add",
                                    "HKEY_CLASSES_ROOT\\*\\shell\\Open file with Marcide\\command",
                                    "/ve",
                                    "/d",
                                    &format!("\"{}\" \"%1\"", app_path.display()),
                                    "/f",
                                ])
                                .output()
                                .expect("Failed to execute command");
                            let path = format!("C:\\Users\\{}\\AppData\\Roaming\\Marcide\\data\\icon.ico", env::var("USERNAME").unwrap());
                            let mut output_file = std::fs::File::create(path).expect("Failed to create file");
                            io::Write::write_all(&mut output_file, ICON_BYTES).expect("Failed to write to file");
                            let icon_path = format!(r#""C:\\Users\\{}\\AppData\\Roaming\\Marcide\\data\\icon.ico""#, env::var("USERNAME").unwrap());
                            let _ = std::process::Command::new("reg")
                                .args(&[
                                    "add",
                                    "HKEY_CLASSES_ROOT\\*\\shell\\Open file with Marcide",
                                    "/v",
                                    "Icon",
                                    "/d",
                                    &format!("{}", icon_path),
                                    "/f",
                                ])
                                .output()
                                .expect("Failed to execute command");
                        }
                    };
                    if ui.button("Remove from windows context menu").clicked() {
                        let path = format!("{}", env::current_dir().unwrap().display());
                        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                        let environment_key = hklm.open_subkey_with_flags(
                            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
                            KEY_READ | KEY_WRITE,
                        );

                        match environment_key {
                            Ok(key) => {
                                // Read the existing PATH value
                                let path_value: String = key.get_value("Path").unwrap_or_default();

                                // Split the existing paths by semicolon
                                let paths: Vec<_> = path_value.split(';').collect();

                                // Create a new PATH value without the path to remove
                                let new_paths: Vec<_> = paths.into_iter().filter(|&p| p != path).collect();
                                let new_path_value = new_paths.join(";");

                                // Set the modified PATH value
                                key.set_value("Path", &new_path_value).expect("Failed to set value");

                                // Notify the user about the updated PATH
                                println!("Updated PATH: {}", new_path_value);
                            }
                            Err(err) => {
                                eprintln!("Error: {}", err);
                            }
                        }
                        let _ = std::process::Command::new("reg")
                            .args(&[
                                "delete",
                                "HKEY_CLASSES_ROOT\\*\\shell\\Open file with Marcide",
                                "/f",
                            ])
                            .output()
                            .expect("Failed to execute command");
                    };
                    ui.separator();
                    ui.label(RichText::from("Window").size(20.0));
                    ui.checkbox(&mut self.window_options_always_on_top, "Always on top");
                    ui.separator();
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
                    ui.separator();
                    ui.label(RichText::from("Configuration").size(20.0));
                    ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui|{
                        if ui.button("Export").clicked() {
                            let files = FileDialog::new()
                            .set_title("Save as")
                            .add_filter("Marcide config", &["marcfg"])
                            .set_directory("/")
                            .save_file();
                            if files.clone().is_some() {
                                let config = format!("{}\n{}\n{}\n{}\n{}\n{}\n{}",self.window_options_always_on_top, self.auto_save , self.auto_save_to_ram, self.language, self.is_gui_development, self.terminal_mode, self.unsafe_mode);
                                savetofile(files.clone(), config);
                            };
                        }
                        if ui.button("Import").clicked() {
                            let files = FileDialog::new()
                                .set_title("Save as")
                                .add_filter("Marcide config", &["marcfg"])
                                .set_directory("/")
                                .pick_file();
                            let contains = openfile(files);
                            let lines : Vec<&str> = contains.lines().collect();
                            //xd
                            self.window_options_always_on_top = trueorfalse(lines[0].to_owned());
                            self.auto_save = trueorfalse(lines[1].to_owned());
                            self.auto_save_to_ram = trueorfalse(lines[2].to_owned());
                            self.language = lines[3].to_owned();
                            self.is_gui_development = trueorfalse(lines[4].to_owned());
                            self.terminal_mode = trueorfalse(lines[5].to_owned());
                            self.unsafe_mode = trueorfalse(lines[6].to_owned());

                        };
                    });
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
                                let mut to_push: String = String::new();
                                if !self.terminal_mode {
                                    to_push = format!("%marcide.temp%\\temp.{}", self.language);
                                }
                                else if self.unsafe_mode {
                                    to_push = format!("%marcide.temp%\\temp");
                                }
                                else {
                                    to_push = format!("%marcide.temp%\\temp.bat");
                                }
                               
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
                if open.clicked() || self.can_open {
                    self.can_open = false;
                    let files = FileDialog::new()
                        .set_title("Open")
                        .set_directory("/")
                        .pick_file();
                    if files.clone().is_some() {
                        self.last_save_path = files.clone();
                        self.code_editor.code = openfile(self.last_save_path.clone());
                        self.code_editor_text_lenght = self.code_editor.code.len();
                    }
                }
                if save_as.clicked() || self.can_save_as {
                    self.can_save_as = false;
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
                    } else if self.code_editor_text_lenght <= self.code_editor.code.len() {
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
                    //newcmd();
                    self.terminal_help = !self.terminal_help;
                }
                if support.clicked() {
                    match webbrowser::open("https://discord.gg/7s3VRr4H6j") {
                        Ok(_) => {}
                        Err(_) => {}
                    };
                }
                run.on_hover_text("CTRL + R");
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
    }
}
impl egui_dock::TabViewer for TemplateApp {
    type Tab = usize;
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        if *tab == 1 {
            //terminal
            
        }
        if *tab == 2 {
            //code editor
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.label("text");

                },
            );
        }
    }
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        if *(tab) == 1 {
            format!("Terminal").into()
        }
        else if *(tab) == 2 {
            format!("Code editor").into()
        }
        else {
            format!("Tab {tab}").into()
        }
    }
}
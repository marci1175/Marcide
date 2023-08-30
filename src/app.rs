use self::code_editor::CodeEditor;
use dirs::home_dir;
use egui::{Layout, Stroke, Rounding, Ui, Widget};
use egui::{Color32, RichText, TextBuffer, Vec2};

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


use eframe::{egui, CreationContext, App, Frame};
use egui_terminal::TermHandler;

//mod gks;
mod terminal;
mod cmdmod;
mod code_editor;
mod richpresence;

use cmdmod::{
    finder, mkdir, openfile, rmdir, runfile, savetofile, terminalr
};

use egui_dock::{DockArea, NodeIndex, Style, Tree};
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    counter: usize,
    #[serde(skip)]
    go_to_offset: bool,
    
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
}

impl Default for TemplateApp {
    fn default() -> Self {
        let (sender, recv) = mpsc::channel::<String>();
        Self {
            counter: 0,
            go_to_offset: false,
            
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
        }
    }
}

struct TabViewer<'a> {
    added_nodes: &'a mut Vec<NodeIndex>,
    ctx: &'a egui::Context,
    frame: &'a mut Frame,
    data: &'a mut TemplateApp,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AppData {
    app_data : TemplateApp,
    #[serde(skip)]
    tree: Tree<usize>,
}
impl Default for AppData {
    fn default() -> Self {
        Self {
            app_data: TemplateApp::default(),
            tree: Tree::new(vec![1, 2, 3, 4, 5]),
        }
    }
}
impl AppData {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}
impl eframe::App for AppData {
    fn on_close_event(&mut self) -> bool {
        //remove temp dir
        rmdir();
        let window_on_top_state = self.app_data.window_options_always_on_top;
        if window_on_top_state {
            self.app_data.window_options_always_on_top = false;
        }
        if !self.app_data.auto_save || self.app_data.last_save_path.is_none() {
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
                            self.app_data.last_save_path = files.clone();
                            savetofile(files.clone(), self.app_data.text.clone());
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
                            self.app_data.window_options_always_on_top = true;
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
        if !self.app_data.auto_save_to_ram {
            self.app_data.code_editor.code.clear();
        }
        
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        let mut added_nodes = Vec::new();
        DockArea::new(&mut self.tree)
            .show_close_buttons(true)
            //.show_add_buttons(true)
            .style({
                let mut style = Style::from_egui(ctx.style().as_ref());
                style.tabs.fill_tab_bar = true;
                style
            })
            .show(
                ctx,
                &mut TabViewer {
                    added_nodes: &mut added_nodes,
                    ctx,
                    frame : _frame,
                    data: &mut self.app_data,
                },
            );

        added_nodes.drain(..).for_each(|node| {
            self.tree.set_focused_node(node);
            self.tree.push_to_focused_leaf(self.app_data.counter);
            self.app_data.counter += 1;
        });
    }
}


    /*fn on_add(&mut self, node: NodeIndex) {
        self.added_nodes.push(node);
    }*/

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

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = usize;
    ///ctx = ctx _frame = _frame
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        //rest of the app
        self.data.window_size = self.frame.info().window_info.size;
        const ICON_BYTES: &[u8] = include_bytes!("../icon.ico");
        let args: Vec<String> = env::args().collect();
        //[0]self
        //path
        if args.len() == 2 && self.data.read_from_args {
            self.data.read_from_args = false;
            match std::fs::metadata(args[1].clone()){
                Ok(m) => {
                    if m.is_file() && !m.is_dir(){
                        self.data.last_save_path = Some(args[1].clone().into());
                        self.data.code_editor.code = openfile(self.data.last_save_path.clone());
                        self.data.code_editor_text_lenght = self.data.code_editor.code.len();
                    }
                },
                Err(_) => {
                    println!("File doesnt exist");
                }
            }
           
        }

        if self.data.code_editor_text_lenght > self.data.code_editor.code.len() || self.data.code_editor.code.len() == 0 {
            self.data.code_editor_text_lenght = self.data.code_editor.code.len();
        }

        match self.data.recv.try_recv() {
            Ok(ok) => {
                self.data.output = ok.clone();
            }
            Err(_) => { /*Task didnt finsih yet*/ }
        };
        let _projname: String = self.data.opened_file.clone();
        let starttime: String = self.data.session_started.format("%m-%d %H:%M:%S").to_string();
        let tx = self.data.rpc_sender.get_or_insert_with(|| {
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
        match tx.send(self.data.opened_file.clone()) {
            Ok(_) => {}
            Err(err) => {
                println!("Failed to send msg : {}", err)
            }
        };

        self.data.discord_presence_is_running = true;

        //title logic for * when unsaved

        //hotkeys
        if let Some(wintitle) = self.data.last_save_path.clone() {
            if let Some(file_name) = wintitle.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    self.data.window_title = format!("Marcide - {}", file_name_str.to_string());
                    self.data.opened_file = file_name_str.to_string();
                }
            }
            if self.data.code_editor.code.len() != self.data.code_editor_text_lenght {
                let window_title = self.data.window_title.clone() + &"*".as_str();
                self.frame.set_window_title(window_title.as_str());
            }
            else {
                self.frame.set_window_title(self.data.window_title.as_str());
            }
        }
        //frame settings
        
        self.frame.set_fullscreen(self.data.window_options_full_screen);
        self.frame.set_always_on_top(self.data.window_options_always_on_top);
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
        let has_focus = self.ctx.input(|i| i.focused);
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
            self.data.window_options_full_screen = !self.data.window_options_full_screen;
        }
        if alt_is_pressed {
            ctrlis_pressed = false;
        }
        if sis_pressed && ctrlis_pressed && has_focus {
            self.data.can_save = !self.data.can_save;
        }
        //finder hotkey
        if ctrlis_pressed && fis_pressed && has_focus {
            if !self.data.finder_is_open {
                self.data.finder_is_open = true;
            }
        }
        if ctrlis_pressed && ris_pressed && has_focus {
            if !self.data.output_window_is_open {
                self.data.can_run = !self.data.can_run
            }
        }
        if ctrlis_pressed && fis_pressed && has_focus {
            if !self.data.finder_is_open {
                self.data.finder_is_open = !self.data.finder_is_open;
            }
        }
        if ctrlis_pressed && ois_pressed && has_focus {
            self.data.can_open = !self.data.can_open;
        }
        if ctrlis_pressed && tis_pressed && has_focus {
            if !self.data.settings_window_is_open {
                self.data.settings_window_is_open = !self.data.settings_window_is_open;
            }
        }
        if ctrlis_pressed && nis_pressed && has_focus {
            self.data.can_save_as = !self.data.can_save_as;
        }
        
        /*egui::TopBottomPanel::new(egui::panel::TopBottomSide::Top, "settings").show(self.ctx, |ui|{
                    ui.label("text");
                }); */
            egui::CentralPanel::default().show_inside(ui, |ui|{
            
                if *tab == 2 {
                    let frame_rect = dbg!(ui.max_rect());
                    
                        ui.allocate_ui_at_rect(frame_rect, |ui|{
                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::Center),
                                |ui| {
                                    self.data.scroll_offset = code_editor::CodeEditor::show(
                                        &mut self.data.code_editor,
                                        "id".into(),
                                        ui,
                                        self.data.scroll_offset,
                                        self.data.go_to_offset,
                                    );
                                },
                            );
                        });
                        
                    
                    
                }
                if *tab == 1 {
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
                            ui.add(terminal::new(&mut self.data.terminal_terminal_style, ui.available_size()));
                        });
                }
            });
        
        //tab 1 == code editor tab 2 == terminal
        
        
    }
    
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        if *(tab) == 1 {
            return format!("Terminal").into()
        }
        else if *(tab) == 2 {
            return format!("Code editor").into()
        }
        else if *(tab) == 3 {
            return format!("Find").into()
        }
        else if *(tab) == 4 {
            return format!("Output").into()
        }
        else if *(tab) == 5 {
            return format!("Settings").into()
        }
        else {
            format!("Tab {}", tab).into()
        }
    }

}
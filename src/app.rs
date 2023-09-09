use self::code_editor::CodeEditor;

use dirs::home_dir;

use egui::{Color32, RichText, TextBuffer, Vec2};
use egui::{Layout, Rounding, Stroke};

use egui_terminal::term::CommandBuilder;
use rfd::FileDialog;
use std::env;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;

use windows_sys::w;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_F, VK_F11, VK_M, VK_N, VK_O, VK_R, VK_RMENU, VK_S, VK_T,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_ICONEXCLAMATION, MB_OK, MB_YESNOCANCEL,
};

use eframe::{egui, Frame};
use egui_terminal::TermHandler;

//mod gks;
mod file_handling;
mod cmdmod;
mod code_editor;
mod richpresence;
mod terminal;
mod win_handling;

use win_handling::{
    add_win_ctx, remove_win_ctx,
};
use file_handling::{
    openf, savef, savefas, savefas_w, openf_w
};
use cmdmod::{finder, mkdir, openfile, rmdir, runfile, savetofile, terminalr};

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
    can_save_as: bool,
    #[serde(skip)]
    can_open: bool,
    #[serde(skip)]
    can_save: bool,
    #[serde(skip)]
    can_run: bool,
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
    #[serde(skip)]
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
    first_run: bool,
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
            first_run: true,
        }
    }
}

struct TabViewer<'a> {
    //tree: &'a mut Tree<usize>,
    added_nodes: &'a mut Vec<NodeIndex>,
    ctx: &'a egui::Context,
    frame: &'a mut Frame,
    data: &'a mut TemplateApp,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AppData {
    app_data: TemplateApp,
    #[serde(skip)]
    tree: Tree<usize>,
    #[serde(skip)]
    f11_is_held: bool,
}
impl Default for AppData {
    fn default() -> Self {
        Self {
            app_data: TemplateApp::default(),
            tree: Tree::new(vec![1, 2, 3]),
            f11_is_held: false,
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
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let altimput = unsafe { GetAsyncKeyState(VK_RMENU as i32) as u16 & 0x8000 != 0 };
        let ninput = unsafe { GetAsyncKeyState(VK_N as i32) as u16 & 0x8000 != 0 };
        let oinput = unsafe { GetAsyncKeyState(VK_O as i32) as u16 & 0x8000 != 0 };
        let rinput = unsafe { GetAsyncKeyState(VK_R as i32) as u16 & 0x8000 != 0 };
        let tinput = unsafe { GetAsyncKeyState(VK_T as i32) as u16 & 0x8000 != 0 };

        let has_focus = ctx.input(|i| i.focused);
        let ctrlinput = unsafe { GetAsyncKeyState(VK_CONTROL as i32) as u16 & 0x8000 != 0 };
        let f11input = unsafe { GetAsyncKeyState(VK_F11 as i32) as u16 & 0x8000 != 0 };
        let sinput = unsafe { GetAsyncKeyState(VK_S as i32) as u16 & 0x8000 != 0 };
        let finput = unsafe { GetAsyncKeyState(VK_F as i32) as u16 & 0x8000 != 0 };
        let minput = unsafe { GetAsyncKeyState(VK_M as i32) as u16 & 0x8000 != 0 };
        //set fullscreen
        if f11input && has_focus && !self.f11_is_held {
            self.app_data.window_options_full_screen = !self.app_data.window_options_full_screen;
            self.f11_is_held = true;
        }
        //prevent fullscreen spamming
        if !f11input {
            self.f11_is_held = false;
        }

        let mut added_nodes = Vec::new();
        let tx = self.app_data.autosave_sender.get_or_insert_with(|| {
            let (tx, rx) = mpsc::channel::<String>();

            std::thread::spawn(move || loop {
                match rx.try_recv() {
                    Ok(text) => {
                        //println!("{}", lines[1]);
                        let lines: Vec<&str> = text.split("VxpAM$616*9Y8G%tOp$en*KDJ").collect();

                        savetofile(Some(PathBuf::from(lines[1].trim())), lines[0].to_string());
                    }
                    Err(_) => {
                        //"SzeRinTetEk tuDja A hArmAdiK szAbÁLyT?"
                    }
                };
            });
            tx
        });
        if self.app_data.auto_save {
            if let Some(path) = self.app_data.last_save_path.clone() {
                let data_to_send: String = format!(
                    "{} VxpAM$616*9Y8G%tOp$en*KDJ {}",
                    self.app_data.code_editor.code.clone(),
                    path.to_str().unwrap_or_default().to_string()
                );
                if self.app_data.code_editor_text_lenght <= self.app_data.code_editor.code.len() {
                    match tx.send(data_to_send) {
                        Ok(_) => {}
                        Err(_) => {}
                    };
                    self.app_data.code_editor_text_lenght = self.app_data.code_editor.code.len();
                }
            }
        }
        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Top, "settings").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                //define buttons
                let _ = ui.menu_button("File", |ui| {
                    let new = ui.button("New").on_hover_text("CTRL + N");
                    let open = ui.button("Open").on_hover_text("CTRL + O");
                    let save = ui.button("Save").on_hover_text("CTRL + S");
                    let save_as = ui.button("Save as").on_hover_text("CTRL + M");
                    ui.separator();
                    let open_workspace = ui.button("Open workspace");
                    if self.app_data.last_save_path.is_some() {
                        let save_workspace = ui.button("Save workspace");
                        if save_workspace.clicked() {
                            let mut data_to_write : Vec<String> = Vec::new();
                            data_to_write.push(self.app_data.code_editor.language.clone() + ";");
                            data_to_write.push(self.app_data.code_editor_text_lenght.to_string() + ";");
                            data_to_write.push(self.app_data.last_save_path.clone().unwrap().display().to_string() + ";");
                            data_to_write.push(self.app_data.code_editor.code.clone() + ";");
                            //protection
                            let workspace_final = data_to_write.join("");
                            data_to_write.push("MARCIDE_WORKSPACE;".to_string());
                            savefas_w("Save Marcide workspace as", workspace_final);
                        }
                    }
                    if open_workspace.clicked() {
                        let file = openf_w("Open Marcide workspace");
                        //let items : Vec<&str> =  file.unwrap().split(";").collect();
                        if let Some(items) = file {
                            let items = items.split(";").collect::<Vec<&str>>();
                            if items[4] == "MARCIDE_WORKSPACE" {
                                self.app_data.code_editor.language = items[0].to_string();
                                self.app_data.code_editor_text_lenght = items[1].parse().unwrap();
                                self.app_data.last_save_path = Some(PathBuf::from(items[2]));
                                self.app_data.code_editor.code = items[3].to_string();
                            }
                        }
                            
                    }
                    ui.separator();
                    ui.checkbox(&mut self.app_data.auto_save, "Auto save");
                    let settings = ui.button("Settings");
                    if new.clicked() {
                        let (y, z) = savefas(
                            self.app_data.last_save_path.clone(),
                            Some(self.app_data.code_editor_text_lenght),
                            self.app_data.code_editor.code.clone(),
                        );
                        if y.is_some() && z.is_some() {
                            self.app_data.code_editor_text_lenght = y.unwrap();
                            self.app_data.last_save_path = z;
                            self.app_data.code_editor.code.clear();
                            self.app_data.can_save_as = false;
                        } else {
                            //aborted save
                        }
                    }
                    if open.clicked() {
                        let (x, y, z) = openf(
                            self.app_data.last_save_path.clone(),
                            self.app_data.code_editor_text_lenght,
                            self.app_data.code_editor.code.clone(),
                        );
                        self.app_data.code_editor_text_lenght = x;
                        self.app_data.code_editor.code = y;
                        self.app_data.last_save_path = z;
                    }
                    if save.clicked() {
                        self.app_data.code_editor_text_lenght = savef(
                            self.app_data.last_save_path.clone(),
                            self.app_data.code_editor.code.clone(),
                            self.app_data.code_editor_text_lenght,
                        );
                    }
                    if save_as.clicked() {
                        let (y, z) = savefas(
                            self.app_data.last_save_path.clone(),
                            Some(self.app_data.code_editor_text_lenght),
                            self.app_data.code_editor.code.clone(),
                        );
                        if y.is_some() && z.is_some() {
                            self.app_data.can_save_as = false;
                            self.app_data.code_editor_text_lenght = y.unwrap();
                            self.app_data.last_save_path = z;
                        } else {
                            //aborted save
                        }
                    }
                    if settings.clicked() {
                        self.app_data.settings_window_is_open =
                            !self.app_data.settings_window_is_open;
                        if self.tree.find_tab(&5).is_none() {
                            self.tree.push_to_first_leaf(5);
                        };
                    }
                    return (new, open, save, save_as, settings);
                });   

                if ninput && has_focus && ctrlinput {
                    let (y, z) = savefas(
                        self.app_data.last_save_path.clone(),
                        Some(self.app_data.code_editor_text_lenght),
                        self.app_data.code_editor.code.clone(),
                    );
                    if y.is_some() && z.is_some() {
                        self.app_data.code_editor_text_lenght = y.unwrap();
                        self.app_data.last_save_path = z;
                        self.app_data.code_editor.code.clear();
                        self.app_data.can_save_as = false;
                    } else {
                        //aborted save
                    }
                }
                if oinput && has_focus && ctrlinput {
                    //self.app_data.can_open = false;
                    let (x, y, z) = openf(
                        self.app_data.last_save_path.clone(),
                        self.app_data.code_editor_text_lenght,
                        self.app_data.code_editor.code.clone(),
                    );
                    self.app_data.code_editor_text_lenght = x;
                    self.app_data.code_editor.code = y;
                    self.app_data.last_save_path = z;
                }
                if sinput && has_focus && ctrlinput {
                    self.app_data.code_editor_text_lenght = savef(
                        self.app_data.last_save_path.clone(),
                        self.app_data.code_editor.code.clone(),
                        self.app_data.code_editor_text_lenght,
                    );
                }
                if minput && has_focus && ctrlinput {
                    let (y, z) = savefas(
                        self.app_data.last_save_path.clone(),
                        Some(self.app_data.code_editor_text_lenght),
                        self.app_data.code_editor.code.clone(),
                    );
                    if y.is_some() && z.is_some() {
                        self.app_data.can_save_as = false;
                        self.app_data.code_editor_text_lenght = y.unwrap();
                        self.app_data.last_save_path = z;
                    } else {
                        //aborted save
                    }
                }

                let _ = ui.menu_button("Edit", |ui| {
                    let copy = ui.button("Copy").on_hover_text("CTRL + C");
                    let paste = ui.button("Paste").on_hover_text("CTRL + V");
                    let cut = ui.button("Cut").on_hover_text("CTRL + X");
                    let undo = ui.button("Undo").on_hover_text("CTRL + Z");
                    let redo = ui.button("Redo").on_hover_text("CTRL + Y");
                    let select_all = ui.button("Select all").on_hover_text("CTRL + A");
                    let find = ui.button("Find").on_hover_text("CTRL + F");

                    if copy.clicked() {}
                    if paste.clicked() {}
                    if cut.clicked() {}
                    if undo.clicked() {}
                    if redo.clicked() {}
                    if select_all.clicked() {}
                    if find.clicked() {
                        self.app_data.finder_is_open = !self.app_data.finder_is_open;
                        if self.tree.find_tab(&3).is_none() {
                            self.tree.push_to_first_leaf(3);
                        };
                    }
                    return (copy, paste, cut, undo, redo, select_all, find);
                
                });
                if finput && has_focus && ctrlinput {
                    self.app_data.finder_is_open = !self.app_data.finder_is_open;
                    if self.tree.find_tab(&3).is_none() {
                        self.tree.push_to_first_leaf(3);
                    };
                }
                //code
                let run = ui.button("Run").on_hover_text("CTRL + R");
                if run.clicked() || rinput && has_focus && ctrlinput {
                    if self.tree.find_tab(&4).is_none() {
                        self.tree.push_to_first_leaf(4);
                    };
                    if self.app_data.terminal_mode
                        || self.app_data.unsafe_mode
                        || self.app_data.language == "py"
                        || self.app_data.language == "lua"
                    {
                        //reset value
                        self.app_data.can_run = false;
                        //save to temp folder
                        if self.app_data.last_save_path.is_none() {

                            mkdir();

                            //C:\Users\%user_name%\marcide.temp
                            if let Some(mut home_dir) = home_dir() {
                                let mut to_push: String = String::new();
                                if !self.app_data.terminal_mode {
                                    to_push =
                                        format!("%marcide.temp%\\temp.{}", self.app_data.language);
                                } else if self.app_data.unsafe_mode {
                                    to_push = format!("%marcide.temp%\\temp");
                                } else {
                                    to_push = format!("%marcide.temp%\\temp.bat");
                                }

                                home_dir.push(to_push);

                                // Set the files variable
                                let files: Option<PathBuf> = Some(home_dir);
                                //save file
                                
                                savetofile(files.clone(), self.app_data.code_editor.code.clone());
                                //run file

                                let lang = self.app_data.language.clone();
                                let s = self.app_data.sender.clone();
                                let terminalm = self.app_data.terminal_mode.clone();
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
                            let files = self.app_data.last_save_path.clone();

                            let lang = self.app_data.language.clone();
                            let terminalm = self.app_data.terminal_mode.clone();
                            let s = self.app_data.sender.clone();
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

                let terminal = ui.button("Terminal").on_hover_text("CTRL + T");
                if terminal.clicked() || tinput && has_focus && ctrlinput {
                    self.app_data.terminal_help = !self.app_data.terminal_help;
                    if self.tree.find_tab(&1).is_none() {
                        self.tree.push_to_first_leaf(1);
                    };
                }

                let help = ui.button("Help").on_hover_text("Official discord server");
                if help.clicked() {
                    match webbrowser::open("https://discord.gg/7s3VRr4H6j") {
                        Ok(_) => {}
                        Err(_) => {}
                    };
                }
                /*

                run.on_hover_text("CTRL + R");
                open.on_hover_text("CTRL + O");
                save_as.on_hover_text("CTRL + N");
                save.on_hover_text("CTRL + S");
                find.on_hover_text("CTRL + F");
                settings.on_hover_text("CTRL + T");

                */
            });
        });

        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "stats").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let lenght = self.app_data.code_editor.code.len();
                let lines = count_lines(&self.app_data.code_editor.code);
                let final_lenght = lenght - (lines - 1);
                ui.label(lines.to_string() + " : Lines");

                //separate self.label into a vector by whitespaces, then count them
                let trimmed_string = self.app_data.code_editor.code.trim();
                let words: Vec<&str> = trimmed_string.split_whitespace().collect();
                ui.label(words.len().to_string() + " : Words");
                ui.label(final_lenght.to_string() + " : Characters");
                ui.separator();
                let current_datetime = chrono::Local::now();
                let datetime_str = current_datetime.format("%H:%M:%S ").to_string();
                let sessiondate_str = self
                    .app_data
                    .session_started
                    .format("%H:%M:%S ")
                    .to_string();
                ui.label(format!("Session started : {}", sessiondate_str));
                ui.label(format!("Current time : {}", datetime_str));
                ctx.request_repaint();
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(
                RichText::from("Open a file to start editing!")
                    .size(20.)
                    .color(Color32::LIGHT_BLUE),
            );
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
                        //tree: &mut self.tree,
                        added_nodes: &mut added_nodes,
                        ctx,
                        frame: _frame,
                        data: &mut self.app_data,
                    },
                );
        });

        added_nodes.drain(..).for_each(|node| {
            self.tree.set_focused_node(node);
            self.tree.push_to_focused_leaf(self.app_data.counter);
            self.app_data.counter += 1;
        });
    }
}

fn count_lines(text: &str) -> usize {
    text.split('\n').count()
}
fn trueorfalse(arg: String) -> bool {
    if arg.to_lowercase() == "false" {
        return false;
    } else {
        return true;
    }
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = usize;
    ///ctx = ctx _frame = _frame
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        //reset find state
        self.data.go_to_offset = false;
        
        //rest of the app
        self.data.window_size = self.frame.info().window_info.size;
        const ICON_BYTES: &[u8] = include_bytes!("../icon.ico");
        let args: Vec<String> = env::args().collect();
        //[0]self
        //path
        if args.len() == 2 && self.data.read_from_args {
            self.data.read_from_args = false;
            match std::fs::metadata(args[1].clone()) {
                Ok(m) => {
                    if m.is_file() && !m.is_dir() {
                        self.data.last_save_path = Some(args[1].clone().into());
                        self.data.code_editor.code = openfile(self.data.last_save_path.clone());
                        self.data.code_editor_text_lenght = self.data.code_editor.code.len();
                    }
                }
                Err(_) => {
                    println!("File doesnt exist");
                }
            }
        }

        if self.data.code_editor_text_lenght > self.data.code_editor.code.len()
            || self.data.code_editor.code.is_empty()
        {
            self.data.code_editor_text_lenght = self.data.code_editor.code.len();
        }

        match self.data.recv.try_recv() {
            Ok(ok) => {
                self.data.output = ok.clone();
            }
            Err(_) => { /*Task didnt finsih yet*/ }
        };

        let starttime: String = self
            .data
            .session_started
            .format("%m-%d %H:%M:%S")
            .to_string();
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
                    Err(_) => {}
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
                    self.data.window_title = format!("Marcide - {}", file_name_str);
                    self.data.opened_file = file_name_str.to_string();
                }
            }
            if self.data.code_editor.code.len() != self.data.code_editor_text_lenght {
                let window_title = self.data.window_title.clone() + "*".as_str();
                self.frame.set_window_title(window_title.as_str());
            } else {
                self.frame.set_window_title(self.data.window_title.as_str());
            }
        }
        //frame settings

        self.frame
            .set_fullscreen(self.data.window_options_full_screen);
        self.frame
            .set_always_on_top(self.data.window_options_always_on_top);
        //get alt input => if true ctrl => false
        
        if *tab == 3 {
            let occurence: usize;
            ui.label("Finder");
            ui.text_edit_singleline(&mut self.data.to_find);
            if ui.button("Search").clicked() {
                let occur: io::Result<Vec<usize>> = finder(
                    self.data.code_editor.code.clone(),
                    self.data.to_find.clone(),
                );
                if occur.as_ref().unwrap().is_empty() {
                    self.data.is_found = None;
                } else {
                    self.data.go_to_offset = true;
                    let px = ui.fonts(|f| {
                        f.row_height(&egui::FontId {
                            size: 10.0,
                            family: egui::FontFamily::Monospace,
                        })
                    });
                    occurence = occur.as_ref().unwrap()[0];
                    self.data.scroll_offset[1] = occurence as f32 * px;

                    self.data.occurences = occur.unwrap().len();
                    self.data.is_found = Some(true);
                }
                //update scroll offset and done!
            }
            if self.data.is_found.is_none() {
                ui.colored_label(Color32::RED, "0 Occurences in the text");
            } else {
                ui.colored_label(
                    Color32::GREEN,
                    format!("Appears in {} line(s)", self.data.occurences),
                );
            }
        } else if *tab == 2 {
            let frame_rect = ui.max_rect();
            ui.allocate_ui_at_rect(frame_rect, |ui| {
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        self.data.scroll_offset = code_editor::CodeEditor::show(
                            &mut self.data.code_editor,
                            "code".into(),
                            ui,
                            self.data.scroll_offset,
                            self.data.go_to_offset,
                        );
                    },
                );
            });
        } else if *tab == 1 {
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
                    ui.add(terminal::new(
                        &mut self.data.terminal_terminal_style,
                        ui.available_size(),
                    ));
                });
        } else if *tab == 4 {
            ui.label(RichText::from("Output").size(20.).color(Color32::GRAY));

            ui.label(RichText::from(self.data.output.clone()).size(30.));
        } else if *tab == 5 {
            ui.label(RichText::from("Application").size(20.0));
            if ui.button("Add to windows context menu").clicked() {
                add_win_ctx(ICON_BYTES);
            };
            if ui.button("Remove from windows context menu").clicked() {
                remove_win_ctx();
            };
            
            ui.separator();
            ui.label(RichText::from("Window").size(20.0));
            ui.checkbox(&mut self.data.window_options_always_on_top, "Always on top");
            ui.separator();
            ui.label(egui::RichText::from("File handling").size(20.0));
            ui.checkbox(&mut self.data.auto_save, "Autosave to file");
            if self.data.auto_save {
                ui.label("Autosave will only turn on after you have saved your file somewhere.");
            }
            //this will enable/disable the savestate feature to codeeditor.code
            ui.checkbox(&mut self.data.auto_save_to_ram, "Autosave");
            if self.data.auto_save_to_ram {
                ui.label("This will save your text temporarily in the application. ");
            }
            ui.separator();
            ui.label(egui::RichText::from("Programming language").size(20.0));
            ui.text_edit_singleline(&mut self.data.code_editor.language);
            if self.data.code_editor.language == "quran"
                || self.data.code_editor.language == "Quran"
                || self.data.code_editor.language == "Korán"
                || self.data.code_editor.language == "korán"
            {
                ui.hyperlink_to("Quran", "https://mek.oszk.hu/06500/06534/06534.pdf");
            }
            if self.data.code_editor.language == "marci1175"
                || self.data.code_editor.language == "marci"
                || self.data.code_editor.language == "Marci"
                || self.data.code_editor.language == "Marcell"
                || self.data.code_editor.language == "Varga Marcell"
            {
                ui.separator();
                ui.label("Credits");
                ui.separator();
                ui.label("Made by : Varga Marcell also known as marci1175 at 5111 days old");
                ui.label("Had so much fun developing this lol.");
                ui.hyperlink_to("Github", "https://github.com/marci1175");
            }
            if self.data.code_editor.language == "py" || self.data.code_editor.language == "lua" {
                ui.checkbox(&mut self.data.is_gui_development, "Gui mode");
                if self.data.is_gui_development {
                    ui.label(RichText::from("Output window wont show up when running the application, with gui mode.").color(Color32::LIGHT_YELLOW));
                    self.data.output_window_is_open = false;
                }
            } else {
                self.data.is_gui_development = false;
            }
            if self.data.code_editor.language == "bat"
                || self.data.code_editor.language == "cmd"
                || self.data.code_editor.language == "ps1"
                || self.data.code_editor.language == "vbs"
                || self.data.code_editor.language == "wsf"
                || self.data.code_editor.language == "reg"
            {
                self.data.terminal_mode = true;
                ui.label(
                    RichText::from("Terminal mode is on, but syntaxing is unavailable")
                        .color(Color32::LIGHT_YELLOW),
                );
            } else if !self.data.unsafe_mode {
                self.data.terminal_mode = false;
            }
            ui.checkbox(&mut self.data.unsafe_mode, "Unsafe mode");
            if self.data.unsafe_mode {
                ui.label(
                            "With unsafe mode on you can try running any programming language added to PATH",
                        );
                ui.checkbox(&mut self.data.terminal_mode, "Terminal mode");
            }
            if self.data.terminal_mode {
                ui.label("You can use marcide to excecute terminal commands");
            }
            ui.separator();
            ui.label(RichText::from("Configuration").size(20.0));
            ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                if ui.button("Export").clicked() {
                    let files = FileDialog::new()
                        .set_title("Save as")
                        .add_filter("Marcide config", &["marcfg"])
                        .set_directory("/")
                        .save_file();
                    if files.clone().is_some() {
                        let config = format!(
                            "{}\n{}\n{}\n{}\n{}\n{}\n{}\nMARCIDE_CONFIG",
                            self.data.window_options_always_on_top,
                            self.data.auto_save,
                            self.data.auto_save_to_ram,
                            self.data.language,
                            self.data.is_gui_development,
                            self.data.terminal_mode,
                            self.data.unsafe_mode
                        );
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
                    let lines: Vec<&str> = contains.lines().collect();
                    //xd
                    if lines[7] == "MARCIDE_CONFIG" {
                        self.data.window_options_always_on_top = trueorfalse(lines[0].to_owned());
                        self.data.auto_save = trueorfalse(lines[1].to_owned());
                        self.data.auto_save_to_ram = trueorfalse(lines[2].to_owned());
                        self.data.language = lines[3].to_owned();
                        self.data.is_gui_development = trueorfalse(lines[4].to_owned());
                        self.data.terminal_mode = trueorfalse(lines[5].to_owned());
                        self.data.unsafe_mode = trueorfalse(lines[6].to_owned());
                    }
                    
                };
            });
            ui.separator();
            ui.label(RichText::from("Visuals").size(20.0));
            egui::widgets::global_dark_light_mode_switch(ui);
        } else {
            //infinite terminals
        }

        //tab 1 == code editor tab 2 == terminal
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        if *(tab) == 1 {
            "Terminal".to_string().into()
        } else if *(tab) == 2 {
            "Code editor".to_string().into()
        } else if *(tab) == 3 {
            "Find".to_string().into()
        } else if *(tab) == 4 {
            "Output".to_string().into()
        } else if *(tab) == 5 {
            "Settings".to_string().into()
        } else {
            format!("Tab {}", tab).into()
        }
    }
}

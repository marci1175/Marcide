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
            tree: Tree::new(vec![1, 2]),
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


        //tab 1 == code editor tab 2 == terminal
        if *tab == 2 {
            let frame_rect = dbg!(ui.max_rect());
            egui::CentralPanel::default().show(self.ctx, |ui| {
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
/*
    
    // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    
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
            
            ui.label(RichText::from("Open a file using `CTRL + O` or using the open option.").size(30.).color(Color32::LIGHT_RED));


        
        });
    } */
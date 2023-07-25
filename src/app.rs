use std::sync::mpsc::{self};
use std::path::PathBuf;
use std::time::Duration;
use rfd::FileDialog;

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_S, VK_CONTROL};
use std::fs::OpenOptions;
use self::code_editor::CodeEditor;
use std::io::{Write, Read};
use std::fs::File;
use chrono::Utc;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
 // if we add new fields, give them default values when deserializing old state
mod code_editor;
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    #[serde(skip)]
    settings_window_is_open: bool,
    auto_save: bool,
    #[serde(skip)]
    text: String,
    language: String,
    #[serde(skip)]
    code_editor: CodeEditor,
    #[serde(skip)]
    last_save_path: Option<PathBuf>,

    auto_save_interval: u64,
    #[serde(skip)]
    autosave_sender: Option<mpsc::Sender<String>>,

    #[serde(skip)]
    session_started: chrono::DateTime<Utc>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            session_started: Utc::now(),
            settings_window_is_open: false,
            auto_save: true,
            text: String::new(),
            language: "rs".into(),
            code_editor: CodeEditor::default(),
            last_save_path: None,
            auto_save_interval: 15,
            autosave_sender: None,
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
fn openfile(path : Option<PathBuf>) -> String {
    let mut contents = String::new();
        if let Some(file_path) = path {
            let mut file = File::open(file_path)
                .expect("Failed to open file");

            file.read_to_string(&mut contents)
                .expect("Failed to read file");
        }

        return contents;
}
fn savetofile(path : Option<PathBuf>, text : String){
        if let Some(file_path) = path {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(file_path.clone())
                .expect("Failed to open file");
            
            //pushback info
            println!("\n{}\n", text);
            // Write some data to the file
            match write!(file ,"{}", text){
                Ok(_) => {},
                Err(e) => {
                    println!("Error opening the file : {}", e);
                }
            }
    }
}
impl eframe::App for TemplateApp {
    
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        
        eframe::set_value(storage, eframe::APP_KEY, self);
        
    }
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //hotkeys
        let has_focus = ctx.input(|i| i.focused);
        let ctrlimput = unsafe {
            GetAsyncKeyState(VK_CONTROL as i32)
        };
        let ctrlis_pressed = (ctrlimput as u16 & 0x8000) != 0;
        //listen if ENTER key is pressed so we can send the message, except when r or l shift is pressed
        let sinp = unsafe {
             GetAsyncKeyState(VK_S as i32)
        };
        let sis_pressed = (sinp as u16 & 0x8000) != 0;

        //save hotkey
        if sis_pressed && ctrlis_pressed && has_focus {
            if self.last_save_path.is_some() {
                savetofile(self.last_save_path.clone(), self.text.clone());
            }
            else {
                let files = FileDialog::new()
                            .set_title("Save as")
                            .set_directory("/")
                            .save_file();
                    if files.clone().is_some(){
                        self.last_save_path = files.clone();
                        savetofile(files.clone(), self.text.clone());
                    }
            }
        }
        self.text = self.code_editor.code.clone();
        self.code_editor.language = self.language.clone();
        //autosave implementation
        if self.last_save_path.is_some() {
            let place = self.last_save_path.clone();
            //define recv sender
            let tx = self.autosave_sender.get_or_insert_with(||{
                let (tx,rx) = mpsc::channel::<String>();
                std::thread::spawn(move || loop {
                    //reciver, text always gets updated
                    match rx.try_recv(){
                        Ok(text) => {println!("RECV : {}", text);savetofile(place.clone(), text.clone())},
                        Err(err) => {println!("{}", err)}
                    };
                    std::thread::sleep(Duration::from_secs(15));
                });
                tx
            });
            
            match tx.send(self.code_editor.code.clone()){
                Ok(ok) => {ok},
                Err(_) => {}
            };
        }
        if self.settings_window_is_open{
            egui::Window::new("Settings")
                .open(&mut self.settings_window_is_open)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::from("File handling").size(20.0));
                    ui.checkbox(&mut self.auto_save, "Autosave");
                    if self.auto_save {
                        ui.label("Autosave will only turn on after you have saved your file somewhere.");
                    }
                    ui.separator();
                    ui.label(egui::RichText::from("Syntaxing").size(20.0));
                    ui.text_edit_singleline(&mut self.language);
                    if self.language == "quaran" || self.language == "Quaran" || self.language == "Korán" || self.language == "korán" {
                        ui.hyperlink_to("Quaran", "https://mek.oszk.hu/06500/06534/06534.pdf");
                    }
                });
        }
        egui::TopBottomPanel::top("Settings").show(ctx, |ui|{
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
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui|{
                if ui.button("Open").clicked() {
                    let files = FileDialog::new()
                        .set_title("Open")
                        .set_directory("/")
                        .pick_file();
                    if files.clone().is_some(){
                        self.last_save_path = files.clone();
                        self.code_editor.code = openfile(files);
                    }

                    
                }
                if ui.button("Save as").clicked(){
                    let files = FileDialog::new()
                            .set_title("Save as")
                            .set_directory("/")
                            .save_file();
                    if files.clone().is_some(){
                        self.last_save_path = files.clone();
                        savetofile(files.clone(), self.text.clone());
                    }
                    
                }
                if ui.button("Save").clicked(){
                    if Some(self.last_save_path.clone()).is_some(){
                        savetofile(self.last_save_path.clone(), self.text.clone())
                    }
                    
                }
                if ui.button("Settings").clicked(){
                    self.settings_window_is_open = !self.settings_window_is_open;
                }
            });
            
        });
        egui::TopBottomPanel::bottom("Stats").show(ctx, |ui|{
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui|{
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
                let current_datetime = chrono::Utc::now();
                let datetime_str = current_datetime.format("%H:%M:%S ").to_string();
                let sessiondate_str = self.session_started.format("%H:%M:%S ").to_string();
                ui.label(format!("Session started : {}", sessiondate_str));
                ui.label(format!("Current time : {}", datetime_str));
                ctx.request_repaint();
            });
        });
        egui::CentralPanel::default().show(ctx, |ui|{
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui|{
                    code_editor::CodeEditor::show(&mut self.code_editor, "id".into(), ui, egui::vec2(0.0, 0.0));
                });
                
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

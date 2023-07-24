use std::path::PathBuf;
use std::time::Duration;
use rfd::FileDialog;
use egui::{Sense, Ui};
use std::fs::OpenOptions;
use self::code_editor::CodeEditor;
use std::io::{Write, Read};
use std::fs::File;

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
    last_save_path: PathBuf,

    auto_save_interval: u64,
    #[serde(skip)]
    thread_is_running: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            settings_window_is_open: false,
            auto_save: true,
            text: String::new(),
            language: "rs".into(),
            code_editor: CodeEditor::default(),
            last_save_path: PathBuf::default(),
            auto_save_interval: 15,
            thread_is_running: false,
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
                .open(file_path)
                .expect("Failed to open file");
            
            //pushback info

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
        self.text = self.code_editor.code.clone();
        self.code_editor.language = self.language.clone();
        //autosave implementation
        if self.last_save_path.is_file(){
            let amount_of_sleep = self.auto_save_interval;
            let can_run = self.auto_save;
            let text = self.text.clone();
            let place = Some(self.last_save_path.clone());
            if !self.thread_is_running {
                println!("it has started");
                std::thread::spawn(move || loop {
                    if !can_run{
                        break;
                    }
                    else {
                        savetofile(place.clone(), text.clone());
                    }
                    std::thread::sleep(Duration::from_secs(amount_of_sleep));
                });
                self.thread_is_running = true;
                if !can_run {
                    self.thread_is_running = false;
                }
            }
            
        }
        if self.settings_window_is_open{
            egui::Window::new("Settings")
                .open(&mut self.settings_window_is_open)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::from("File handling").size(20.0));
                    ui.checkbox(&mut self.auto_save, "Autosave");
                    ui.separator();
                    ui.label(egui::RichText::from("Syntaxing").size(20.0));
                    ui.text_edit_singleline(&mut self.language);
                });
        }
        egui::TopBottomPanel::top("Settings").show(ctx, |ui|{
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui|{
                if ui.button("Open").clicked() {
                    let files = FileDialog::new()
                        .set_title("Open")
                        .set_directory("/")
                        .pick_file();
                    if files.clone().is_some(){
                        self.last_save_path = files.clone().unwrap();
                        self.code_editor.code = openfile(files);
                    }

                    
                }
                if ui.button("Save as").clicked(){
                    let files = FileDialog::new()
                            .set_title("Save as")
                            .add_filter("", &["txt"])
                            .set_directory("/")
                            .save_file();
                    if files.clone().is_some(){
                        self.last_save_path = files.clone().unwrap();
                        savetofile(files.clone(), self.text.clone());
                    }
                    
                }
                if ui.button("Save").clicked(){
                    if Some(self.last_save_path.clone()).is_some(){
                        savetofile(Some(self.last_save_path.clone()), self.text.clone())
                    }
                    
                }
                if ui.button("Settings").clicked(){
                    self.settings_window_is_open = !self.settings_window_is_open;
                }
            });
            
        });
        egui::TopBottomPanel::bottom("Stats").show(ctx, |ui|{
            //implement statistics

        });
        egui::CentralPanel::default().show(ctx, |ui|{
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui|{
                    ui.add_sized(ui.available_size(), |ui:  &mut Ui|{
                        let code_editor = code_editor::CodeEditor::show(&mut self.code_editor, "id".into(), ui, egui::vec2(0.0, 0.0));
                        ui.allocate_response( code_editor, Sense::click())
                    });
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


/// We derive Deserialize/Serialize so we can persist app state on shutdown.
 // if we add new fields, give them default values when deserializing old state
mod code_editor;
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    text: String,
    language: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            text: String::new(),
            language: "rs".into(),
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

impl eframe::App for TemplateApp {
    
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        
        eframe::set_value(storage, eframe::APP_KEY, self);
        
    }
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui|{
            egui::ScrollArea::vertical().show(ui, |ui|{
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui|{
                    ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut self.text)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .lock_focus(true)
                    .desired_width(f32::INFINITY))
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

use eframe::egui;
use egui::Vec2;
use egui_terminal::{TermHandler, Terminal};

pub fn new(style: &mut TermHandler, size: Vec2) -> Terminal<'_> {
    egui_terminal::Terminal::new(style)
}

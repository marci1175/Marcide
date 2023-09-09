use egui_terminal::{TermHandler, Terminal};

pub fn new(style: &mut TermHandler) -> Terminal<'_> {
    egui_terminal::Terminal::new(style)
}

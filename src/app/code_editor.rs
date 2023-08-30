// ----------------------------------------------------------------------------
use egui::{FontId, FontFamily, Pos2, pos2, Rect};
use egui::text::LayoutJob;
use egui::{vec2, Color32, FontSelection, Id, Layout, Rounding, Stroke, Vec2, Align};

use serde::{Deserialize, Serialize};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_2, /*shft + vk_oem_3 = ( | shft + vk_oem_4 = { */
    /*[*/ VK_8, /*""*/ VK_B, /*(*/ VK_F, VK_RMENU, VK_SHIFT,
};

/// Memoized Code highlighting
pub fn highlight(ctx: &egui::Context, theme: &CodeTheme, code: &str, language: &str) -> LayoutJob {
    impl egui::util::cache::ComputerMut<(&CodeTheme, &str, &str), LayoutJob> for Highlighter {
        fn compute(&mut self, (theme, code, lang): (&CodeTheme, &str, &str)) -> LayoutJob {
            self.highlight(theme, code, lang)
        }
    }

    type HighlightCache = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    ctx.memory_mut(|memory| {
        let highlight_cache = memory.caches.cache::<HighlightCache>();
        highlight_cache.get((theme, code, language))
    })
}

// ----------------------------------------------------------------------------

#[derive(Clone, Copy, Hash, PartialEq, Deserialize, Serialize)]
enum SyntectTheme {
    Base16EightiesDark,
    Base16MochaDark,
    Base16OceanDark,
    Base16OceanLight,
    InspiredGitHub,
    SolarizedDark,
    SolarizedLight,
}

impl SyntectTheme {
    /*
    fn all() -> impl ExactSizeIterator<Item = Self> {
        [
            Self::Base16EightiesDark,
            Self::Base16MochaDark,
            Self::Base16OceanDark,
            Self::Base16OceanLight,
            Self::InspiredGitHub,
            Self::SolarizedDark,
            Self::SolarizedLight,
        ]
        .iter()
        .copied()
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Base16EightiesDark => "Base16 Eighties (dark)",
            Self::Base16MochaDark => "Base16 Mocha (dark)",
            Self::Base16OceanDark => "Base16 Ocean (dark)",
            Self::Base16OceanLight => "Base16 Ocean (light)",
            Self::InspiredGitHub => "InspiredGitHub (light)",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
        }
    }
     */
    fn syntect_key_name(&self) -> &'static str {
        match self {
            Self::Base16EightiesDark => "base16-eighties.dark",
            Self::Base16MochaDark => "base16-mocha.dark",
            Self::Base16OceanDark => "base16-ocean.dark",
            Self::Base16OceanLight => "base16-ocean.light",
            Self::InspiredGitHub => "InspiredGitHub",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
        }
    }
    /*
    pub fn is_dark(&self) -> bool {
        match self {
            Self::Base16EightiesDark
            | Self::Base16MochaDark
            | Self::Base16OceanDark
            | Self::SolarizedDark => true,

            Self::Base16OceanLight | Self::InspiredGitHub | Self::SolarizedLight => false,
        }
    }
    */
}

#[derive(Clone, Hash, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct CodeTheme {
    dark_mode: bool,
    syntect_theme: SyntectTheme,
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl CodeTheme {
    /*
    pub fn from_style(style: &egui::Style) -> Self {
        if style.visuals.dark_mode {
            Self::dark()
        } else {
            Self::light()
        }
    }
    */
    pub fn from_memory(ctx: &egui::Context) -> Self {
        if ctx.style().visuals.dark_mode {
            ctx.data_mut(|data| {
                data.get_persisted(egui::Id::new("dark"))
                    .unwrap_or_else(|| CodeTheme::dark())
            })
        } else {
            ctx.data_mut(|data| {
                data.get_persisted(egui::Id::new("light"))
                    .unwrap_or_else(|| CodeTheme::light())
            })
        }
    }
}

impl CodeTheme {
    pub fn dark() -> Self {
        Self {
            dark_mode: true,
            syntect_theme: SyntectTheme::Base16MochaDark,
        }
    }

    pub fn light() -> Self {
        Self {
            dark_mode: false,
            syntect_theme: SyntectTheme::SolarizedLight,
        }
    }
}

// ----------------------------------------------------------------------------

struct Highlighter {
    ps: syntect::parsing::SyntaxSet,
    ts: syntect::highlighting::ThemeSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self {
            ps: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            ts: syntect::highlighting::ThemeSet::load_defaults(),
        }
    }
}

impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: &CodeTheme, code: &str, lang: &str) -> LayoutJob {
        self.highlight_impl(theme, code, lang).unwrap_or_else(|| {
            // Fallback:
            LayoutJob::simple(
                code.into(),
                egui::FontId::monospace(12.0),
                if theme.dark_mode {
                    egui::Color32::LIGHT_GRAY
                } else {
                    egui::Color32::DARK_GRAY
                },
                f32::INFINITY,
            )
        })
    }

    fn highlight_impl(&self, theme: &CodeTheme, text: &str, language: &str) -> Option<LayoutJob> {
        use syntect::easy::HighlightLines;
        use syntect::highlighting::FontStyle;
        use syntect::util::LinesWithEndings;

        let syntax = self
            .ps
            .find_syntax_by_name(language)
            .or_else(|| self.ps.find_syntax_by_extension(language))?;

        let theme = theme.syntect_theme.syntect_key_name();
        let mut h = HighlightLines::new(syntax, &self.ts.themes[theme]);

        use egui::text::{LayoutSection, TextFormat};

        let mut job = LayoutJob {
            text: text.into(),
            ..Default::default()
        };

        for line in LinesWithEndings::from(text) {
            for (style, range) in h.highlight_line(line, &self.ps).ok()? {
                let fg = style.foreground;
                let text_color = egui::Color32::from_rgb(fg.r, fg.g, fg.b);
                let italics = style.font_style.contains(FontStyle::ITALIC);
                let underline = style.font_style.contains(FontStyle::ITALIC);
                let underline = if underline {
                    egui::Stroke::new(1.0, text_color)
                } else {
                    egui::Stroke::NONE
                };
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: as_byte_range(text, range),
                    format: TextFormat {
                        font_id: egui::FontId::monospace(12.0),
                        color: text_color,
                        italics,
                        underline,
                        ..Default::default()
                    },
                });
            }
        }

        Some(job)
    }
}

fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CodeEditor {
    pub language: String,
    pub code: String,
    bracket_is_held: bool,
    curlybracket_is_held: bool,
    quote_is_held: bool,
    sbracket_is_held: bool,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            language: "py".into(),
            code: "".into(),
            bracket_is_held: false,
            curlybracket_is_held: false,
            quote_is_held: false,
            sbracket_is_held: false,
        }
    }
}

impl CodeEditor {
    pub fn show(
        &mut self,
        id: Id,
        ui: &mut egui::Ui,
        mut scroll_offset: Vec2,
        go_to_offset: bool,
    ) -> Vec2 {
        let Self {
            language,
            code,
            bracket_is_held: _,
            curlybracket_is_held: _,
            quote_is_held: _,
            sbracket_is_held: _,
        } = self;
        
        let rect_size = ui.available_size();
        let rect_pos = pos2(10., 20.);
        let rect = Rect::from_min_size(rect_pos, rect_size);

        let frame_rect = ui.max_rect().shrink(5.0);
        let code_rect = frame_rect.shrink(10.0);

        let theme = CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = highlight(ui.ctx(), &theme, string, language);
            layout_job.wrap.max_width = wrap_width;
            //ui.fonts().layout_job(layout_job)
            ui.fonts(|fonts| fonts.layout_job(layout_job))
        };
        
        ui.allocate_space(ui.available_size());
         
        ui.painter().rect(
            frame_rect,
            Rounding::same(5.0),
            Color32::BLACK,
            Stroke::NONE,
        );
        
        let mut frame_ui = ui.child_ui(code_rect, Layout::default());

        // get how many rows it takes to fill up our max rect
        let font_id = FontSelection::default().resolve(ui.style());
        let row_height = ui.fonts(|fonts| fonts.row_height(&font_id));
        let rows = ((code_rect.height() - 5.0) / row_height).floor() as usize;
        //full retard
        let code_ref = code.clone();
        /*if go_to_offset {
            scroll_offset[1] = scroll_offset[1] * row_height;
        }
        */
        let text_widget = egui::TextEdit::multiline(code)
            .font(FontSelection::FontId(FontId::new(10.0, FontFamily::Monospace))) // for cursor height
            .code_editor()
            // remove the frame and draw our own
            .frame(false)
            .desired_width(ui.available_width())
            .margin(vec2(2.0, 2.0))
            .layouter(&mut layouter)
            .id(id)
            .desired_rows(rows);
        
        let mut scroll_res = egui::ScrollArea::vertical()
            .id_source("code editor")
            .stick_to_bottom(true)
            .scroll_offset(scroll_offset)
            .show(&mut frame_ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Min), |ui|{
                    let mut numbers: String = String::new();
                    for i in 1..=code_ref.clone().lines().count() {
                        numbers.push_str(&i.to_string());
                        numbers.push('\n');
                    }
                    
                    ui.allocate_ui(vec2(25., row_height), |ui|{
                        ui.add(egui::Label::new(egui::RichText::from(numbers.to_string()).font(FontId::new(12., FontFamily::Monospace))));
                    });
                    
                    ui.add(text_widget);
                    
                });
                
                
            });
        //finder is on
        if go_to_offset {
            scroll_res.state.offset[1] = scroll_offset.clone()[1] * row_height;
        }
        
        let eightinput = unsafe { GetAsyncKeyState(VK_8 as i32) };
        let eight_is_pressed = (eightinput as u16 & 0x8000) != 0;
        let twoinput = unsafe { GetAsyncKeyState(VK_2 as i32) };
        let two_is_pressed = (twoinput as u16 & 0x8000) != 0;
        let fimput = unsafe { GetAsyncKeyState(VK_F as i32) };
        let fis_pressed = (fimput as u16 & 0x8000) != 0;
        let shiftimput = unsafe { GetAsyncKeyState(VK_SHIFT as i32) };
        let shift_is_pressed = (shiftimput as u16 & 0x8000) != 0;
        let altimput = unsafe { GetAsyncKeyState(VK_RMENU as i32) };
        let alt_is_pressed = (altimput as u16 & 0x8000) != 0;
        let binput = unsafe { GetAsyncKeyState(VK_B as i32) };
        let b_is_pressed = (binput as u16 & 0x8000) != 0;
        /*shft + vk_oem_3 = ( | shft + vk_oem_4 = { */
        let has_focus = ui.input(|i| i.focused);
        if has_focus {
            if shift_is_pressed && eight_is_pressed && !self.sbracket_is_held {
                simulate::type_str(")").unwrap();
                match simulate::release(simulate::Key::Shift) {
                    Ok(_) => {}
                    Err(_) => {
                        println!(r#"Paniced at ) trying to send )"#)
                    }
                };
                match simulate::send(simulate::Key::Left) {
                    Ok(_) => {}
                    Err(_) => {
                        println!(r#"Paniced at ) trying to send <-"#)
                    }
                };
                self.sbracket_is_held = true;
            } else if !(shift_is_pressed && eight_is_pressed) {
                self.sbracket_is_held = false;
            }

            if shift_is_pressed && two_is_pressed && !self.quote_is_held {
                simulate::type_str(r#"""#).unwrap();
                match simulate::release(simulate::Key::Shift) {
                    Ok(_) => {}
                    Err(_) => {
                        println!(r#"Paniced at " trying to send " "#)
                    }
                };
                match simulate::send(simulate::Key::Left) {
                    Ok(_) => {}
                    Err(_) => {
                        println!(r#"Paniced at " trying to send <-"#)
                    }
                };
                self.quote_is_held = true;
            } else if !(shift_is_pressed && two_is_pressed) {
                self.quote_is_held = false;
            }

            if alt_is_pressed && b_is_pressed && !self.curlybracket_is_held {
                simulate::type_str("}").unwrap();
                match simulate::release(simulate::Key::Shift) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Paniced at curlybracket trying to send curlybracket")
                    }
                };
                match simulate::send(simulate::Key::Left) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Paniced at curlybracket trying to send <-")
                    }
                };
                self.curlybracket_is_held = true;
            } else if !(alt_is_pressed && b_is_pressed) {
                self.curlybracket_is_held = false;
            }

            if alt_is_pressed && fis_pressed && !self.bracket_is_held {
                simulate::type_str("]").unwrap();
                match simulate::release(simulate::Key::Shift) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Paniced at ] trying to send ]")
                    }
                };
                match simulate::send(simulate::Key::Left) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Paniced at ] trying to send <-")
                    }
                };
                self.bracket_is_held = true;
            } else if !(alt_is_pressed && fis_pressed) {
                self.bracket_is_held = false;
            }
            if shift_is_pressed {
                match simulate::press(simulate::Key::Shift) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Paniced at <- trying to send <-")
                    }
                };
            }
        }
         
        scroll_res.state.offset
        
    }
}

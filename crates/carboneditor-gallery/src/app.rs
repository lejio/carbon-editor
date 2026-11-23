//! Gallery chrome: one editor on a sample buffer.

use carboneditor::{Buffer, Editor, Mode, Theme};
use egui::{Frame, Margin, RichText};

pub struct GalleryApp {
    mode: Mode,
    wrap: bool,
    buffer: Buffer,
}

impl GalleryApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mode = Mode::Dark;
        mode.theme().install(&cc.egui_ctx);
        Self {
            mode,
            wrap: true,
            buffer: Buffer::from_text(SAMPLE_MAIN),
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let theme = Theme::of(ui);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.label(
                RichText::new("CarbonEditor")
                    .strong()
                    .size(16.0)
                    .color(theme.text),
            );
            ui.checkbox(&mut self.wrap, "Word wrap");
            let mode_label = match self.mode {
                Mode::Dark => "Light",
                Mode::Light => "Dark",
            };
            if ui.button(mode_label).clicked() {
                self.mode = self.mode.toggled();
            }
        });
    }
}

impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.mode.theme().install(ui.ctx());

        let theme = Theme::of(ui);
        egui::Panel::top("carboneditor.toolbar")
            .frame(
                Frame::new()
                    .fill(theme.panel)
                    .inner_margin(Margin::symmetric(12, 8)),
            )
            .show_inside(ui, |ui| self.toolbar(ui));

        egui::CentralPanel::default()
            .frame(Frame::new().fill(theme.bg).inner_margin(Margin::same(12)))
            .show_inside(ui, |ui| {
                Editor::new(&mut self.buffer)
                    .path("main.rs")
                    .wrap(self.wrap)
                    .show(ui);
            });
    }
}

const SAMPLE_MAIN: &str = r#"fn main() {
    // The basic text theme colors keywords, strings, numbers, and comments.
    let message = "Word wrap keeps this sentence inside the editor width, then continues it on the next visual row. Toggle Word wrap to scroll the same line sideways.";
    let count = 1;
    println!("{message} ({count})");
}
"#;

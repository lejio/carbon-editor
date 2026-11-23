//! Dark and light themes shared by the editor and the project tree.

use egui::{Color32, CornerRadius, Margin, Stroke, Visuals};

use crate::font;

const THEME_ID: &str = "carboneditor.theme";

/// Which palette [`Theme`] installs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Dark,
    Light,
}

impl Mode {
    pub fn toggled(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }

    pub fn theme(self) -> Theme {
        match self {
            Self::Dark => Theme::dark(),
            Self::Light => Theme::light(),
        }
    }
}

/// Colors for source text.
///
/// [`TextTheme::basic`] is the highlighter palette: keywords, strings, numbers,
/// and comments. Plain text follows the editor theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextTheme {
    /// Unstyled source text.
    pub plain: Color32,
    /// `//` and `/* */` comments.
    pub comment: Color32,
    /// Strings and character literals.
    pub string: Color32,
    /// Language keywords.
    pub keyword: Color32,
    /// Numeric literals.
    pub number: Color32,
}

impl TextTheme {
    /// Basic highlighter colors that sit on this editor theme.
    pub fn basic(theme: Theme) -> Self {
        let (comment, string, keyword, number) = match theme.mode {
            Mode::Dark => (
                Color32::from_rgb(106, 115, 130),
                Color32::from_rgb(152, 195, 121),
                Color32::from_rgb(198, 149, 232),
                Color32::from_rgb(230, 176, 110),
            ),
            Mode::Light => (
                Color32::from_rgb(120, 126, 138),
                Color32::from_rgb(24, 128, 64),
                Color32::from_rgb(121, 72, 184),
                Color32::from_rgb(168, 96, 16),
            ),
        };
        Self {
            plain: theme.text,
            comment,
            string,
            keyword,
            number,
        }
    }
}

/// Colors and radii for CarbonEditor.
///
/// Install one on the egui context before drawing. Widgets read it back from
/// egui memory.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub mode: Mode,
    /// Application background.
    pub bg: Color32,
    /// Toolbars and side panels.
    pub panel: Color32,
    /// Raised surfaces such as buttons.
    pub raised: Color32,
    /// Editor page.
    pub sunken: Color32,
    /// Hairline borders.
    pub stroke: Color32,
    /// Hover and focus borders.
    pub stroke_strong: Color32,
    /// Primary text.
    pub text: Color32,
    /// Secondary text, including inactive line numbers.
    pub text_weak: Color32,
    /// Accent used for focus, selection, and the modified marker.
    pub accent: Color32,
    /// Text drawn on top of [`Self::accent`].
    pub on_accent: Color32,
    /// Line-number column.
    pub gutter: Color32,
    /// Current line in the gutter.
    pub current_line: Color32,
    /// Corner radius for the editor frame.
    pub radius: u8,
}

impl Theme {
    /// Dark graphite theme, aligned with CarbonUI's graphite palette.
    pub fn dark() -> Self {
        Self {
            mode: Mode::Dark,
            bg: Color32::from_rgb(16, 17, 20),
            panel: Color32::from_rgb(24, 26, 30),
            raised: Color32::from_rgb(32, 34, 40),
            sunken: Color32::from_rgb(14, 15, 18),
            stroke: Color32::from_rgb(58, 62, 72),
            stroke_strong: Color32::from_rgb(110, 118, 134),
            text: Color32::from_rgb(226, 228, 234),
            text_weak: Color32::from_rgb(148, 154, 168),
            accent: Color32::from_rgb(86, 156, 214),
            on_accent: Color32::from_rgb(246, 249, 252),
            gutter: Color32::from_rgb(20, 21, 25),
            current_line: Color32::from_rgb(32, 42, 56),
            radius: 6,
        }
    }

    /// Paper theme for the same widgets.
    pub fn light() -> Self {
        Self {
            mode: Mode::Light,
            bg: Color32::from_rgb(236, 236, 234),
            panel: Color32::from_rgb(250, 250, 248),
            raised: Color32::from_rgb(255, 255, 255),
            sunken: Color32::from_rgb(255, 255, 255),
            stroke: Color32::from_rgb(214, 214, 210),
            stroke_strong: Color32::from_rgb(150, 152, 160),
            text: Color32::from_rgb(32, 33, 36),
            text_weak: Color32::from_rgb(110, 112, 120),
            accent: Color32::from_rgb(36, 110, 176),
            on_accent: Color32::from_rgb(248, 250, 252),
            gutter: Color32::from_rgb(246, 246, 244),
            current_line: Color32::from_rgb(226, 236, 246),
            radius: 6,
        }
    }

    /// Store this theme, push its colors into egui, and register Lyth Mono.
    ///
    /// Safe to call on every frame. The font bytes are registered once.
    pub fn install(self, ctx: &egui::Context) {
        ctx.data_mut(|data| data.insert_temp(egui::Id::new(THEME_ID), self));
        font::install_font(ctx);
        let visuals = self.visuals();
        ctx.all_styles_mut(|style| {
            style.visuals = visuals.clone();
            style.spacing.button_padding = egui::vec2(12.0, 6.0);
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.interact_size.y = 28.0;
            style.spacing.window_margin = Margin::same(8);
            style.spacing.menu_margin = Margin::same(6);
        });
    }

    /// Theme previously installed on this context, or [`Theme::dark`].
    pub fn from_ctx(ctx: &egui::Context) -> Self {
        ctx.data(|data| data.get_temp(egui::Id::new(THEME_ID)))
            .unwrap_or_else(Self::dark)
    }

    /// Theme for the context this `Ui` belongs to.
    pub fn of(ui: &egui::Ui) -> Self {
        Self::from_ctx(ui.ctx())
    }

    /// Basic text colors for this editor theme.
    pub fn text_theme(self) -> TextTheme {
        TextTheme::basic(self)
    }

    /// egui visuals matching this theme, so stock widgets sit next to the editor.
    pub fn visuals(&self) -> Visuals {
        let mut visuals = match self.mode {
            Mode::Dark => Visuals::dark(),
            Mode::Light => Visuals::light(),
        };
        visuals.weak_text_alpha = 0.62;
        visuals.hyperlink_color = self.accent;
        visuals.faint_bg_color = mix(self.bg, self.raised, 0.45);
        visuals.extreme_bg_color = self.sunken;
        visuals.code_bg_color = self.sunken;
        visuals.window_fill = self.panel;
        visuals.window_stroke = Stroke::new(1.0_f32, self.stroke);
        visuals.window_corner_radius = CornerRadius::same(self.radius);
        visuals.menu_corner_radius = CornerRadius::same(self.radius);
        visuals.panel_fill = self.bg;
        visuals.selection.bg_fill = with_alpha(self.accent, 56);
        visuals.selection.stroke = Stroke::new(1.0_f32, self.accent);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.text);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, self.stroke);
        visuals.widgets.noninteractive.bg_fill = self.bg;
        visuals.widgets.noninteractive.weak_bg_fill = self.panel;
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(self.radius);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.text);
        visuals.widgets.inactive.bg_fill = self.raised;
        visuals.widgets.inactive.weak_bg_fill = self.raised;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, self.stroke);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(self.radius);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, self.text);
        visuals.widgets.hovered.bg_fill = mix(self.raised, self.text, 0.08);
        visuals.widgets.hovered.weak_bg_fill = mix(self.raised, self.text, 0.08);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, self.stroke_strong);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(self.radius);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, self.on_accent);
        visuals.widgets.active.bg_fill = self.accent;
        visuals.widgets.active.weak_bg_fill = self.accent;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, self.accent);
        visuals.widgets.active.corner_radius = CornerRadius::same(self.radius);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, self.text);
        visuals.widgets.open.bg_fill = self.panel;
        visuals.widgets.open.weak_bg_fill = self.panel;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, self.stroke);
        visuals.widgets.open.corner_radius = CornerRadius::same(self.radius);
        visuals
    }
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let channel = |from: u8, to: u8| -> u8 {
        (f32::from(from) + (f32::from(to) - f32::from(from)) * t).round() as u8
    };
    Color32::from_rgb(
        channel(a.r(), b.r()),
        channel(a.g(), b.g()),
        channel(a.b(), b.b()),
    )
}

fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    let [r, g, b, _] = color.to_array();
    Color32::from_rgba_unmultiplied(r, g, b, alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_roundtrip() {
        let ctx = egui::Context::default();
        Theme::light().install(&ctx);
        assert_eq!(Theme::from_ctx(&ctx).mode, Mode::Light);
        assert_eq!(Theme::from_ctx(&ctx).bg, Theme::light().bg);
        Theme::dark().install(&ctx);
        assert_eq!(Theme::from_ctx(&ctx).accent, Theme::dark().accent);
    }

    #[test]
    fn basic_text_theme_follows_the_editor_theme() {
        let dark = Theme::dark();
        let light = Theme::light();
        assert_eq!(dark.text_theme().plain, dark.text);
        assert_eq!(light.text_theme().plain, light.text);
        assert_ne!(dark.text_theme().keyword, light.text_theme().keyword);
        assert_ne!(dark.text_theme().keyword, dark.text_theme().string);
        assert_ne!(dark.text_theme().comment, dark.text_theme().number);
    }
}

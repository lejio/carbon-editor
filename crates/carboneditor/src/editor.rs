//! Multiline text editor with a gutter and optional word wrap.

use std::hash::Hash;

use egui::{
    pos2, Align, Color32, CornerRadius, FontId, Frame, Galley, Id, Layout, Margin, Rect,
    ScrollArea, Stroke, StrokeKind, TextBuffer, TextEdit, Ui, UiBuilder, Vec2,
};

use crate::font::{self, FONT_NAME, FONT_SIZE};
use crate::highlight;
use crate::text::{self, TextPos};
use crate::theme::Theme;
use crate::Buffer;

const HEADER: f32 = 32.0;
const FOOTER: f32 = 26.0;

/// Text editor for one [`Buffer`].
///
/// Word wrap breaks lines on the editor width. With wrap off, long lines keep
/// a single visual row and the page scrolls sideways.
pub struct Editor<'a> {
    buffer: &'a mut Buffer,
    path: Option<&'a str>,
    wrap: bool,
    read_only: bool,
    font_size: f32,
    id_salt: Option<Id>,
}

/// What [`Editor::show`] did this frame.
pub struct EditorOutput {
    pub response: egui::Response,
    pub changed: bool,
    pub cursor: TextPos,
    /// Visual rows after wrapping. A wrapped logical line counts as more than one.
    pub rows: usize,
    /// Width of the laid-out text, in points.
    pub text_width: f32,
}

impl<'a> Editor<'a> {
    pub fn new(buffer: &'a mut Buffer) -> Self {
        Self {
            buffer,
            path: None,
            wrap: true,
            read_only: false,
            font_size: FONT_SIZE,
            id_salt: None,
        }
    }

    /// Project-relative path drawn in the header. Without one, the header says "Untitled".
    pub fn path(mut self, path: &'a str) -> Self {
        self.path = Some(path);
        self
    }

    /// Wrap long lines to the editor width.
    ///
    /// When this is off, long lines scroll horizontally instead of breaking.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Editor font size in points. Clamped to a range Lyth Mono stays legible in.
    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size.clamp(10.0, 28.0);
        self
    }

    /// Extra id salt when more than one editor is on screen.
    pub fn id_salt(mut self, salt: impl Hash) -> Self {
        self.id_salt = Some(Id::new(salt));
        self
    }

    pub fn show(self, ui: &mut Ui) -> EditorOutput {
        let theme = Theme::of(ui);
        let Editor {
            buffer,
            path,
            wrap,
            read_only,
            font_size,
            id_salt,
        } = self;

        // `available_size()` is the cursor size in a wrapping layout, which is
        // only as wide as the previous widget. `allocate_exact_size` then
        // clamps to that. The rectangle before wrap is the column we were given.
        let bounds = ui.available_rect_before_wrap();
        let width = finite(bounds.width(), 720.0);
        let height = finite(bounds.height(), 480.0);
        let rect = Rect::from_min_size(bounds.min, Vec2::new(width, height));
        ui.advance_cursor_after_rect(rect);
        ui.painter().rect(
            rect,
            CornerRadius::same(theme.radius),
            theme.sunken,
            Stroke::NONE,
            StrokeKind::Inside,
        );

        let header = Rect::from_min_max(rect.min, pos2(rect.right(), rect.top() + HEADER));
        let footer = Rect::from_min_max(pos2(rect.left(), rect.bottom() - FOOTER), rect.max);
        let body = Rect::from_min_max(
            pos2(rect.left(), header.bottom()),
            pos2(rect.right(), footer.top()),
        );

        let line_count = text::logical_lines(buffer.text());
        let digits = line_count.to_string().len().max(2);
        let advance = ui.fonts_mut(|fonts| fonts.glyph_width(&font::monospace(font_size), '0'));
        let gutter_w = (advance * digits as f32 + 18.0).max(36.0);
        let gutter = Rect::from_min_max(body.min, pos2(body.left() + gutter_w, body.bottom()));
        let text_rect = Rect::from_min_max(pos2(gutter.right(), body.top()), body.max);

        ui.painter()
            .rect_filled(gutter, CornerRadius::ZERO, theme.gutter);
        rule(ui, header.bottom(), rect.x_range(), theme.stroke);
        rule(ui, footer.top(), rect.x_range(), theme.stroke);

        let text_id = ui.make_persistent_id((
            id_salt.unwrap_or_else(|| Id::new("carboneditor")),
            path.unwrap_or(""),
        ));
        let page_width = (text_rect.width() - 16.0).max(32.0);
        let text_theme = theme.text_theme();
        let color = text_theme.plain;
        let mut layouter = move |ui: &Ui, text: &dyn TextBuffer, wrap_width: f32| {
            // TextEdit reports the cursor width here when the surrounding layout
            // wraps, which is about one glyph. The page width is the column we
            // reserved next to the gutter.
            let max_width = if wrap {
                let reported = if wrap_width.is_finite() {
                    wrap_width
                } else {
                    page_width
                };
                reported.max(page_width)
            } else {
                f32::INFINITY
            };
            let job = highlight::job(
                text.as_str(),
                FontId::monospace(font_size),
                text_theme,
                max_width,
            );
            ui.fonts_mut(|fonts| fonts.layout_job(job))
        };

        // A wrapping parent reports `available_width()` as the cursor width, which
        // collapses a multiline editor to one character. A top-down layout reports
        // the rectangle we reserved for the page.
        let mut text_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(text_rect)
                .layout(Layout::top_down(Align::Min))
                .id_salt((text_id, "page")),
        );
        let scroll = if wrap {
            ScrollArea::vertical()
        } else {
            ScrollArea::both()
        };
        let scrolled = scroll
            .id_salt((text_id, "scroll"))
            .auto_shrink([false, false])
            .show(&mut text_ui, |ui| {
                let desired = if wrap { page_width } else { f32::INFINITY };
                if !wrap {
                    ui.set_min_width(page_width);
                }
                TextEdit::multiline(buffer.as_text_mut())
                    .code_editor()
                    .id(text_id)
                    .font(FontId::monospace(font_size))
                    .desired_rows(1)
                    .cursor_at_end(false)
                    .interactive(!read_only)
                    .text_color(color)
                    .frame(
                        Frame::new()
                            .fill(Color32::TRANSPARENT)
                            .inner_margin(Margin::symmetric(8, 4)),
                    )
                    .desired_width(desired)
                    .layouter(&mut layouter)
                    .show(ui)
            });
        let edit = scrolled.inner;

        let index = edit
            .cursor_range
            .or_else(|| edit.state.cursor.char_range())
            .map(|range| range.primary.index)
            .unwrap_or(0);
        let cursor = text::cursor_at(buffer.text(), index);
        paint_gutter(
            ui,
            gutter,
            &edit.galley,
            edit.galley_pos,
            cursor.line,
            font_size,
            &theme,
        );
        paint_header(
            ui,
            header,
            path.unwrap_or("Untitled"),
            buffer.is_dirty(),
            &theme,
        );
        paint_footer(ui, footer, cursor, wrap, line_count, &theme);

        let border = if edit.response.has_focus() {
            Stroke::new(1.0_f32, theme.accent)
        } else {
            Stroke::new(1.0_f32, theme.stroke)
        };
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(theme.radius),
            border,
            StrokeKind::Inside,
        );

        EditorOutput {
            changed: edit.response.changed(),
            response: edit.response.response,
            cursor,
            rows: edit.galley.rows.len().max(1),
            text_width: edit.galley.size().x,
        }
    }
}

fn finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.max(1.0)
    } else {
        fallback
    }
}

fn rule(ui: &Ui, y: f32, x: egui::Rangef, color: Color32) {
    ui.painter().hline(x, y, Stroke::new(1.0_f32, color));
}

fn paint_header(ui: &Ui, rect: Rect, path: &str, dirty: bool, theme: &Theme) {
    let painter = ui.painter_at(rect);
    let path_color = if path == "Untitled" {
        theme.text_weak
    } else {
        theme.text
    };
    let galley = painter.layout_no_wrap(path.to_owned(), FontId::proportional(13.0), path_color);
    let y = rect.center().y - galley.size().y * 0.5;
    painter.galley(pos2(rect.left() + 12.0, y), galley, path_color);
    if dirty {
        let mark = painter.layout_no_wrap(
            "Modified".to_owned(),
            FontId::proportional(12.0),
            theme.accent,
        );
        let x = rect.right() - 12.0 - mark.size().x;
        painter.galley(
            pos2(x, rect.center().y - mark.size().y * 0.5),
            mark,
            theme.accent,
        );
    }
}

fn paint_footer(ui: &Ui, rect: Rect, cursor: TextPos, wrap: bool, lines: usize, theme: &Theme) {
    let wrap = if wrap { "Wrap" } else { "No wrap" };
    let status = format!(
        "Ln {}, Col {}    {}    {} {}",
        cursor.line,
        cursor.column,
        wrap,
        lines,
        if lines == 1 { "line" } else { "lines" },
    );
    let painter = ui.painter_at(rect);
    let galley = painter.layout_no_wrap(status, FontId::proportional(12.0), theme.text_weak);
    let y = rect.center().y - galley.size().y * 0.5;
    painter.galley(pos2(rect.left() + 12.0, y), galley, theme.text_weak);

    let font = painter.layout_no_wrap(
        FONT_NAME.to_owned(),
        FontId::proportional(12.0),
        theme.text_weak,
    );
    let x = rect.right() - 12.0 - font.size().x;
    painter.galley(pos2(x, y), font, theme.text_weak);
}

fn paint_gutter(
    ui: &Ui,
    gutter: Rect,
    galley: &Galley,
    galley_pos: egui::Pos2,
    cursor_line: usize,
    font_size: f32,
    theme: &Theme,
) {
    let painter = ui.painter_at(gutter);
    let mut line = 1usize;
    let mut starts_line = true;
    for row in &galley.rows {
        if starts_line {
            let row_rect = row.rect().translate(galley_pos.to_vec2());
            if row_rect.intersects(gutter) {
                if line == cursor_line {
                    painter.rect_filled(
                        Rect::from_min_max(
                            pos2(gutter.left(), row_rect.top()),
                            pos2(gutter.right(), row_rect.bottom()),
                        ),
                        CornerRadius::ZERO,
                        theme.current_line,
                    );
                }
                let color = if line == cursor_line {
                    theme.text
                } else {
                    theme.text_weak
                };
                let number =
                    painter.layout_no_wrap(line.to_string(), FontId::monospace(font_size), color);
                let x = gutter.right() - 10.0 - number.size().x;
                painter.galley(pos2(x, row_rect.top()), number, color);
            }
            line += 1;
        }
        starts_line = row.ends_with_newline;
    }
    painter.vline(
        gutter.right() - 0.5,
        gutter.y_range(),
        Stroke::new(1.0_f32, theme.stroke),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Theme;

    fn show_editor(width: f32, wrap: bool, text: &str) -> (usize, f32) {
        let ctx = egui::Context::default();
        Theme::dark().install(&ctx);
        let mut buffer = Buffer::from_text(text);
        let mut metrics = (0, 0.0);
        // The gallery's panels use a wrapping layout. Reproduce that so a
        // one-character wrap width fails this test.
        let _ = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(width, 420.0),
                )),
                ..Default::default()
            },
            |ui| {
                ui.with_layout(Layout::top_down(Align::Min).with_main_wrap(true), |ui| {
                    let output = Editor::new(&mut buffer).wrap(wrap).show(ui);
                    metrics = (output.rows, output.text_width);
                });
            },
        );
        metrics
    }

    #[test]
    fn word_wrap_breaks_on_words_not_characters() {
        let text = "alpha beta ".repeat(40);
        let (wrapped_rows, wrapped_width) = show_editor(320.0, true, &text);
        let (plain_rows, plain_width) = show_editor(320.0, false, &text);
        assert!(
            (2..=30).contains(&wrapped_rows),
            "wrapped rows = {wrapped_rows}, width = {wrapped_width}"
        );
        assert!(
            wrapped_width > 80.0 && wrapped_width < 320.0,
            "wrapped width = {wrapped_width}"
        );
        assert_eq!(plain_rows, 1, "unwrapped rows = {plain_rows}");
        assert!(
            plain_width > wrapped_width,
            "unwrapped width = {plain_width}, wrapped width = {wrapped_width}"
        );
    }

    #[test]
    fn showing_a_buffer_does_not_mark_it_dirty() {
        let ctx = egui::Context::default();
        Theme::dark().install(&ctx);
        let mut buffer = Buffer::from_text("fn main() {}\n");
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            Editor::new(&mut buffer).path("src/main.rs").show(ui);
        });
        assert!(!buffer.is_dirty());
        assert_eq!(buffer.text(), "fn main() {}\n");
    }
}

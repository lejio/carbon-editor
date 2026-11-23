//! CarbonEditor is a text editor on top of [egui](https://github.com/emilk/egui).
//!
//! It is a sister library to CarbonUI: a standalone crate with its own theme,
//! not a dependency of CarbonUI and not dependent on it. v1 edits a buffer,
//! wraps long lines, and keeps a [`Project`] of open files. Text is drawn in
//! [Lyth Mono](https://github.com/why-trv/LythMono). [`TextTheme::basic`] colors
//! keywords, strings, numbers, and comments.
//!
//! Call [`Theme::install`] once when the app starts. [`Editor`] reads that
//! theme back from egui memory, and the install registers Lyth Mono as the
//! monospace font.
//!
//! ```
//! # fn demo(ui: &mut carboneditor::egui::Ui, project: &mut carboneditor::Project, wrap: bool) {
//! if let Some(document) = project.active_mut() {
//!     let path = document.path().to_owned();
//!     carboneditor::Editor::new(document.buffer_mut())
//!         .path(&path)
//!         .wrap(wrap)
//!         .show(ui);
//! }
//! # }
//! ```

mod buffer;
mod editor;
mod font;
mod highlight;
mod project;
mod text;
mod theme;
mod tree;

pub use buffer::Buffer;
pub use editor::{Editor, EditorOutput};
pub use font::{install_font, FONT_NAME, FONT_SIZE};
pub use project::{Document, Project, ProjectError};
pub use text::TextPos;
pub use theme::{Mode, TextTheme, Theme};
pub use tree::ProjectTree;

pub use egui;

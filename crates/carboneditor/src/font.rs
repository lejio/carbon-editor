//! Lyth Mono, vendored from the v0.10.0 release.
//!
//! The bytes are the hinted Regular face. The SIL Open Font License is
//! `fonts/OFL.txt` next to this crate.

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily, FontId, Id};

/// Family name shown in the editor status line.
pub const FONT_NAME: &str = "Lyth Mono";

/// Default editor size, in points. Lyth Mono is drawn for about this size.
pub const FONT_SIZE: f32 = 14.0;

const FONT_KEY: &str = "LythMono";
const INSTALLED_ID: &str = "carboneditor.lyth_mono";

const LYTH_MONO_REGULAR: &[u8] = include_bytes!("../fonts/LythMono-Regular.ttf");

/// Register Lyth Mono as the first monospace font.
///
/// Safe to call every frame. The font is registered once per context.
pub fn install_font(ctx: &egui::Context) {
    let id = Id::new(INSTALLED_ID);
    let already = ctx.data(|data| data.get_temp::<bool>(id).unwrap_or(false));
    if already {
        return;
    }

    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        FONT_KEY.to_owned(),
        Arc::new(FontData::from_static(LYTH_MONO_REGULAR)),
    );
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .expect("egui monospace family")
        .insert(0, FONT_KEY.to_owned());
    ctx.set_fonts(fonts);
    ctx.data_mut(|data| data.insert_temp(id, true));
}

pub(crate) fn monospace(size: f32) -> FontId {
    FontId::monospace(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_is_truetype() {
        let magic = u32::from_be_bytes(LYTH_MONO_REGULAR[..4].try_into().unwrap());
        assert_eq!(magic, 0x0001_0000);
    }
}

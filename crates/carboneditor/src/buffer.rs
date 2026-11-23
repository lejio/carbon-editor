//! The string an [`crate::Editor`] mutates.

/// Editable text plus the last saved snapshot.
///
/// Dirty means the text differs from that snapshot, so an undo back to the
/// saved contents clears the flag without the editor tracking keystrokes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buffer {
    text: String,
    saved: String,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::from_text("")
    }
}

impl Buffer {
    /// A clean buffer holding `text`.
    pub fn from_text(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            saved: text.clone(),
            text,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Replace the text. Dirty when it differs from the saved snapshot.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn is_dirty(&self) -> bool {
        self.text != self.saved
    }

    /// Remember the current text as the saved snapshot.
    pub fn mark_clean(&mut self) {
        self.saved.clone_from(&self.text);
    }

    pub(crate) fn as_text_mut(&mut self) -> &mut String {
        &mut self.text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_tracks_the_saved_snapshot() {
        let mut buffer = Buffer::from_text("let x = 1;\n");
        assert!(!buffer.is_dirty());
        buffer.set_text("let x = 2;\n");
        assert!(buffer.is_dirty());
        buffer.set_text("let x = 1;\n");
        assert!(!buffer.is_dirty());
        buffer.as_text_mut().push_str("let y = 3;\n");
        assert!(buffer.is_dirty());
        buffer.mark_clean();
        assert!(!buffer.is_dirty());
    }
}

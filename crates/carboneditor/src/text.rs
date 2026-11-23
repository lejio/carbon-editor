//! Cursor math that does not need an egui context.

/// A caret position. Lines and columns are 1-based.
///
/// A column counts characters, not rendered cells. A tab is one character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextPos {
    pub line: usize,
    pub column: usize,
}

/// How many logical lines `text` occupies. An empty buffer is one line.
pub(crate) fn logical_lines(text: &str) -> usize {
    text.bytes().filter(|byte| *byte == b'\n').count() + 1
}

/// Caret position at a character index. Indexes past the end land at the end.
pub(crate) fn cursor_at(text: &str, char_index: usize) -> TextPos {
    let mut line = 1;
    let mut column = 1;
    for (index, ch) in text.chars().enumerate() {
        if index == char_index {
            return TextPos { line, column };
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    TextPos { line, column }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_tracks_lines() {
        let text = "ab\nc";
        assert_eq!(cursor_at(text, 0), TextPos { line: 1, column: 1 });
        assert_eq!(cursor_at(text, 2), TextPos { line: 1, column: 3 });
        assert_eq!(cursor_at(text, 3), TextPos { line: 2, column: 1 });
        assert_eq!(cursor_at(text, 99), TextPos { line: 2, column: 2 });
        assert_eq!(logical_lines(text), 2);
        assert_eq!(logical_lines(""), 1);
        assert_eq!(logical_lines("a\n"), 2);
    }
}

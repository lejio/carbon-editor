//! Basic source coloring: keywords, strings, numbers, and comments.
//!
//! The scan is not a language parser. It recognizes `//` and `/* */` comments,
//! quoted strings, character literals, numbers, and a small keyword list.

use std::ops::Range;

use egui::text::{LayoutJob, LayoutSection, TextFormat, TextWrapping};
use egui::{Color32, FontId};

use crate::theme::TextTheme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Plain,
    Comment,
    String,
    Keyword,
    Number,
}

pub(crate) fn job(text: &str, font_id: FontId, theme: TextTheme, wrap_width: f32) -> LayoutJob {
    let sections = token_spans(text)
        .into_iter()
        .map(|(byte_range, kind)| LayoutSection {
            leading_space: 0.0,
            byte_range,
            format: TextFormat::simple(font_id.clone(), color(theme, kind)),
        })
        .collect();
    LayoutJob {
        text: text.to_owned(),
        sections,
        wrap: TextWrapping {
            max_width: wrap_width,
            ..Default::default()
        },
        break_on_newline: true,
        ..Default::default()
    }
}

fn color(theme: TextTheme, kind: Kind) -> Color32 {
    match kind {
        Kind::Plain => theme.plain,
        Kind::Comment => theme.comment,
        Kind::String => theme.string,
        Kind::Keyword => theme.keyword,
        Kind::Number => theme.number,
    }
}

fn token_spans(text: &str) -> Vec<(Range<usize>, Kind)> {
    if text.is_empty() {
        return vec![(0..0, Kind::Plain)];
    }
    let bytes = text.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        let ch = bytes[i];
        if ch == b'/' && bytes.get(i + 1) == Some(&b'/') {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            push(&mut spans, start, i, Kind::Comment);
            continue;
        }
        if ch == b'/' && bytes.get(i + 1) == Some(&b'*') {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            if i + 1 < bytes.len() {
                i += 2;
            } else {
                i = bytes.len();
            }
            push(&mut spans, start, i, Kind::Comment);
            continue;
        }
        if ch == b'"' {
            i = string_end(bytes, i);
            push(&mut spans, start, i, Kind::String);
            continue;
        }
        if ch == b'\'' {
            if let Some(end) = char_literal_end(bytes, i) {
                push(&mut spans, start, end, Kind::String);
                i = end;
            } else {
                push(&mut spans, start, i + 1, Kind::Plain);
                i += 1;
            }
            continue;
        }
        if ch.is_ascii_digit()
            || (ch == b'.' && bytes.get(i + 1).is_some_and(|b| b.is_ascii_digit()))
        {
            i = number_end(bytes, i);
            push(&mut spans, start, i, Kind::Number);
            continue;
        }
        if is_ident_start(ch) {
            i += 1;
            while i < bytes.len() && is_ident_continue(bytes[i]) {
                i += 1;
            }
            let kind = if is_keyword(&text[start..i]) {
                Kind::Keyword
            } else {
                Kind::Plain
            };
            push(&mut spans, start, i, kind);
            continue;
        }
        i += char_len(bytes, i);
        push(&mut spans, start, i, Kind::Plain);
    }
    spans
}

fn push(spans: &mut Vec<(Range<usize>, Kind)>, start: usize, end: usize, kind: Kind) {
    if start >= end {
        return;
    }
    if let Some((range, previous)) = spans.last_mut() {
        if *previous == kind && range.end == start {
            range.end = end;
            return;
        }
    }
    spans.push((start..end, kind));
}

fn string_end(bytes: &[u8], open: usize) -> usize {
    let mut i = open + 1;
    while i < bytes.len() && bytes[i] != b'\n' {
        if bytes[i] == b'\\' {
            i += 1;
            if i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'"' {
            return i + 1;
        }
        i += 1;
    }
    i
}

fn char_literal_end(bytes: &[u8], open: usize) -> Option<usize> {
    let rest = bytes.get(open + 1..)?;
    if rest.first() == Some(&b'\\') {
        if rest.get(1) == Some(&b'u') {
            let close = rest.iter().position(|byte| *byte == b'}')?;
            let end = open + 1 + close + 1;
            return (bytes.get(end) == Some(&b'\'')).then_some(end + 1);
        }
        let end = open + 4;
        return (bytes.get(open + 3) == Some(&b'\'') && end <= bytes.len()).then_some(end);
    }
    (rest.len() >= 2 && rest[1] == b'\'' && rest[0] != b'\n').then_some(open + 3)
}

fn number_end(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte.is_ascii_alphanumeric() || byte == b'_' {
            i += 1;
            continue;
        }
        if byte == b'.' && bytes.get(i + 1).is_some_and(|next| next.is_ascii_digit()) {
            i += 1;
            continue;
        }
        break;
    }
    i
}

fn char_len(bytes: &[u8], index: usize) -> usize {
    let first = bytes[index];
    let len = if first < 0x80 {
        1
    } else if first < 0xE0 {
        2
    } else if first < 0xF0 {
        3
    } else {
        4
    };
    len.min(bytes.len() - index).max(1)
}

fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_ident_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "Self"
            | "as"
            | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Theme;

    fn kind_at<'a>(spans: &'a [(Range<usize>, Kind)], text: &str, needle: &str) -> Kind {
        let start = text.find(needle).expect(needle);
        spans
            .iter()
            .find(|(range, _)| range.contains(&start))
            .expect(needle)
            .1
    }

    #[test]
    fn basic_theme_colors_tokens_and_covers_the_buffer() {
        let text = "fn main() {\n    // note\n    let n = 12;\n    let s = \"hi // there\";\n    /* block */\n    let c = 'x';\n    let life = 'static;\n}\n";
        let spans = token_spans(text);
        let mut at = 0;
        for (range, _) in &spans {
            assert_eq!(range.start, at);
            assert!(text.is_char_boundary(range.start));
            assert!(text.is_char_boundary(range.end));
            at = range.end;
        }
        assert_eq!(at, text.len());
        assert_eq!(kind_at(&spans, text, "fn"), Kind::Keyword);
        assert_eq!(kind_at(&spans, text, "main"), Kind::Plain);
        assert_eq!(kind_at(&spans, text, "// note"), Kind::Comment);
        assert_eq!(kind_at(&spans, text, "12"), Kind::Number);
        assert_eq!(kind_at(&spans, text, "\"hi // there\""), Kind::String);
        assert_eq!(kind_at(&spans, text, "/* block */"), Kind::Comment);
        assert_eq!(kind_at(&spans, text, "'x'"), Kind::String);
        assert_eq!(kind_at(&spans, text, "'static"), Kind::Plain);

        let theme = Theme::dark().text_theme();
        let laid = job(text, FontId::monospace(14.0), theme, 320.0);
        assert_eq!(laid.text, text);
        assert!(laid
            .sections
            .iter()
            .any(|section| section.format.color == theme.keyword));
        assert!(laid
            .sections
            .iter()
            .any(|section| section.format.color == theme.string));
        assert_eq!(job("", FontId::monospace(14.0), theme, 80.0).text, "");
    }
}

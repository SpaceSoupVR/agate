use std::sync::Arc;

use space_soup::ui2d::{Align, Font, Span};

use crate::theme;

#[derive(Clone, Copy, PartialEq)]
enum Kind { Str, Number, Literal, Punct, Ws, Comment, Other }

struct Tok { start: usize, end: usize, kind: Kind }

fn tokenize(chars: &[char]) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut i = 0;
    let n = chars.len();
    while i < n {
        let c = chars[i];
        if c == ' ' || c == '\t' {
            let s = i;
            while i < n && (chars[i] == ' ' || chars[i] == '\t') { i += 1; }
            out.push(Tok { start: s, end: i, kind: Kind::Ws });
        } else if c == '"' {
            let s = i; i += 1;
            while i < n { if chars[i] == '\\' { i += 2; continue; } if chars[i] == '"' { i += 1; break; } i += 1; }
            out.push(Tok { start: s, end: i.min(n), kind: Kind::Str });
        } else if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            out.push(Tok { start: i, end: n, kind: Kind::Comment }); i = n;
        } else if c.is_ascii_digit() || (c == '-' && i + 1 < n && chars[i + 1].is_ascii_digit()) {
            let s = i; i += 1;
            while i < n && (chars[i].is_ascii_digit() || matches!(chars[i], '.' | 'e' | 'E' | '+' | '-')) { i += 1; }
            out.push(Tok { start: s, end: i, kind: Kind::Number });
        } else if c.is_ascii_alphabetic() {
            let s = i;
            while i < n && chars[i].is_ascii_alphabetic() { i += 1; }
            let word: String = chars[s..i].iter().collect();
            let kind = if matches!(word.as_str(), "true" | "false" | "null") { Kind::Literal } else { Kind::Other };
            out.push(Tok { start: s, end: i, kind });
        } else if matches!(c, '{' | '}' | '[' | ']' | ':' | ',') {
            out.push(Tok { start: i, end: i + 1, kind: Kind::Punct }); i += 1;
        } else { out.push(Tok { start: i, end: i + 1, kind: Kind::Other }); i += 1; }
    }
    out
}

fn next_is_colon(toks: &[Tok], from: usize, chars: &[char]) -> bool {
    for t in &toks[from + 1..] {
        match t.kind {
            Kind::Ws => continue,
            Kind::Punct => return chars.get(t.start) == Some(&':'),
            _ => return false,
        }
    }
    false
}

pub(crate) fn highlight_json_line(line: &str, scroll_col: usize, font: &Arc<Font>, font_px: f32) -> Vec<Span> {
    if line.is_empty() { return Vec::new(); }
    let chars: Vec<char> = line.chars().collect();
    let toks = tokenize(&chars);
    let mut spans = Vec::new();
    let mut char_cursor = 0usize;
    for (i, t) in toks.iter().enumerate() {
        let text: String = chars[t.start..t.end].iter().collect();
        let tok_len = t.end - t.start;
        let visible = if char_cursor + tok_len <= scroll_col {
            char_cursor += tok_len; continue;
        } else if char_cursor < scroll_col {
            let skip = scroll_col - char_cursor;
            text.chars().skip(skip).collect::<String>()
        } else { text.clone() };
        char_cursor += tok_len;
        if visible.is_empty() { continue; }
        let color = match t.kind {
            Kind::Str => if next_is_colon(&toks, i, &chars) { theme::SYN_KEY } else { theme::SYN_STRING },
            Kind::Number => theme::SYN_NUMBER,
            Kind::Literal => theme::SYN_KEYWORD,
            Kind::Punct => theme::SYN_PUNCT,
            Kind::Comment => theme::SYN_COMMENT,
            _ => theme::SYN_PLAIN,
        };
        spans.push(Span::new(visible, font.clone(), font_px, color).with_align(Align::Left));
    }
    spans
}

pub(crate) fn plain_line(line: &str, scroll_col: usize, font: &Arc<Font>, font_px: f32) -> Vec<Span> {
    let visible: String = line.chars().skip(scroll_col).collect();
    if visible.is_empty() { return Vec::new(); }
    vec![Span::new(visible, font.clone(), font_px, theme::SYN_PLAIN).with_align(Align::Left)]
}
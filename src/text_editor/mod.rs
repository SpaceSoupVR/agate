use std::path::{Path, PathBuf};

mod edit_ops;
mod highlight;
mod motion;
mod render;
mod undo;

pub(crate) use undo::Snapshot;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}

impl Pos {
    pub fn new(row: usize, col: usize) -> Self { Self { row, col } }
}

#[derive(Clone, Copy, Debug)]
pub struct EditorGeom {
    pub rect: (f32, f32, f32, f32),
    pub text_x: f32,
    pub top_y: f32,
    pub char_w: f32,
    pub line_h: f32,
    pub visible_rows: usize,
}

/// Geometry of the horizontal scrollbar track, recomputed in `build_items`
/// whenever a line is wider than the viewport — lets `text_editor()` turn
/// clicks/drags on the track into a `scroll_col` jump, since Shift+wheel is
/// the only other way to pan a long line into view and isn't discoverable.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HScrollGeom {
    pub track_x: f32,
    pub track_w: f32,
    pub bar_y: f32,
    pub bar_h: f32,
    pub max_scroll_col: usize,
}

pub(crate) const TAB_STR: &str = "  ";

/// How many columns of padding to keep between the caret and the viewport edge.
const SCROLL_COL_MARGIN: usize = 4;
/// How many rows of padding to keep when scrolling vertically to the caret.
const SCROLL_ROW_MARGIN: usize = 3;

pub struct TextEditor {
    pub path: Option<PathBuf>,
    pub(crate) lines: Vec<String>,
    pub(crate) cursor: Pos,
    pub(crate) anchor: Option<Pos>,
    pub(crate) scroll_row: usize,
    pub(crate) scroll_col: usize,
    pub dirty: bool,
    pub(crate) clipboard: String,
    pub(crate) undo: Vec<Snapshot>,
    pub(crate) redo: Vec<Snapshot>,
    pub(crate) last_geom: Option<EditorGeom>,
    pub(crate) last_hscroll: Option<HScrollGeom>,
    pub tab_width: usize,
    pub syntax: bool,
    pub(crate) last_click_at: Option<std::time::Instant>,
    pub(crate) last_click_pos: Option<(f32, f32)>,
    pub(crate) click_count: u32,
}

impl TextEditor {
    pub fn empty() -> Self {
        Self {
            path: None,
            lines: vec![String::new()],
            cursor: Pos::default(),
            anchor: None,
            scroll_row: 0,
            scroll_col: 0,
            dirty: false,
            clipboard: String::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            last_geom: None,
            last_hscroll: None,
            tab_width: 2,
            syntax: true,
            last_click_at: None,
            last_click_pos: None,
            click_count: 0,
        }
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let mut ed = Self::empty();
        ed.set_text(&text);
        ed.path = Some(path.to_path_buf());
        ed.dirty = false;
        Ok(ed)
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(p) = self.path.clone() {
            std::fs::write(&p, self.text())?;
            self.dirty = false;
        }
        Ok(())
    }

    pub fn set_text(&mut self, text: &str) {
        self.lines = if text.is_empty() {
            vec![String::new()]
        } else {
            text.replace('\t', TAB_STR)
                .split('\n')
                .map(|l| l.trim_end_matches('\r').to_string())
                .collect()
        };
        if self.lines.is_empty() { self.lines.push(String::new()); }
        self.cursor = Pos::default();
        self.anchor = None;
        self.scroll_row = 0;
        self.scroll_col = 0;
        self.undo.clear();
        self.redo.clear();
        self.last_click_at = None;
        self.last_click_pos = None;
        self.click_count = 0;
    }

    pub fn text(&self) -> String { self.lines.join("\n") }

    pub fn file_name(&self) -> String {
        self.path.as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".to_string())
    }

    pub fn cursor_line_col(&self) -> (usize, usize) {
        (self.cursor.row + 1, self.cursor.col + 1)
    }

    pub fn line_count(&self) -> usize { self.lines.len() }
    pub fn has_selection(&self) -> bool { self.selection_range().is_some() }

    pub(crate) fn selection_range(&self) -> Option<(Pos, Pos)> {
        let a = self.anchor?;
        let b = self.cursor;
        if a == b { return None; }
        Some(if a <= b { (a, b) } else { (b, a) })
    }

    pub(crate) fn begin_selection(&mut self, extend: bool) {
        if extend {
            if self.anchor.is_none() { self.anchor = Some(self.cursor); }
        } else {
            self.anchor = None;
        }
    }

    pub(crate) fn selected_text(&self) -> String {
        let Some((s, e)) = self.selection_range() else { return String::new() };
        if s.row == e.row { return substr(&self.lines[s.row], s.col, e.col); }
        let mut out = substr_from(&self.lines[s.row], s.col);
        out.push('\n');
        for r in (s.row + 1)..e.row { out.push_str(&self.lines[r]); out.push('\n'); }
        out.push_str(&substr(&self.lines[e.row], 0, e.col));
        out
    }

    pub(crate) fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection_range() else { return false };
        self.push_undo();
        if start.row == end.row {
            let line = &mut self.lines[start.row];
            let s = byte_idx(line, start.col);
            let e = byte_idx(line, end.col);
            line.replace_range(s..e, "");
        } else {
            let head = substr(&self.lines[start.row], 0, start.col);
            let tail = substr_from(&self.lines[end.row], end.col);
            self.lines.splice(start.row..=end.row, std::iter::once(format!("{head}{tail}")));
        }
        self.cursor = start;
        self.anchor = None;
        self.dirty = true;
        true
    }

    // ── Scroll / visibility helpers ──────────────────────────────────────────

    /// Scroll so the caret row is visible, with a margin of a few lines.
    pub(crate) fn ensure_row_visible(&mut self) {
        let Some(geom) = self.last_geom else { return };
        let vis = geom.visible_rows;
        let row = self.cursor.row;
        let margin = SCROLL_ROW_MARGIN.min(vis / 2);

        if row < self.scroll_row + margin {
            self.scroll_row = row.saturating_sub(margin);
        } else if row + margin >= self.scroll_row + vis {
            self.scroll_row = (row + margin + 1).saturating_sub(vis);
        }
        self.scroll_row = self.scroll_row.min(self.lines.len().saturating_sub(1));
    }

    /// Scroll horizontally so the caret column is visible, with a small margin.
    pub(crate) fn ensure_col_visible(&mut self) {
        let Some(geom) = self.last_geom else { return };
        let col = self.cursor.col;
        let margin = SCROLL_COL_MARGIN;

        // Derive how many columns are visible from geom
        let visible_cols = if geom.char_w > 0.0 {
            let text_w = geom.rect.2 - (geom.text_x - geom.rect.0);
            (text_w / geom.char_w).floor() as usize
        } else {
            80
        };

        if col < self.scroll_col + margin {
            self.scroll_col = col.saturating_sub(margin);
        } else if col + margin >= self.scroll_col + visible_cols {
            self.scroll_col = (col + margin + 1).saturating_sub(visible_cols);
        }
    }

    /// Scroll both axes so the caret is visible. Call after any cursor move.
    pub(crate) fn ensure_visible_default(&mut self) {
        self.ensure_row_visible();
        self.ensure_col_visible();
    }

    /// Scroll to show a specific row without moving the cursor.
    pub fn scroll_to_row(&mut self, row: usize) {
        self.scroll_row = row.min(self.lines.len().saturating_sub(1));
    }

    /// Scroll by `delta` rows (positive = down, negative = up).
    pub fn scroll_by_rows(&mut self, delta: i32) {
        if delta < 0 {
            self.scroll_row = self.scroll_row.saturating_sub((-delta) as usize);
        } else {
            self.scroll_row = (self.scroll_row + delta as usize)
                .min(self.lines.len().saturating_sub(1));
        }
    }

    /// Scroll by `delta` columns (positive = right, negative = left).
    pub fn scroll_by_cols(&mut self, delta: i32) {
        if delta < 0 {
            self.scroll_col = self.scroll_col.saturating_sub((-delta) as usize);
        } else {
            let max_col = self.lines.iter()
                .map(|l| char_len(l))
                .max()
                .unwrap_or(0);
            self.scroll_col = (self.scroll_col + delta as usize).min(max_col);
        }
    }

    /// Hit-test a pixel coordinate against the editor, returning a (row, col) Pos.
    /// Useful for mouse click → cursor position mapping.
    pub fn pos_from_point(&self, px: f32, py: f32) -> Option<Pos> {
        let geom = self.last_geom?;
        let (_, _, _, _) = geom.rect;
        let rel_y = py - geom.top_y;
        let rel_x = px - geom.text_x;
        if rel_x < 0.0 || rel_y < 0.0 { return None; }
        let vi = (rel_y / geom.line_h).floor() as usize;
        let row = (self.scroll_row + vi).min(self.lines.len().saturating_sub(1));
        let col_offset = (rel_x / geom.char_w).round() as usize;
        let col = (self.scroll_col + col_offset).min(char_len(&self.lines[row]));
        Some(Pos::new(row, col))
    }
}

pub(crate) fn char_len(s: &str) -> usize { s.chars().count() }

pub(crate) fn byte_idx(s: &str, col: usize) -> usize {
    s.char_indices().nth(col).map(|(i, _)| i).unwrap_or(s.len())
}

pub(crate) fn substr(s: &str, from: usize, to: usize) -> String {
    s.chars().skip(from).take(to.saturating_sub(from)).collect()
}

pub(crate) fn substr_from(s: &str, from: usize) -> String {
    s.chars().skip(from).collect()
}

pub(crate) fn digit_count(n: usize) -> usize {
    let mut d = 1;
    let mut v = n;
    while v >= 10 { v /= 10; d += 1; }
    d.max(2)
}
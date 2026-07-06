use super::{Pos, TextEditor};

pub(crate) struct Snapshot {
    pub(crate) lines: Vec<String>,
    pub(crate) cursor: Pos,
}

impl TextEditor {
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            lines: self.lines.clone(),
            cursor: self.cursor,
        }
    }

    pub(crate) fn push_undo(&mut self) {
        self.undo.push(self.snapshot());
        if self.undo.len() > 200 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn undo(&mut self) {
        if let Some(s) = self.undo.pop() {
            self.redo.push(self.snapshot());
            self.lines = s.lines;
            self.cursor = s.cursor;
            self.anchor = None;
            self.dirty = true;
            self.ensure_visible_default();
        }
    }

    pub fn redo(&mut self) {
        if let Some(s) = self.redo.pop() {
            self.undo.push(self.snapshot());
            self.lines = s.lines;
            self.cursor = s.cursor;
            self.anchor = None;
            self.dirty = true;
            self.ensure_visible_default();
        }
    }

    pub fn select_all(&mut self) {
        self.anchor = Some(Pos::new(0, 0));
        let last = self.lines.len() - 1;
        self.cursor = Pos::new(last, super::char_len(&self.lines[last]));
    }

    pub fn select_line(&mut self, row: usize) {
        let row = row.min(self.lines.len() - 1);
        self.anchor = Some(Pos::new(row, 0));
        if row + 1 < self.lines.len() {
            self.cursor = Pos::new(row + 1, 0);
        } else {
            self.cursor = Pos::new(row, super::char_len(&self.lines[row]));
        }
    }
}

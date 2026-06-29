use super::{byte_idx, char_len, substr, substr_from, TextEditor, TAB_STR};

impl TextEditor {
    pub fn insert_str(&mut self, text: &str) {
        if text.is_empty() { return; }
        self.delete_selection();
        self.push_undo();
        let text = text.replace('\t', TAB_STR).replace('\r', "");
        if !text.contains('\n') {
            let line = &mut self.lines[self.cursor.row];
            let b = byte_idx(line, self.cursor.col);
            line.insert_str(b, &text);
            self.cursor.col += char_len(&text);
        } else {
            let parts: Vec<&str> = text.split('\n').collect();
            let row = self.cursor.row;
            let cur = self.lines[row].clone();
            let head = substr(&cur, 0, self.cursor.col);
            let tail = substr_from(&cur, self.cursor.col);
            let mut new_lines = Vec::with_capacity(parts.len());
            new_lines.push(format!("{head}{}", parts[0]));
            for p in &parts[1..parts.len() - 1] { new_lines.push(p.to_string()); }
            let last = parts[parts.len() - 1];
            let new_col = char_len(last);
            new_lines.push(format!("{last}{tail}"));
            let added = new_lines.len();
            self.lines.splice(row..=row, new_lines);
            self.cursor.row = row + added - 1;
            self.cursor.col = new_col;
        }
        self.dirty = true;
        self.ensure_visible_default();
    }

    pub fn insert_char(&mut self, ch: char) {
        let mut buf = [0u8; 4];
        self.insert_str(ch.encode_utf8(&mut buf));
    }

    pub fn newline(&mut self) {
        let indent: String = self.lines[self.cursor.row]
            .chars().take_while(|c| *c == ' ').collect();

        // Smart indent: if line ends with '{', '[', or '(' add one more level
        let trimmed = self.lines[self.cursor.row][..{
            let bi = super::byte_idx(&self.lines[self.cursor.row], self.cursor.col);
            bi
        }].trim_end();
        let extra = if trimmed.ends_with('{') || trimmed.ends_with('[') || trimmed.ends_with('(') {
            TAB_STR
        } else {
            ""
        };
        self.insert_str(&format!("\n{indent}{extra}"));
    }

    pub fn backspace(&mut self) {
        if self.delete_selection() { self.ensure_visible_default(); return; }
        self.push_undo();
        if self.cursor.col > 0 {
            // Smart un-indent: if we're at the start of an indent block, remove a whole level
            let col = self.cursor.col;
            let line = &self.lines[self.cursor.row];
            let all_spaces = line[..super::byte_idx(line, col)]
                .chars().all(|c| c == ' ');
            let tab_w = TAB_STR.len();
            if all_spaces && col >= tab_w && col % tab_w == 0 {
                // Remove a whole indent level
                let new_col = col - tab_w;
                let line = &mut self.lines[self.cursor.row];
                let s = super::byte_idx(line, new_col);
                let e = super::byte_idx(line, col);
                line.replace_range(s..e, "");
                self.cursor.col = new_col;
            } else {
                let prev = self.cursor.col - 1;
                let line = &mut self.lines[self.cursor.row];
                let s = super::byte_idx(line, prev);
                let e = super::byte_idx(line, self.cursor.col);
                line.replace_range(s..e, "");
                self.cursor.col = prev;
            }
        } else if self.cursor.row > 0 {
            let cur = self.lines.remove(self.cursor.row);
            let above = self.cursor.row - 1;
            let new_col = char_len(&self.lines[above]);
            self.lines[above].push_str(&cur);
            self.cursor.row = above;
            self.cursor.col = new_col;
        } else {
            self.undo.pop();
            return;
        }
        self.dirty = true;
        self.ensure_visible_default();
    }

    pub fn delete_forward(&mut self) {
        if self.delete_selection() { self.ensure_visible_default(); return; }
        self.push_undo();
        let len = char_len(&self.lines[self.cursor.row]);
        if self.cursor.col < len {
            let line = &mut self.lines[self.cursor.row];
            let s = super::byte_idx(line, self.cursor.col);
            let e = super::byte_idx(line, self.cursor.col + 1);
            line.replace_range(s..e, "");
        } else if self.cursor.row + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor.row + 1);
            self.lines[self.cursor.row].push_str(&next);
        } else {
            self.undo.pop();
            return;
        }
        self.dirty = true;
        self.ensure_visible_default();
    }

    /// Delete from cursor to end of current word (Ctrl+Delete).
    pub fn delete_word_forward(&mut self) {
        if self.delete_selection() { self.ensure_visible_default(); return; }
        self.push_undo();
        let line = &self.lines[self.cursor.row];
        let col = self.cursor.col;
        let len = char_len(line);
        if col >= len {
            if self.cursor.row + 1 < self.lines.len() {
                let next = self.lines.remove(self.cursor.row + 1);
                self.lines[self.cursor.row].push_str(&next);
                self.dirty = true;
            } else {
                self.undo.pop();
            }
            return;
        }
        // Skip whitespace, then skip word chars
        let chars: Vec<char> = line.chars().collect();
        let mut end = col;
        while end < len && chars[end] == ' ' { end += 1; }
        if end == col { // no leading spaces – skip word
            while end < len && chars[end] != ' ' { end += 1; }
        }
        let line = &mut self.lines[self.cursor.row];
        let s = super::byte_idx(line, col);
        let e = super::byte_idx(line, end);
        line.replace_range(s..e, "");
        self.dirty = true;
        self.ensure_visible_default();
    }

    /// Delete from cursor back to start of current word (Ctrl+Backspace).
    pub fn delete_word_backward(&mut self) {
        if self.delete_selection() { self.ensure_visible_default(); return; }
        self.push_undo();
        let col = self.cursor.col;
        if col == 0 {
            if self.cursor.row > 0 {
                let cur = self.lines.remove(self.cursor.row);
                let above = self.cursor.row - 1;
                let new_col = char_len(&self.lines[above]);
                self.lines[above].push_str(&cur);
                self.cursor.row = above;
                self.cursor.col = new_col;
                self.dirty = true;
            } else {
                self.undo.pop();
            }
            return;
        }
        let line = &self.lines[self.cursor.row];
        let chars: Vec<char> = line.chars().take(col).collect();
        let mut start = col;
        // skip spaces backwards, then skip word chars
        while start > 0 && chars[start - 1] == ' ' { start -= 1; }
        if start == col { // no trailing spaces – skip word
            while start > 0 && chars[start - 1] != ' ' { start -= 1; }
        }
        let line = &mut self.lines[self.cursor.row];
        let s = super::byte_idx(line, start);
        let e = super::byte_idx(line, col);
        line.replace_range(s..e, "");
        self.cursor.col = start;
        self.dirty = true;
        self.ensure_visible_default();
    }

    pub fn copy(&mut self) {
        let s = self.selected_text();
        if !s.is_empty() { self.clipboard = s; }
    }

    pub fn cut(&mut self) {
        let s = self.selected_text();
        if !s.is_empty() {
            self.clipboard = s;
            self.delete_selection();
            self.ensure_visible_default();
        }
    }

    pub fn paste(&mut self) {
        let s = self.clipboard.clone();
        if !s.is_empty() { self.insert_str(&s); }
    }

    /// Duplicate the current line (or selection lines) below.
    pub fn duplicate_line(&mut self) {
        self.push_undo();
        let row = self.cursor.row;
        let line = self.lines[row].clone();
        self.lines.insert(row + 1, line);
        self.cursor.row = row + 1;
        self.dirty = true;
        self.ensure_visible_default();
    }

    /// Move the current line up by one row.
    pub fn move_line_up(&mut self) {
        if self.cursor.row == 0 { return; }
        self.push_undo();
        self.lines.swap(self.cursor.row - 1, self.cursor.row);
        self.cursor.row -= 1;
        self.dirty = true;
        self.ensure_visible_default();
    }

    /// Move the current line down by one row.
    pub fn move_line_down(&mut self) {
        if self.cursor.row + 1 >= self.lines.len() { return; }
        self.push_undo();
        self.lines.swap(self.cursor.row, self.cursor.row + 1);
        self.cursor.row += 1;
        self.dirty = true;
        self.ensure_visible_default();
    }

    /// Toggle a `//` line comment on the current line.
    pub fn toggle_line_comment(&mut self) {
        self.push_undo();
        let row = self.cursor.row;
        let line = self.lines[row].trim_start().to_string();
        if line.starts_with("//") {
            // Remove comment marker
            let orig = &self.lines[row];
            let pos = orig.find("//").unwrap();
            self.lines[row].replace_range(pos..pos + 2, "");
            // Also remove a single trailing space after //
            if self.lines[row].len() > pos && &self.lines[row][pos..pos + 1] == " " {
                self.lines[row].remove(pos);
            }
        } else {
            let indent_len = self.lines[row].chars().take_while(|c| *c == ' ').count();
            let bi = super::byte_idx(&self.lines[row], indent_len);
            self.lines[row].insert_str(bi, "// ");
            self.cursor.col = (self.cursor.col + 3).min(char_len(&self.lines[row]));
        }
        self.dirty = true;
    }

}
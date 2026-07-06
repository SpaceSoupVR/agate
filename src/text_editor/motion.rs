use super::{char_len, Pos, TextEditor};

impl TextEditor {
    pub fn move_left(&mut self, extend: bool) {
        self.begin_selection(extend);
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.cursor.col = char_len(&self.lines[self.cursor.row]);
        }
        self.ensure_visible_default();
    }

    pub fn move_right(&mut self, extend: bool) {
        self.begin_selection(extend);
        let len = char_len(&self.lines[self.cursor.row]);
        if self.cursor.col < len {
            self.cursor.col += 1;
        } else if self.cursor.row + 1 < self.lines.len() {
            self.cursor.row += 1;
            self.cursor.col = 0;
        }
        self.ensure_visible_default();
    }

    pub fn move_up(&mut self, extend: bool) {
        self.begin_selection(extend);
        if self.cursor.row > 0 {
            self.cursor.row -= 1;
            let len = char_len(&self.lines[self.cursor.row]);
            self.cursor.col = self.cursor.col.min(len);
        }
        self.ensure_visible_default();
    }

    pub fn move_down(&mut self, extend: bool) {
        self.begin_selection(extend);
        if self.cursor.row + 1 < self.lines.len() {
            self.cursor.row += 1;
            let len = char_len(&self.lines[self.cursor.row]);
            self.cursor.col = self.cursor.col.min(len);
        }
        self.ensure_visible_default();
    }

    pub fn move_home(&mut self, extend: bool) {
        self.begin_selection(extend);
        let indent: usize = self.lines[self.cursor.row]
            .chars()
            .take_while(|c| *c == ' ')
            .count();
        self.cursor.col = if self.cursor.col == indent { 0 } else { indent };
        self.ensure_col_visible();
    }

    pub fn move_end(&mut self, extend: bool) {
        self.begin_selection(extend);
        self.cursor.col = char_len(&self.lines[self.cursor.row]);
        self.ensure_col_visible();
    }

    pub fn move_file_start(&mut self, extend: bool) {
        self.begin_selection(extend);
        self.cursor = Pos::new(0, 0);
        self.scroll_row = 0;
        self.scroll_col = 0;
    }

    pub fn move_file_end(&mut self, extend: bool) {
        self.begin_selection(extend);
        let row = self.lines.len() - 1;
        let col = char_len(&self.lines[row]);
        self.cursor = Pos::new(row, col);
        self.ensure_visible_default();
    }

    pub fn page_up(&mut self, extend: bool) {
        self.begin_selection(extend);
        let vis = self.last_geom.map(|g| g.visible_rows).unwrap_or(20);
        let delta = vis.saturating_sub(2).max(1);
        self.cursor.row = self.cursor.row.saturating_sub(delta);
        let len = char_len(&self.lines[self.cursor.row]);
        self.cursor.col = self.cursor.col.min(len);
        self.ensure_visible_default();
    }

    pub fn page_down(&mut self, extend: bool) {
        self.begin_selection(extend);
        let vis = self.last_geom.map(|g| g.visible_rows).unwrap_or(20);
        let delta = vis.saturating_sub(2).max(1);
        self.cursor.row = (self.cursor.row + delta).min(self.lines.len() - 1);
        let len = char_len(&self.lines[self.cursor.row]);
        self.cursor.col = self.cursor.col.min(len);
        self.ensure_visible_default();
    }

    pub fn move_word_left(&mut self, extend: bool) {
        self.begin_selection(extend);
        if self.cursor.col == 0 {
            if self.cursor.row > 0 {
                self.cursor.row -= 1;
                self.cursor.col = char_len(&self.lines[self.cursor.row]);
            }
        } else {
            let chars: Vec<char> = self.lines[self.cursor.row].chars().collect();
            let mut col = self.cursor.col;

            while col > 0 && chars[col - 1] == ' ' {
                col -= 1;
            }

            while col > 0 && chars[col - 1] != ' ' {
                col -= 1;
            }
            self.cursor.col = col;
        }
        self.ensure_visible_default();
    }

    pub fn move_word_right(&mut self, extend: bool) {
        self.begin_selection(extend);
        let len = char_len(&self.lines[self.cursor.row]);
        if self.cursor.col >= len {
            if self.cursor.row + 1 < self.lines.len() {
                self.cursor.row += 1;
                self.cursor.col = 0;
            }
        } else {
            let chars: Vec<char> = self.lines[self.cursor.row].chars().collect();
            let mut col = self.cursor.col;

            while col < len && chars[col] != ' ' {
                col += 1;
            }

            while col < len && chars[col] == ' ' {
                col += 1;
            }
            self.cursor.col = col;
        }
        self.ensure_visible_default();
    }

    pub fn click(&mut self, px: f32, py: f32, extend: bool) {
        let now = std::time::Instant::now();
        let same_spot = self
            .last_click_pos
            .map(|(lx, ly)| (px - lx).abs() < 4.0 && (py - ly).abs() < 4.0)
            .unwrap_or(false);
        let quick = self
            .last_click_at
            .map(|t| now.duration_since(t).as_millis() < 400)
            .unwrap_or(false);

        if same_spot && quick {
            self.click_count += 1;
        } else {
            self.click_count = 1;
        }
        self.last_click_at = Some(now);
        self.last_click_pos = Some((px, py));

        if let Some(pos) = self.pos_from_point(px, py) {
            self.begin_selection(extend);
            self.cursor = pos;
            match self.click_count {
                2 => self.select_word_at_cursor(),
                3 => self.select_line_at_cursor(),
                _ => {}
            }
        }
        self.ensure_visible_default();
    }

    pub fn drag_to(&mut self, px: f32, py: f32) {
        if self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        if let Some(pos) = self.pos_from_point(px, py) {
            self.cursor = pos;
        }
        self.ensure_visible_default();
    }

    fn select_word_at_cursor(&mut self) {
        let line = &self.lines[self.cursor.row];
        let chars: Vec<char> = line.chars().collect();
        let col = self.cursor.col.min(chars.len());
        let is_word = |c: char| c.is_alphanumeric() || c == '_';
        let mut start = col;
        let mut end = col;
        if col < chars.len() && is_word(chars[col]) {
            while start > 0 && is_word(chars[start - 1]) {
                start -= 1;
            }
            while end < chars.len() && is_word(chars[end]) {
                end += 1;
            }
        } else if col < chars.len() {
            end = col + 1;
        }
        self.anchor = Some(Pos::new(self.cursor.row, start));
        self.cursor.col = end;
    }

    fn select_line_at_cursor(&mut self) {
        let row = self.cursor.row;
        self.anchor = Some(Pos::new(row, 0));
        self.cursor.col = char_len(&self.lines[row]);
    }
}

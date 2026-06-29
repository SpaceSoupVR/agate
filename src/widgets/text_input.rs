use space_soup::ui2d::Align;

use crate::text_editor::TextEditor;
use crate::theme;

use super::draw::{byte_idx_of, char_index_for_x, push_border, push_rect, push_rrect, text_advance_width};
use super::{in_rect, Rect, Ui, WidgetId};

impl Ui {
    pub fn text_input(
        &mut self,
        id: WidgetId,
        r: Rect,
        value: &str,
        placeholder: &str,
    ) -> Option<String> {
        let was_focused = self.focused == Some(id);
        let t = self.theme;

        {
            let st = self.state.entry(id).or_default();
            if st.text.is_none() { st.text = Some(value.to_string()); }
        }

        if self.input.left_just_pressed() {
            if in_rect(self.input.mouse_pos, r) {
                self.focused = Some(id);

                let pad = t.px(theme::PAD_SM);
                let size_px = t.body();
                let rel_x = (self.input.mouse_pos.0 - (r[0] + pad)).max(0.0);

                let st = self.state.entry(id).or_default();
                let snapshot = st.text.clone().unwrap_or_default();
                let idx = char_index_for_x(&snapshot, rel_x, &self.font, size_px);
                st.text_cursor = idx;
                st.text_anchor = None;
            } else if was_focused {
                self.focused = None;
            }
        }

        let focused = self.focused == Some(id);

        let border_color = if focused { theme::FIELD_FOCUS } else { theme::FIELD_BORDER };
        push_rrect(&mut self.items, r, t.px(theme::CORNER_SM), theme::FIELD_BG);
        push_border(&mut self.items, r, t.px(theme::CORNER_SM), border_color, t.px(1.0));

        let mut changed = false;
        if focused {
            let new_chars: String = self.input.text.chars()
                .filter(|c| !c.is_control())
                .collect();
            if !new_chars.is_empty() {
                let pad = t.px(theme::PAD_SM);
                let usable_w = (r[2] - pad * 2.0).max(0.0);
                let size_px = t.body();

                let st = self.state.entry(id).or_default();
                if let Some(ref mut txt) = st.text {
                    let cur = st.text_cursor.min(txt.chars().count());
                    let byte_idx_cur = byte_idx_of(txt, cur);

                    let mut accepted: String = String::new();
                    let mut candidate = txt.clone();
                    for ch in new_chars.chars() {
                        let mut probe = candidate.clone();
                        let probe_at = byte_idx_of(&probe, cur + accepted.chars().count());
                        probe.insert(probe_at, ch);
                        let w = text_advance_width(&probe, probe.chars().count(), &self.font, size_px);
                        if w > usable_w { break; }
                        candidate = probe;
                        accepted.push(ch);
                    }

                    if !accepted.is_empty() {
                        txt.insert_str(byte_idx_cur, &accepted);
                        st.text_cursor = cur + accepted.chars().count();
                        changed = true;
                    }
                }
            }

            for key in &self.input.keys.clone() {
                use crate::input::NamedKey::*;
                let st = self.state.entry(id).or_default();
                if let Some(ref mut txt) = st.text {
                    match key {
                        Backspace => {
                            if st.text_cursor > 0 {
                                let prev = st.text_cursor - 1;
                                let s = byte_idx_of(txt, prev);
                                let e = byte_idx_of(txt, st.text_cursor);
                                txt.replace_range(s..e, "");
                                st.text_cursor = prev;
                                changed = true;
                            }
                        }
                        Delete => {
                            let len = txt.chars().count();
                            if st.text_cursor < len {
                                let s = byte_idx_of(txt, st.text_cursor);
                                let e = byte_idx_of(txt, st.text_cursor + 1);
                                txt.replace_range(s..e, "");
                                changed = true;
                            }
                        }
                        ArrowLeft => { if st.text_cursor > 0 { st.text_cursor -= 1; } }
                        ArrowRight => { st.text_cursor = (st.text_cursor + 1).min(txt.chars().count()); }
                        Home => { st.text_cursor = 0; }
                        End => { st.text_cursor = txt.chars().count(); }
                        _ => {}
                    }
                }
            }
        }

        let pad = t.px(theme::PAD_SM);
        let text_y = r[1] + (r[3] - t.body()) * 0.5;
        let (display_text, text_color) = {
            let st = self.state.get(&id);
            let cur = st.and_then(|s| s.text.as_deref()).unwrap_or("");
            if cur.is_empty() {
                (placeholder.to_string(), theme::TEXT_DISABLED)
            } else {
                (cur.to_string(), theme::TEXT_PRIMARY)
            }
        };
        self.push_label(
            (r[0] + pad, text_y),
            &display_text,
            t.body(),
            text_color,
            Align::Left,
            r[2] - pad * 2.0,
            Some(r.into()),
        );

        if focused && (self.elapsed * 1.6) as u64 % 2 == 0 {
            let st = self.state.get(&id);
            let cur = st.map(|s| s.text_cursor).unwrap_or(0);
            let txt = st.and_then(|s| s.text.as_deref()).unwrap_or("");
            let caret_x = r[0] + pad + text_advance_width(txt, cur, &self.font, t.body());
            let caret_h = r[3] - t.px(6.0);
            push_rect(
                &mut self.items,
                [caret_x, r[1] + t.px(3.0), t.px(1.5), caret_h],
                theme::CARET,
            );
        }

        if changed {
            let st = self.state.get(&id);
            st.and_then(|s| s.text.clone())
        } else {
            None
        }
    }

    pub fn text_editor(
        &mut self,
        r: Rect,
        editor: &mut TextEditor,
        focused: bool,
    ) -> bool {
        let blink = (self.elapsed * 1.6) as u64 % 2 == 0;
        editor.build_items(
            (r[0], r[1], r[2], r[3]),
            &self.theme,
            &self.font,
            blink,
            focused,
            &mut self.items,
        );

        if self.input.left_just_pressed() && in_rect(self.input.mouse_pos, r) {
            let (mx, my) = self.input.mouse_pos;
            editor.click(mx, my, self.input.shift);
            return true;
        }
        if self.input.left_held() {
            let (mx, my) = self.input.mouse_pos;
            editor.drag_to(mx, my);
        }
        if self.is_hovered(r) && self.input.scroll_y.abs() > 0.01 {
            if self.input.shift {
                // Shift+scroll → horizontal
                editor.scroll_by_cols(self.input.scroll_y as i32);
            } else {
                // Normal scroll → vertical
                editor.scroll_by_rows(self.input.scroll_y as i32);
            }
        }
        false
    }
}
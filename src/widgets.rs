use std::sync::Arc;
use std::collections::HashMap;

use space_soup::ui2d::{Area, Item, Shape, ShapeType, Text, Span, Font, Align, Color};

use crate::theme::{self, Theme};
use crate::input::UiInput;
use crate::text_editor::TextEditor;


pub type Rect = [f32; 4];

fn in_rect(p: (f32, f32), r: Rect) -> bool {
    p.0 >= r[0] && p.0 <= r[0] + r[2] && p.1 >= r[1] && p.1 <= r[1] + r[3]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

impl WidgetId {
    pub fn of(label: &str) -> Self {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in label.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        Self(h)
    }
}

#[derive(Default)]
struct WidgetState {
    pressed: bool,
    slider_drag: Option<(f32, f32)>,
    dropdown_open: bool,
    scroll_y: f32,
    text: Option<String>,
    text_cursor: usize,
    text_anchor: Option<usize>,
}


struct PendingTooltip {
    rect:  Rect,
    label: String,
}

pub struct Ui {
    pub theme: Theme,
    pub font:  Arc<Font>,
    items:     Vec<(Area, Item)>,
    state:     HashMap<WidgetId, WidgetState>,
    input:     UiInput,
    win_w:     f32,
    win_h:     f32,
    drag_owner: Option<WidgetId>,
    focused:    Option<WidgetId>,
    elapsed:    f32,
    tooltip:    Option<PendingTooltip>,
}

impl Ui {
    pub fn new(scale: f32, font: Arc<Font>) -> Self {
        Self {
            theme:      Theme::new(scale),
            font,
            items:      Vec::with_capacity(256),
            state:      HashMap::new(),
            input:      UiInput::default(),
            win_w:      0.0,
            win_h:      0.0,
            drag_owner: None,
            focused:    None,
            elapsed:    0.0,
            tooltip:    None,
        }
    }

    pub fn begin_frame(&mut self, win_w: f32, win_h: f32, input: &UiInput) {
        self.win_w = win_w;
        self.win_h = win_h;
        self.input = input.clone();
        self.items.clear();
        self.tooltip = None;
        self.elapsed += input.dt;

        if input.left_just_released() { self.drag_owner = None; }
    }

    pub fn finish(&mut self) -> Vec<(Area, Item)> {
        if let Some(tt) = self.tooltip.take() {
            self.draw_tooltip_now(tt);
        }
        std::mem::take(&mut self.items)
    }

    fn draw_tooltip_now(&mut self, tt: PendingTooltip) {
        let t = &self.theme;
        let font_px = t.body();
        let pad = t.px(theme::PAD_SM);
        let text_w: f32 = tt.label.chars().count() as f32 * font_px * 0.62;
        let bw = text_w + pad * 2.0;
        let bh = t.px(theme::FIELD_H);

        let mx = (tt.rect[0] + tt.rect[2] * 0.5 - bw * 0.5)
            .clamp(0.0, (self.win_w - bw).max(0.0));
        let my = (tt.rect[1] - bh - t.px(6.0)).max(0.0);

        push_rrect(&mut self.items, [mx, my, bw, bh], t.px(theme::CORNER_SM), theme::SURFACE_RAISED);
        push_border(&mut self.items, [mx, my, bw, bh], t.px(theme::CORNER_SM), theme::BORDER, t.px(1.0));
        self.push_label(
            (mx + pad, my + (bh - font_px) * 0.5),
            &tt.label,
            font_px,
            theme::TEXT_PRIMARY,
            Align::Left,
            bw - pad,
            None,
        );
    }

    fn state(&mut self, id: WidgetId) -> &mut WidgetState {
        self.state.entry(id).or_default()
    }

    fn is_hovered(&self, r: Rect) -> bool {
        in_rect(self.input.mouse_pos, r)
    }

    fn just_clicked(&self, r: Rect) -> bool {
        self.input.left_just_released() && in_rect(self.input.mouse_pos, r)
    }

    fn push_label(
        &mut self,
        offset: (f32, f32),
        text: &str,
        size_px: f32,
        color: Color,
        align: Align,
        max_w: f32,
        clip: Option<(f32, f32, f32, f32)>,
    ) {
        let span = Span::new(text.to_string(), self.font.clone(), size_px, color).with_align(align);
        self.items.push((
            Area { offset, bounds: clip },
            Item::Text(Text::new(vec![span], max_w.max(1.0))),
        ));
    }

    fn push_center_label(&mut self, r: Rect, text: &str, size_px: f32, color: Color) {
        let text_w: f32 = text.chars()
            .map(|c| self.font.metrics(c, size_px).advance_width)
            .sum();
        let x = r[0] + (r[2] - text_w) * 0.5;
        let y = r[1] + (r[3] - size_px) * 0.5;
        let span = Span::new(text.to_string(), self.font.clone(), size_px, color).with_align(Align::Left);
        self.items.push((
            Area { offset: (x, y), bounds: Some((r[0], r[1], r[0] + r[2], r[1] + r[3])) },
            Item::Text(Text::new(vec![span], r[2].max(text_w + 1.0))),
        ));
    }

    pub fn fill(&mut self, r: Rect, color: Color) {
        push_rect(&mut self.items, r, color);
    }

    pub fn panel(&mut self, r: Rect, color: Color) {
        let radius = self.theme.px(theme::CORNER);
        push_rrect(&mut self.items, r, radius, color);
    }

    pub fn card(&mut self, r: Rect) {
        let t = &self.theme;
        push_rrect(&mut self.items, r, t.px(theme::CORNER_LG), theme::SURFACE);
        push_border(&mut self.items, r, t.px(theme::CORNER_LG), theme::BORDER, t.px(1.0));
    }

    pub fn separator(&mut self, x: f32, y: f32, width: f32) {
        let h = self.theme.px(1.0);
        push_rect(&mut self.items, [x, y, width, h], theme::SEPARATOR);
    }

    pub fn separator_v(&mut self, x: f32, y: f32, height: f32) {
        let w = self.theme.px(1.0);
        push_rect(&mut self.items, [x, y, w, height], theme::SEPARATOR);
    }

    pub fn label(&mut self, x: f32, y: f32, text: &str) {
        self.label_styled(x, y, text, self.theme.body(), theme::TEXT_PRIMARY, self.win_w - x, None);
    }

    pub fn label_secondary(&mut self, x: f32, y: f32, text: &str) {
        let size = self.theme.small();
        self.label_styled(x, y, text, size, theme::TEXT_SECONDARY, self.win_w - x, None);
    }

    pub fn label_styled(
        &mut self,
        x: f32, y: f32,
        text: &str,
        size_px: f32,
        color: Color,
        max_w: f32,
        clip: Option<Rect>,
    ) {
        let bounds = clip.map(|r| (r[0], r[1], r[0] + r[2], r[1] + r[3]));
        self.push_label((x, y), text, size_px, color, Align::Left, max_w, bounds);
    }

    pub fn label_clipped(&mut self, x: f32, y: f32, text: &str, clip: Rect) {
        let size = self.theme.body();
        let max_w = clip[2];
        self.label_styled(x, y, text, size, theme::TEXT_PRIMARY, max_w, Some(clip));
    }

    pub fn button(&mut self, r: Rect, label: &str) -> bool {
        self.button_styled(r, label, theme::ACCENT, theme::TEXT_ON_ACCENT)
    }

    pub fn button_secondary(&mut self, r: Rect, label: &str) -> bool {
        self.button_styled(r, label, theme::CONTROL_BG, theme::TEXT_PRIMARY)
    }

    pub fn button_danger(&mut self, r: Rect, label: &str) -> bool {
        self.button_styled(r, label, theme::DANGER, theme::TEXT_ON_ACCENT)
    }

    pub fn button_success(&mut self, r: Rect, label: &str) -> bool {
        self.button_styled(r, label, theme::SUCCESS, theme::TEXT_ON_ACCENT)
    }

    pub fn button_styled(&mut self, r: Rect, label: &str, bg: Color, fg: Color) -> bool {
        let t = &self.theme;
        let hovered  = self.is_hovered(r);
        let pressing = hovered && self.input.left_held();
        let clicked  = self.just_clicked(r);

        let bg = if pressing { dim_color(bg, 0.80) }
                 else if hovered { brighten_color(bg, 1.08) }
                 else { bg };

        push_rrect(&mut self.items, r, t.px(theme::CORNER), bg);
        let font_px = t.body();
        self.push_center_label(r, label, font_px, fg);
        clicked
    }

    pub fn icon_button(&mut self, r: Rect, icon: &str, tooltip: Option<&str>) -> bool {
        let clicked = self.button_secondary(r, icon);
        if let Some(tip) = tooltip {
            if self.is_hovered(r) {
                self.tooltip = Some(PendingTooltip { rect: r, label: tip.to_string() });
            }
        }
        clicked
    }

    pub fn toggle(&mut self, r: Rect, value: bool, label: &str) -> Option<bool> {
        let t = self.theme;
        let clicked = self.just_clicked(r);
        let new_val = if clicked { Some(!value) } else { None };
        let current = new_val.unwrap_or(value);

        let track_color = if current { theme::ACCENT } else { theme::CONTROL_BG };
        push_rrect(&mut self.items, r, r[3] * 0.5, track_color);

        let thumb_d = r[3] - t.px(4.0);
        let thumb_x = if current { r[0] + r[2] - thumb_d - t.px(2.0) } else { r[0] + t.px(2.0) };
        let thumb_y = r[1] + t.px(2.0);
        push_rrect(&mut self.items, [thumb_x, thumb_y, thumb_d, thumb_d], thumb_d * 0.5, theme::TEXT_ON_ACCENT);

        if !label.is_empty() {
            let lx = r[0] + r[2] + t.px(theme::PAD_SM);
            let ly = r[1] + (r[3] - t.body()) * 0.5;
            self.push_label((lx, ly), label, t.body(), theme::TEXT_PRIMARY, Align::Left, 400.0, None);
        }
        new_val
    }

    pub fn checkbox(&mut self, r: Rect, value: bool, label: &str) -> Option<bool> {
        let t = self.theme;
        let side = r[3].min(r[2]);
        let box_r = [r[0], r[1] + (r[3] - side) * 0.5, side, side];

        let clicked = self.just_clicked(r);
        let new_val = if clicked { Some(!value) } else { None };
        let current = new_val.unwrap_or(value);

        let bg = if current { theme::ACCENT } else { theme::FIELD_BG };
        push_rrect(&mut self.items, box_r, t.px(theme::CORNER_SM), bg);
        push_border(&mut self.items, box_r, t.px(theme::CORNER_SM), theme::FIELD_BORDER, t.px(1.0));

        if current {
            self.push_center_label(box_r, "✓", t.body(), theme::TEXT_ON_ACCENT);
        }

        if !label.is_empty() {
            let lx = r[0] + side + t.px(theme::PAD_SM);
            let ly = r[1] + (r[3] - t.body()) * 0.5;
            self.push_label((lx, ly), label, t.body(), theme::TEXT_PRIMARY, Align::Left, 400.0, None);
        }
        new_val
    }

    pub fn slider(
        &mut self,
        id: WidgetId,
        r: Rect,
        value: f32,
        range: std::ops::RangeInclusive<f32>,
    ) -> Option<f32> {
        let t = self.theme;
        let min = *range.start();
        let max = *range.end();
        let norm = if max > min { ((value - min) / (max - min)).clamp(0.0, 1.0) } else { 0.0 };

        let track_h = t.px(theme::SLIDER_TRACK);
        let thumb_d = t.px(theme::SLIDER_THUMB);
        let track_y = r[1] + (r[3] - track_h) * 0.5;
        let thumb_d_half = thumb_d * 0.5;

        let usable_w = r[2] - thumb_d;
        let filled_w = usable_w * norm;
        push_rrect(&mut self.items, [r[0] + thumb_d_half, track_y, usable_w, track_h], track_h * 0.5, theme::CONTROL_BG);
        if filled_w > 0.0 {
            push_rrect(&mut self.items, [r[0] + thumb_d_half, track_y, filled_w, track_h], track_h * 0.5, theme::ACCENT);
        }

        let thumb_x = r[0] + norm * usable_w;
        let thumb_y = r[1] + (r[3] - thumb_d) * 0.5;
        let hovered = self.is_hovered(r);
        let thumb_color = if hovered || self.drag_owner == Some(id) { theme::ACCENT_HI } else { theme::ACCENT };
        push_rrect(&mut self.items, [thumb_x, thumb_y, thumb_d, thumb_d], thumb_d * 0.5, thumb_color);

        let st = self.state.entry(id).or_default();
        let mut new_val = None;

        if self.input.left_just_pressed() && in_rect(self.input.mouse_pos, r) {
            self.drag_owner = Some(id);
            st.slider_drag = Some((value, self.input.mouse_pos.0));
        }

        if self.drag_owner == Some(id) {
            if let Some((start_val, start_x)) = st.slider_drag {
                let dx = self.input.mouse_pos.0 - start_x;
                let delta_norm = dx / usable_w;
                let new = (start_val + delta_norm * (max - min)).clamp(min, max);
                if (new - value).abs() > 1e-6 { new_val = Some(new); }
            }
        }
        new_val
    }

    pub fn slider_labeled(
        &mut self,
        id: WidgetId,
        r: Rect,
        value: f32,
        range: std::ops::RangeInclusive<f32>,
        format: &str,
    ) -> Option<f32> {
        let t = self.theme;
        let label_w = t.px(48.0);
        let slider_r = [r[0], r[1], r[2] - label_w - t.px(theme::PAD_SM), r[3]];
        let val_text = format.replace("{}", &format!("{:.2}", value));
        let lx = r[0] + r[2] - label_w;
        let ly = r[1] + (r[3] - t.body()) * 0.5;
        self.push_label((lx, ly), &val_text, t.body(), theme::TEXT_SECONDARY, Align::Right, label_w, None);
        self.slider(id, slider_r, value, range)
    }

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

                let pad     = t.px(theme::PAD_SM);
                let size_px = t.body();
                let rel_x   = (self.input.mouse_pos.0 - (r[0] + pad)).max(0.0);

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
                let pad      = t.px(theme::PAD_SM);
                let usable_w = (r[2] - pad * 2.0).max(0.0);
                let size_px  = t.body();

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
                        ArrowLeft  => { if st.text_cursor > 0 { st.text_cursor -= 1; } }
                        ArrowRight => { st.text_cursor = (st.text_cursor + 1).min(txt.chars().count()); }
                        Home       => { st.text_cursor = 0; }
                        End        => { st.text_cursor = txt.chars().count(); }
                        _          => {}
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
            editor.scroll_by(self.input.scroll_y as i32);
        }
        false
    }

    pub fn progress_bar(&mut self, r: Rect, progress: f32) {
        let t = self.theme;
        let p = progress.clamp(0.0, 1.0);
        push_rrect(&mut self.items, r, t.px(theme::CORNER_SM), theme::CONTROL_BG);
        if p > 0.0 {
            let filled_w = r[2] * p;
            push_rrect(
                &mut self.items,
                [r[0], r[1], filled_w, r[3]],
                t.px(theme::CORNER_SM),
                theme::ACCENT,
            );
        }
    }

    pub fn progress_bar_labeled(&mut self, r: Rect, progress: f32) {
        self.progress_bar(r, progress);
        let label = format!("{:.0}%", progress * 100.0);
        self.push_center_label(r, &label, self.theme.small(), theme::TEXT_ON_ACCENT);
    }

    pub fn color_swatch(&mut self, r: Rect, color: Color) -> bool {
        let t = self.theme;
        let hovered = self.is_hovered(r);
        push_rrect(&mut self.items, r, t.px(theme::CORNER_SM), color);
        if hovered {
            push_border(&mut self.items, r, t.px(theme::CORNER_SM), theme::TEXT_ON_ACCENT, t.px(2.0));
        }
        self.just_clicked(r)
    }

    pub fn spinner(&mut self, cx: f32, cy: f32, radius: f32) {
        let t = self.theme;
        let d = radius * 2.0;
        push_ellipse_outline(
            &mut self.items,
            [cx - radius, cy - radius, d, d],
            t.px(2.0),
            theme::CONTROL_BG,
        );

        let angle = (self.elapsed * 360.0 * 1.5) % 360.0;
        push_ellipse_outline_rotated(
            &mut self.items,
            [cx - radius * 0.7, cy - radius * 0.7, radius * 1.4, radius * 1.4],
            t.px(2.0),
            theme::ACCENT,
            angle,
        );
    }

    pub fn dropdown(
        &mut self,
        id: WidgetId,
        r: Rect,
        selected: usize,
        options: &[&str],
    ) -> Option<usize> {
        let t = self.theme;
        let open = self.state.get(&id).map(|s| s.dropdown_open).unwrap_or(false);
        let hovered = self.is_hovered(r);

        let bg = if hovered { theme::CONTROL_HOVER } else { theme::CONTROL_BG };
        push_rrect(&mut self.items, r, t.px(theme::CORNER), bg);
        push_border(&mut self.items, r, t.px(theme::CORNER), theme::CONTROL_BORDER, t.px(1.0));

        let label = options.get(selected).copied().unwrap_or("—");
        let pad = t.px(theme::PAD_SM);
        let font_px = t.body();
        let ly = r[1] + (r[3] - font_px) * 0.5;
        self.push_label((r[0] + pad, ly), label, font_px, theme::TEXT_PRIMARY, Align::Left, r[2] - pad * 3.0, None);

        let chevron = if open { "▲" } else { "▼" };
        self.push_label(
            (r[0] + r[2] - pad * 2.5, ly),
            chevron,
            t.small(),
            theme::TEXT_SECONDARY,
            Align::Left,
            pad * 3.0,
            None,
        );

        let clicked = self.just_clicked(r);
        if clicked {
            let st = self.state.entry(id).or_default();
            st.dropdown_open = !open;
        }

        let mut result = None;
        if open {
            let item_h = t.px(theme::ROW_H);
            let list_h = item_h * options.len() as f32;
            let lx = r[0];
            let ly = r[1] + r[3];
            let lw = r[2];

            push_rrect(&mut self.items, [lx, ly, lw, list_h], t.px(theme::CORNER), theme::SURFACE_RAISED);
            push_border(&mut self.items, [lx, ly, lw, list_h], t.px(theme::CORNER), theme::BORDER, t.px(1.0));

            for (i, opt) in options.iter().enumerate() {
                let item_r = [lx, ly + i as f32 * item_h, lw, item_h];
                let ih = self.is_hovered(item_r);
                let sel = i == selected;
                let item_bg = if sel { theme::ACCENT_DIM } else if ih { theme::CONTROL_HOVER } else { Color(0,0,0,0) };
                push_rect(&mut self.items, item_r, item_bg);

                let text_color = if sel { theme::ACCENT_HI } else { theme::TEXT_PRIMARY };
                let pad2 = t.px(theme::PAD);
                let ty = item_r[1] + (item_h - t.body()) * 0.5;
                self.push_label((item_r[0] + pad2, ty), opt, t.body(), text_color, Align::Left, lw - pad2 * 2.0, None);

                if self.just_clicked(item_r) {
                    result = Some(i);
                    let st = self.state.entry(id).or_default();
                    st.dropdown_open = false;
                }
            }

            if self.input.left_just_pressed() && !in_rect(self.input.mouse_pos, r) {
                let list_rect = [lx, ly, lw, list_h];
                if !in_rect(self.input.mouse_pos, list_rect) {
                    let st = self.state.entry(id).or_default();
                    st.dropdown_open = false;
                }
            }
        }
        result
    }


   pub fn scroll_area(
        &mut self,
        id: WidgetId,
        r: Rect,
        content_height: f32,
    ) -> (Rect, f32) {
        let t = self.theme;
        push_rrect(&mut self.items, r, t.px(theme::CORNER), theme::SURFACE);

        let hovered = self.is_hovered(r);
        let scroll_delta = if hovered {
            self.input.scroll_y * t.px(theme::ROW_H)
        } else {
            0.0
        };
        let max_scroll = (content_height - r[3]).max(0.0);

        let st = self.state.entry(id).or_default();
        st.scroll_y = (st.scroll_y + scroll_delta).clamp(0.0, max_scroll);
        let scroll_y = st.scroll_y;

        (r, scroll_y)
    }


    pub fn end_scroll_area(
        &mut self,
        id: WidgetId,
        r: Rect,
        content_height: f32,
    ) {
        let t = self.theme;
        if content_height <= r[3] { return; }

        let scroll_y = self.state.get(&id).map(|s| s.scroll_y).unwrap_or(0.0);
        let track_h  = r[3] - t.px(8.0);
        let thumb_h  = (track_h * r[3] / content_height).max(t.px(24.0));
        let max_sc   = (content_height - r[3]).max(1.0);
        let frac     = (scroll_y / max_sc).clamp(0.0, 1.0);
        let thumb_y  = r[1] + t.px(4.0) + frac * (track_h - thumb_h);

        self.items.push((
            Area { offset: (r[0] + r[2] - t.px(8.0), thumb_y), bounds: None },
            Item::Shape(Shape {
                shape: ShapeType::RoundedRectangle(0.0, (t.px(4.0), thumb_h), 0.0, t.px(2.0)),
                color: theme::SCROLLBAR,
            }),
        ));
    }

    pub fn tooltip(&mut self, trigger_rect: Rect, label: &str) {
        if self.is_hovered(trigger_rect) {
            self.tooltip = Some(PendingTooltip { rect: trigger_rect, label: label.to_string() });
        }
    }


    pub fn drag_float(
        &mut self,
        id: WidgetId,
        r: Rect,
        value: f32,
        speed: f32,
        label: &str,
    ) -> Option<f32> {
        let t = self.theme;
        let hovered = self.is_hovered(r);
        let active = self.drag_owner == Some(id);
        let border = if active { theme::FIELD_FOCUS } else { theme::FIELD_BORDER };

        push_rrect(&mut self.items, r, t.px(theme::CORNER_SM), theme::FIELD_BG);
        push_border(&mut self.items, r, t.px(theme::CORNER_SM), border, t.px(1.0));

        let pad = t.px(theme::PAD_SM);
        let ty  = r[1] + (r[3] - t.body()) * 0.5;

        if !label.is_empty() {
            self.push_label((r[0] + pad, ty), label, t.small(), theme::TEXT_SECONDARY, Align::Left, r[2] * 0.4, None);
        }
        let val_str = format!("{:.3}", value);
        self.push_label(
            (r[0] + pad, ty),
            &val_str,
            t.body(),
            theme::TEXT_PRIMARY,
            Align::Right,
            r[2] - pad * 2.0,
            Some(r.into()),
        );

        if self.input.left_just_pressed() && hovered {
            self.drag_owner = Some(id);
            let st = self.state.entry(id).or_default();
            st.slider_drag = Some((value, self.input.mouse_pos.0));
        }

        if active {
            if let Some((start_val, start_x)) = self.state.get(&id).and_then(|s| s.slider_drag) {
                let dx = self.input.mouse_pos.0 - start_x;
                let new = start_val + dx * speed;
                if (new - value).abs() > 1e-6 { return Some(new); }
            }
        }
        None
    }

    pub fn tabs(&mut self, r: Rect, selected: usize, labels: &[&str]) -> Option<usize> {
        let t = self.theme;
        push_rect(&mut self.items, r, theme::SURFACE);

        let tab_w = r[2] / labels.len() as f32;
        let mut result = None;
        for (i, label) in labels.iter().enumerate() {
            let tab_r = [r[0] + i as f32 * tab_w, r[1], tab_w, r[3]];
            let sel = i == selected;
            let hov = self.is_hovered(tab_r);
            let bg = if sel { theme::CONTROL_BG } else if hov { theme::CONTROL_ACTIVE } else { Color(0, 0, 0, 0) };
            push_rrect(&mut self.items, tab_r, t.px(theme::CORNER_SM), bg);

            let fg = if sel { theme::TEXT_PRIMARY } else { theme::TEXT_SECONDARY };
            self.push_center_label(tab_r, label, t.body(), fg);


            if sel {
                push_rect(
                    &mut self.items,
                    [tab_r[0] + t.px(8.0), tab_r[1] + tab_r[3] - t.px(2.0), tab_w - t.px(16.0), t.px(2.0)],
                    theme::ACCENT,
                );
            }
            if self.just_clicked(tab_r) && !sel { result = Some(i); }
        }


        push_rect(&mut self.items, [r[0], r[1] + r[3] - t.px(1.0), r[2], t.px(1.0)], theme::BORDER);
        result
    }

    pub fn badge(&mut self, r: Rect, label: &str, color: Color) {
        let t = self.theme;
        push_rrect(&mut self.items, r, r[3] * 0.5, color);
        self.push_center_label(r, label, t.small(), theme::TEXT_ON_ACCENT);
    }

    pub fn list_row(&mut self, r: Rect, label: &str, selected: bool) -> bool {
        let t = self.theme;
        let hov = self.is_hovered(r);
        let bg = if selected { theme::ACCENT_DIM }
                 else if hov { theme::CONTROL_HOVER }
                 else { Color(0, 0, 0, 0) };
        push_rect(&mut self.items, r, bg);

        let fg = if selected { theme::ACCENT_HI } else { theme::TEXT_PRIMARY };
        let lx = r[0] + t.px(theme::PAD);
        let ly = r[1] + (r[3] - t.body()) * 0.5;
        self.push_label((lx, ly), label, t.body(), fg, Align::Left, r[2] - t.px(theme::PAD * 2.0), None);
        self.just_clicked(r)
    }


    pub fn disclosure(&mut self, r: Rect, open: bool, label: &str) -> bool {
        let t = self.theme;
        let chevron = if open { "▾" } else { "▸" };
        let ch_x = r[0] + t.px(4.0);
        let cy    = r[1] + (r[3] - t.body()) * 0.5;
        self.push_label((ch_x, cy), chevron, t.body(), theme::TEXT_SECONDARY, Align::Left, t.px(16.0), None);

        let lx = r[0] + t.px(20.0);
        self.push_label((lx, cy), label, t.body(), theme::TEXT_PRIMARY, Align::Left, r[2] - t.px(24.0), None);

        if self.just_clicked(r) { !open } else { open }
    }
}


fn push_rect(items: &mut Vec<(Area, Item)>, r: Rect, color: Color) {
    items.push((
        Area { offset: (r[0], r[1]), bounds: None },
        Item::Shape(Shape { shape: ShapeType::Rectangle(0.0, (r[2], r[3]), 0.0), color }),
    ));
}

fn push_rrect(items: &mut Vec<(Area, Item)>, r: Rect, radius: f32, color: Color) {
    items.push((
        Area { offset: (r[0], r[1]), bounds: None },
        Item::Shape(Shape { shape: ShapeType::RoundedRectangle(0.0, (r[2], r[3]), 0.0, radius), color }),
    ));
}

fn push_border(items: &mut Vec<(Area, Item)>, r: Rect, radius: f32, color: Color, stroke: f32) {
    items.push((
        Area { offset: (r[0], r[1]), bounds: None },
        Item::Shape(Shape { shape: ShapeType::RoundedRectangle(stroke, (r[2], r[3]), 0.0, radius), color }),
    ));
}

fn push_ellipse_outline(items: &mut Vec<(Area, Item)>, r: Rect, stroke: f32, color: Color) {
    items.push((
        Area { offset: (r[0], r[1]), bounds: None },
        Item::Shape(Shape { shape: ShapeType::Ellipse(stroke, (r[2], r[3]), 0.0), color }),
    ));
}

fn push_ellipse_outline_rotated(items: &mut Vec<(Area, Item)>, r: Rect, stroke: f32, color: Color, angle_deg: f32) {
    items.push((
        Area { offset: (r[0], r[1]), bounds: None },
        Item::Shape(Shape { shape: ShapeType::Ellipse(stroke, (r[2], r[3]), angle_deg), color }),
    ));
}


fn brighten_color(c: Color, factor: f32) -> Color {
    Color(
        ((c.0 as f32 * factor).min(255.0) as u8),
        ((c.1 as f32 * factor).min(255.0) as u8),
        ((c.2 as f32 * factor).min(255.0) as u8),
        c.3,
    )
}

fn dim_color(c: Color, factor: f32) -> Color {
    brighten_color(c, factor)
}


fn byte_idx_of(s: &str, col: usize) -> usize {
    s.char_indices().nth(col).map(|(i, _)| i).unwrap_or(s.len())
}

fn text_advance_width(s: &str, up_to_col: usize, font: &Font, size_px: f32) -> f32 {
    s.chars().take(up_to_col)
        .map(|c| font.metrics(c, size_px).advance_width)
        .sum()
}

fn char_index_for_x(text: &str, rel_x: f32, font: &Font, size_px: f32) -> usize {
    let mut acc = 0.0f32;
    for (i, ch) in text.chars().enumerate() {
        let w = font.metrics(ch, size_px).advance_width;
        if rel_x < acc + w * 0.5 {
            return i;
        }
        acc += w;
    }
    text.chars().count()
}
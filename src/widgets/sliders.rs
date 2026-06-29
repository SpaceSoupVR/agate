use space_soup::ui2d::{Align, Color};

use crate::theme;

use super::draw::{push_border, push_ellipse_outline, push_ellipse_outline_rotated, push_rrect};
use super::{Rect, Ui, WidgetId};

impl Ui {
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

        if self.input.left_just_pressed() && super::in_rect(self.input.mouse_pos, r) {
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
        let ty = r[1] + (r[3] - t.body()) * 0.5;

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
}
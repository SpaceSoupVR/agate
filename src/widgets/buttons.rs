use space_soup::ui2d::{Align, Color};

use crate::theme;

use super::draw::{brighten_color, dim_color, push_rect, push_rect_clipped, push_rrect, push_border};
use super::{Rect, Ui};

impl Ui {
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
        let hovered = self.is_hovered(r);
        let pressing = hovered && self.input.left_held();
        let clicked = self.just_clicked(r);

        let bg = if pressing { dim_color(bg, 0.80) }
                 else if hovered { brighten_color(bg, 1.08) }
                 else { bg };

        push_rrect(&mut self.items, r, t.px(theme::CORNER), bg);
        let font_px = t.body();
        self.push_center_label(r, label, font_px, fg);
        clicked
    }

    /// Like `button_styled`, but with no hover/press feedback and no click
    /// detection — for a control that's shown but currently can't be
    /// activated (e.g. "Undo" with an empty history), so it doesn't
    /// visually invite a click it will then ignore.
    pub fn button_disabled(&mut self, r: Rect, label: &str, bg: Color, fg: Color) {
        let t = &self.theme;
        push_rrect(&mut self.items, r, t.px(theme::CORNER), bg);
        self.push_center_label(r, label, t.body(), fg);
    }

    pub fn icon_button(&mut self, r: Rect, icon: &str, tooltip: Option<&str>) -> bool {
        let clicked = self.button_secondary(r, icon);
        if let Some(tip) = tooltip {
            if self.is_hovered(r) {
                self.tooltip = Some(super::PendingTooltip { rect: r, label: tip.to_string() });
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
            self.push_center_label(box_r, "\u{2713}", t.body(), theme::TEXT_ON_ACCENT);
        }

        if !label.is_empty() {
            let lx = r[0] + side + t.px(theme::PAD_SM);
            let ly = r[1] + (r[3] - t.body()) * 0.5;
            self.push_label((lx, ly), label, t.body(), theme::TEXT_PRIMARY, Align::Left, 400.0, None);
        }
        new_val
    }

    pub fn badge(&mut self, r: Rect, label: &str, color: Color) {
        let t = self.theme;
        push_rrect(&mut self.items, r, r[3] * 0.5, color);
        self.push_center_label(r, label, t.small(), theme::TEXT_ON_ACCENT);
    }

    pub fn list_row(&mut self, r: Rect, label: &str, selected: bool) -> bool {
        self.list_row_clipped(r, label, selected, None)
    }

    /// Same as `list_row`, but the row's background and label are clamped
    /// to `clip` (when given) — used for scrollable lists like the
    /// navigator, where a row near the edge of its container must not
    /// paint over whatever sits just outside that container (e.g. a fixed
    /// footer panel below it).
    pub fn list_row_clipped(&mut self, r: Rect, label: &str, selected: bool, clip: Option<Rect>) -> bool {
        let t = self.theme;
        let hov = self.is_hovered(r);
        let bg = if selected { theme::ACCENT_DIM }
                 else if hov { theme::CONTROL_HOVER }
                 else { Color(0, 0, 0, 0) };
        push_rect_clipped(&mut self.items, r, bg, clip);

        let fg = if selected { theme::ACCENT_HI } else { theme::TEXT_PRIMARY };
        let lx = r[0] + t.px(theme::PAD);
        let ly = r[1] + (r[3] - t.body()) * 0.5;
        let bounds = clip.map(|c| (c[0], c[1], c[0] + c[2], c[1] + c[3]));
        self.push_label((lx, ly), label, t.body(), fg, Align::Left, r[2] - t.px(theme::PAD * 2.0), bounds);
        self.just_clicked(r)
    }

    pub fn disclosure(&mut self, r: Rect, open: bool, label: &str) -> bool {
        let t = self.theme;
        let chevron = if open { "\u{25be}" } else { "\u{25b8}" };
        let ch_x = r[0] + t.px(4.0);
        let cy = r[1] + (r[3] - t.body()) * 0.5;
        self.push_label((ch_x, cy), chevron, t.body(), theme::TEXT_SECONDARY, Align::Left, t.px(16.0), None);

        let lx = r[0] + t.px(20.0);
        self.push_label((lx, cy), label, t.body(), theme::TEXT_PRIMARY, Align::Left, r[2] - t.px(24.0), None);

        if self.just_clicked(r) { !open } else { open }
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
}
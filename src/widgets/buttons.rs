use space_soup::ui2d::{Align, Color};

use crate::theme;

use super::draw::{push_border, push_rect, push_rrect, push_rrect_clipped};
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

        let radius = t.px(theme::CORNER);
        push_rrect(&mut self.items, r, radius, bg);
        // Hover/press states are alpha overlays rather than RGB scaling, so
        // they stay visible on the translucent (glass) control colors.
        if pressing {
            push_rrect(&mut self.items, r, radius, Color(0, 0, 0, 46));
        } else if hovered {
            push_rrect(&mut self.items, r, radius, Color(255, 255, 255, 22));
        }
        let font_px = t.body();
        self.push_center_label(r, label, font_px, fg);
        clicked
    }

    pub fn button_disabled(&mut self, r: Rect, label: &str, bg: Color, fg: Color) {
        let t = &self.theme;
        push_rrect(&mut self.items, r, t.px(theme::CORNER), bg);
        self.push_center_label(r, label, t.body(), fg);
    }

    pub fn icon_button(&mut self, r: Rect, icon: &str, tooltip: Option<&str>) -> bool {
        let clicked = self.button_secondary(r, icon);
        if let Some(tip) = tooltip {
            if self.is_hovered(r) {
                self.tooltip = Some(super::PendingTooltip {
                    rect: r,
                    label: tip.to_string(),
                });
            }
        }
        clicked
    }

    pub fn toggle(&mut self, r: Rect, value: bool, label: &str) -> Option<bool> {
        let t = self.theme;
        let clicked = self.just_clicked(r);
        let new_val = if clicked { Some(!value) } else { None };
        let current = new_val.unwrap_or(value);

        let track_color = if current {
            theme::ACCENT
        } else {
            theme::CONTROL_BG
        };
        push_rrect(&mut self.items, r, r[3] * 0.5, track_color);

        let thumb_d = r[3] - t.px(4.0);
        let thumb_x = if current {
            r[0] + r[2] - thumb_d - t.px(2.0)
        } else {
            r[0] + t.px(2.0)
        };
        let thumb_y = r[1] + t.px(2.0);
        push_rrect(
            &mut self.items,
            [thumb_x, thumb_y, thumb_d, thumb_d],
            thumb_d * 0.5,
            theme::TEXT_ON_ACCENT,
        );

        if !label.is_empty() {
            let lx = r[0] + r[2] + t.px(theme::PAD_SM);
            let ly = r[1] + (r[3] - t.body()) * 0.5;
            self.push_label(
                (lx, ly),
                label,
                t.body(),
                theme::TEXT_PRIMARY,
                Align::Left,
                400.0,
                None,
            );
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

        let bg = if current {
            theme::ACCENT
        } else {
            theme::FIELD_BG
        };
        push_rrect(&mut self.items, box_r, t.px(theme::CORNER_SM), bg);
        push_border(
            &mut self.items,
            box_r,
            t.px(theme::CORNER_SM),
            theme::FIELD_BORDER,
            t.px(1.0),
        );

        if current {
            self.push_center_label(box_r, "\u{2713}", t.body(), theme::TEXT_ON_ACCENT);
        }

        if !label.is_empty() {
            let lx = r[0] + side + t.px(theme::PAD_SM);
            let ly = r[1] + (r[3] - t.body()) * 0.5;
            self.push_label(
                (lx, ly),
                label,
                t.body(),
                theme::TEXT_PRIMARY,
                Align::Left,
                400.0,
                None,
            );
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

    pub fn list_row_clipped(
        &mut self,
        r: Rect,
        label: &str,
        selected: bool,
        clip: Option<Rect>,
    ) -> bool {
        let t = self.theme;
        let hov = self.is_hovered(r);
        // Apple sidebar row: solid accent capsule when selected, faint wash
        // on hover, nothing otherwise.
        let bg = if selected {
            theme::ACCENT
        } else if hov {
            Color(255, 255, 255, 16)
        } else {
            Color(0, 0, 0, 0)
        };
        push_rrect_clipped(&mut self.items, r, t.px(theme::CORNER_SM), bg, clip);

        let fg = if selected {
            theme::TEXT_ON_ACCENT
        } else {
            theme::TEXT_PRIMARY
        };
        let lx = r[0] + t.px(theme::PAD);
        let ly = r[1] + (r[3] - t.body()) * 0.5;
        let bounds = clip.map(|c| (c[0], c[1], c[0] + c[2], c[1] + c[3]));
        self.push_label(
            (lx, ly),
            label,
            t.body(),
            fg,
            Align::Left,
            r[2] - t.px(theme::PAD * 2.0),
            bounds,
        );
        self.just_clicked(r)
    }

    pub fn disclosure(&mut self, r: Rect, open: bool, label: &str) -> bool {
        let t = self.theme;
        let chevron = if open { "\u{25be}" } else { "\u{25b8}" };
        let ch_x = r[0] + t.px(4.0);
        let cy = r[1] + (r[3] - t.body()) * 0.5;
        self.push_label(
            (ch_x, cy),
            chevron,
            t.body(),
            theme::TEXT_SECONDARY,
            Align::Left,
            t.px(16.0),
            None,
        );

        let lx = r[0] + t.px(20.0);
        self.push_label(
            (lx, cy),
            label,
            t.body(),
            theme::TEXT_PRIMARY,
            Align::Left,
            r[2] - t.px(24.0),
            None,
        );

        if self.just_clicked(r) {
            !open
        } else {
            open
        }
    }

    pub fn tabs(&mut self, r: Rect, selected: usize, labels: &[&str]) -> Option<usize> {
        self.segmented(r, selected, labels)
    }

    /// Apple-style segmented control: a recessed track with one raised chip.
    /// Pass an out-of-range `selected` for a control with no active segment.
    /// Returns the newly selected index when a different segment is clicked.
    pub fn segmented(&mut self, r: Rect, selected: usize, labels: &[&str]) -> Option<usize> {
        let t = self.theme;
        let radius = t.px(7.0);
        push_rrect(&mut self.items, r, radius, theme::CONTROL_ACTIVE);

        let n = labels.len().max(1);
        let seg_w = r[2] / n as f32;
        let inset = t.px(2.0);
        let mut result = None;
        for (i, label) in labels.iter().enumerate() {
            let seg_r = [r[0] + i as f32 * seg_w, r[1], seg_w, r[3]];
            let chip = [
                seg_r[0] + inset,
                seg_r[1] + inset,
                seg_r[2] - inset * 2.0,
                seg_r[3] - inset * 2.0,
            ];
            let sel = i == selected;
            let hov = self.is_hovered(seg_r);
            if sel {
                push_rrect(&mut self.items, chip, radius - inset, theme::CONTROL_HOVER);
                push_border(
                    &mut self.items,
                    chip,
                    radius - inset,
                    Color(255, 255, 255, 26),
                    t.px(1.0),
                );
            } else if hov {
                push_rrect(&mut self.items, chip, radius - inset, Color(255, 255, 255, 12));
            }
            // Hairline between two segments only when neither side is the
            // raised chip, like NSSegmentedControl.
            if i > 0 && !sel && selected != i - 1 {
                let sep_inset = t.px(7.0);
                push_rect(
                    &mut self.items,
                    [
                        seg_r[0],
                        seg_r[1] + sep_inset,
                        t.px(1.0),
                        seg_r[3] - sep_inset * 2.0,
                    ],
                    Color(255, 255, 255, 24),
                );
            }

            let fg = if sel {
                theme::TEXT_PRIMARY
            } else {
                theme::TEXT_SECONDARY
            };
            self.push_center_label(seg_r, label, t.body(), fg);

            if self.just_clicked(seg_r) && !sel {
                result = Some(i);
            }
        }
        result
    }
}

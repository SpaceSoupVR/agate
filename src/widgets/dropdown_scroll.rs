use space_soup::ui2d::{Align, Area, Color, Item, Shape, ShapeType};

use crate::theme;

use super::draw::{push_border, push_rect, push_rrect};
use super::{in_rect, PendingTooltip, Rect, Ui, WidgetId};

impl Ui {
    pub fn dropdown(
        &mut self,
        id: WidgetId,
        r: Rect,
        selected: usize,
        options: &[&str],
    ) -> Option<usize> {
        let t = self.theme;
        let open = self
            .state
            .get(&id)
            .map(|s| s.dropdown_open)
            .unwrap_or(false);
        let hovered = self.is_hovered(r);

        let bg = if hovered {
            theme::CONTROL_HOVER
        } else {
            theme::CONTROL_BG
        };
        push_rrect(&mut self.items, r, t.px(theme::CORNER), bg);
        push_border(
            &mut self.items,
            r,
            t.px(theme::CORNER),
            theme::CONTROL_BORDER,
            t.px(1.0),
        );

        let label = options.get(selected).copied().unwrap_or("\u{2014}");
        let pad = t.px(theme::PAD_SM);
        let font_px = t.body();
        let ly = r[1] + (r[3] - font_px) * 0.5;
        self.push_label(
            (r[0] + pad, ly),
            label,
            font_px,
            theme::TEXT_PRIMARY,
            Align::Left,
            r[2] - pad * 3.0,
            None,
        );

        let chevron = if open { "\u{25b2}" } else { "\u{25bc}" };
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

            push_rrect(
                &mut self.items,
                [lx, ly, lw, list_h],
                t.px(theme::CORNER),
                theme::SURFACE_RAISED,
            );
            push_border(
                &mut self.items,
                [lx, ly, lw, list_h],
                t.px(theme::CORNER),
                theme::BORDER,
                t.px(1.0),
            );

            for (i, opt) in options.iter().enumerate() {
                let item_r = [lx, ly + i as f32 * item_h, lw, item_h];
                let ih = self.is_hovered(item_r);
                let sel = i == selected;
                let item_bg = if sel {
                    theme::ACCENT_DIM
                } else if ih {
                    theme::CONTROL_HOVER
                } else {
                    Color(0, 0, 0, 0)
                };
                push_rect(&mut self.items, item_r, item_bg);

                let text_color = if sel {
                    theme::ACCENT_HI
                } else {
                    theme::TEXT_PRIMARY
                };
                let pad2 = t.px(theme::PAD);
                let ty = item_r[1] + (item_h - t.body()) * 0.5;
                self.push_label(
                    (item_r[0] + pad2, ty),
                    opt,
                    t.body(),
                    text_color,
                    Align::Left,
                    lw - pad2 * 2.0,
                    None,
                );

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

    pub fn scroll_area(&mut self, id: WidgetId, r: Rect, content_height: f32) -> (Rect, f32) {
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

    pub fn end_scroll_area(&mut self, id: WidgetId, r: Rect, content_height: f32) {
        let t = self.theme;
        if content_height <= r[3] {
            return;
        }

        let scroll_y = self.state.get(&id).map(|s| s.scroll_y).unwrap_or(0.0);
        let track_h = r[3] - t.px(8.0);
        let thumb_h = (track_h * r[3] / content_height).max(t.px(24.0));
        let max_sc = (content_height - r[3]).max(1.0);
        let frac = (scroll_y / max_sc).clamp(0.0, 1.0);
        let thumb_y = r[1] + t.px(4.0) + frac * (track_h - thumb_h);

        self.items.push((
            Area {
                offset: (r[0] + r[2] - t.px(8.0), thumb_y),
                bounds: None,
            },
            Item::Shape(Shape {
                shape: ShapeType::RoundedRectangle(0.0, (t.px(4.0), thumb_h), 0.0, t.px(2.0)),
                color: theme::SCROLLBAR,
            }),
        ));
    }

    pub fn tooltip(&mut self, trigger_rect: Rect, label: &str) {
        if self.is_hovered(trigger_rect) {
            self.tooltip = Some(PendingTooltip {
                rect: trigger_rect,
                label: label.to_string(),
            });
        }
    }
}

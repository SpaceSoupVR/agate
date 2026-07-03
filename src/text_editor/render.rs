use std::sync::Arc;

use space_soup::ui2d::{Align, Area, Color, Font, Item, Shape, ShapeType, Span, Text};

use crate::theme::{self, Theme};

use super::highlight::{highlight_json_line, plain_line};
use super::{char_len, digit_count, EditorGeom, HScrollGeom, TextEditor};

impl TextEditor {
    pub fn build_items(
        &mut self,
        rect: (f32, f32, f32, f32),
        theme: &Theme,
        font: &Arc<Font>,
        show_caret: bool,
        focused: bool,
        items: &mut Vec<(Area, Item)>,
    ) {
        let (rx, ry, rw, rh) = rect;
        let font_px = theme.font(theme::PT_EDITOR);
        let line_h = theme.px(theme::PT_EDITOR_LH);
        let char_w = font.metrics('0', font_px).advance_width.max(theme.px(6.0));

        let digits = digit_count(self.lines.len());
        let gutter_w = char_w * digits as f32 + theme.px(20.0);

        // Reserve space for horizontal scrollbar at the bottom
        let h_scrollbar_h = theme.px(8.0);
        let max_line_chars = self.lines.iter().map(|l| char_len(l)).max().unwrap_or(0);
        let content_w = max_line_chars as f32 * char_w;
        let text_viewport_w = rw - gutter_w - theme.px(8.0);
        let needs_hscroll = content_w > text_viewport_w;
        let effective_rh = if needs_hscroll { rh - h_scrollbar_h } else { rh };

        let text_x = rx + gutter_w + theme.px(8.0);
        let top_y = ry + theme.px(6.0);
        let vis_rows = (((effective_rh - theme.px(12.0)) / line_h).floor() as usize).max(1);

        let geom = EditorGeom {
            rect: (rx, ry, rw, effective_rh),
            text_x,
            top_y,
            char_w,
            line_h,
            visible_rows: vis_rows,
        };
        self.last_geom = Some(geom);

        // Clamp scroll positions
        let max_scroll_row = self.lines.len().saturating_sub(1);
        self.scroll_row = self.scroll_row.min(max_scroll_row);

        let max_scroll_col = if needs_hscroll {
            let visible_cols = (text_viewport_w / char_w).floor() as usize;
            max_line_chars.saturating_sub(visible_cols)
        } else {
            0
        };
        self.scroll_col = self.scroll_col.min(max_scroll_col);

        // ── Background ──────────────────────────────────────────────────────
        push_rect(items, (rx, ry), (rw, rh), theme::EDITOR_BG);
        push_rect(items, (rx, ry), (gutter_w, rh), theme::GUTTER_BG);
        push_rect(
            items,
            (rx + gutter_w - theme.px(1.0), ry),
            (theme.px(1.0), effective_rh),
            theme::BORDER,
        );

        let sel = self.selection_range();
        let last_row = (self.scroll_row + vis_rows).min(self.lines.len());

        // Clip rect for the text area (right of gutter, above h-scrollbar)
        let text_clip = (rx + gutter_w, ry, rx + rw, ry + effective_rh);

        for (vi, row) in (self.scroll_row..last_row).enumerate() {
            let ly = top_y + vi as f32 * line_h;

            // Current line highlight (when no selection)
            if sel.is_none() && row == self.cursor.row {
                push_rect(
                    items,
                    (rx + gutter_w, ly),
                    (rw - gutter_w, line_h),
                    theme::CURRENT_LINE,
                );
            }

            // Selection highlight
            if let Some((s, e)) = sel {
                if row >= s.row && row <= e.row {
                    let sc = if row == s.row { s.col } else { 0 };
                    let ec = if row == e.row {
                        e.col
                    } else {
                        char_len(&self.lines[row]) + 1
                    };

                    // Clamp selection rect to visible columns
                    let vis_sc = sc.saturating_sub(self.scroll_col);
                    let vis_ec = ec.saturating_sub(self.scroll_col);
                    let sx = text_x + vis_sc as f32 * char_w;
                    let ex = text_x + vis_ec as f32 * char_w;
                    let w = (ex - sx).max(char_w * 0.35);

                    // Only draw if selection is within text viewport
                    let clip_right = rx + rw;
                    if sx < clip_right && ex > text_x - char_w {
                        let clamped_sx = sx.max(rx + gutter_w);
                        let clamped_w = (ex.min(clip_right) - clamped_sx).max(0.0);
                        if clamped_w > 0.0 {
                            push_rect(items, (clamped_sx, ly), (clamped_w, line_h), theme::SELECTION_BG);
                        }
                    }
                }
            }

            // Gutter line number
            let num_color = if row == self.cursor.row {
                theme::LINE_NUMBER_CUR
            } else {
                theme::LINE_NUMBER
            };
            let num = format!("{:>w$}", row + 1, w = digits);
            items.push((
                Area {
                    offset: (rx + theme.px(8.0), ly),
                    bounds: Some((rx, ry, rx + gutter_w, ry + rh)),
                },
                Item::Text(Text::new(
                    vec![Span::new(num, font.clone(), font_px, num_color)
                        .with_align(Align::Left)],
                    gutter_w,
                )),
            ));

            // Line text with horizontal scroll and clipping
            let spans = if self.syntax {
                highlight_json_line(&self.lines[row], self.scroll_col, font, font_px)
            } else {
                plain_line(&self.lines[row], self.scroll_col, font, font_px)
            };
            if !spans.is_empty() {
                items.push((
                    Area {
                        offset: (text_x, ly),
                        bounds: Some(text_clip),
                    },
                    Item::Text(Text::new(spans, 1.0e6)),
                ));
            }

            // Caret
            if show_caret && focused && row == self.cursor.row && sel.is_none() {
                let vis_col = self.cursor.col.saturating_sub(self.scroll_col);
                let cx = text_x + vis_col as f32 * char_w;
                // Only render caret if it's inside the text viewport
                if cx >= rx + gutter_w && cx < rx + rw {
                    push_rect(
                        items,
                        (cx, ly + theme.px(1.0)),
                        (theme.px(1.5), line_h - theme.px(2.0)),
                        theme::CARET,
                    );
                }
            }
        }

        // ── Vertical scrollbar ───────────────────────────────────────────────
        if self.lines.len() > vis_rows {
            let track_h = effective_rh - theme.px(8.0);
            let thumb_h =
                (track_h * vis_rows as f32 / self.lines.len() as f32).max(theme.px(24.0));
            let max_sc = (self.lines.len() - vis_rows) as f32;
            let t = if max_sc > 0.0 { self.scroll_row as f32 / max_sc } else { 0.0 };
            let thumb_y = ry + theme.px(4.0) + t * (track_h - thumb_h);
            items.push((
                Area {
                    offset: (rx + rw - theme.px(8.0), thumb_y),
                    bounds: None,
                },
                Item::Shape(Shape {
                    shape: ShapeType::RoundedRectangle(0.0, (theme.px(4.0), thumb_h), 0.0, theme.px(2.0)),
                    color: theme::SCROLLBAR,
                }),
            ));
        }

        // ── Horizontal scrollbar ─────────────────────────────────────────────
        self.last_hscroll = None;
        if needs_hscroll {
            let bar_y = ry + rh - h_scrollbar_h;
            // Track background
            push_rect(
                items,
                (rx + gutter_w, bar_y),
                (rw - gutter_w, h_scrollbar_h),
                theme::GUTTER_BG,
            );
            // Separator line
            push_rect(
                items,
                (rx + gutter_w, bar_y),
                (rw - gutter_w, theme.px(1.0)),
                theme::BORDER,
            );

            self.last_hscroll = Some(HScrollGeom {
                track_x: rx + gutter_w,
                track_w: rw - gutter_w,
                bar_y,
                bar_h: h_scrollbar_h,
                max_scroll_col,
            });

            let track_w = rw - gutter_w - theme.px(8.0);
            let visible_cols = (text_viewport_w / char_w).floor() as usize;
            let thumb_frac = visible_cols as f32 / max_line_chars as f32;
            let thumb_w = (track_w * thumb_frac).max(theme.px(32.0)).min(track_w);
            let t = if max_scroll_col > 0 {
                self.scroll_col as f32 / max_scroll_col as f32
            } else {
                0.0
            };
            let thumb_x = rx + gutter_w + theme.px(4.0) + t * (track_w - thumb_w);
            let thumb_y = bar_y + (h_scrollbar_h - theme.px(4.0)) * 0.5;
            items.push((
                Area {
                    offset: (thumb_x, thumb_y),
                    bounds: None,
                },
                Item::Shape(Shape {
                    shape: ShapeType::RoundedRectangle(
                        0.0,
                        (thumb_w, theme.px(4.0)),
                        0.0,
                        theme.px(2.0),
                    ),
                    color: theme::SCROLLBAR,
                }),
            ));
        }
    }
}

fn push_rect(items: &mut Vec<(Area, Item)>, offset: (f32, f32), size: (f32, f32), color: Color) {
    items.push((
        Area { offset, bounds: None },
        Item::Shape(Shape {
            shape: ShapeType::Rectangle(0.0, size, 0.0),
            color,
        }),
    ));
}
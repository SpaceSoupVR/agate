//! `Ui` is an immediate-mode widget layer: call `begin_frame`, draw widgets
//! (each call both draws and returns any interaction that happened), then
//! `finish()` to get the flat `(Area, Item)` list to hand to the renderer.
//!
//! Split into sibling modules by widget family — `buttons`, `sliders`,
//! `text_input`, `dropdown_scroll` — all as `impl Ui` blocks here in
//! `mod.rs`. Shared draw-primitive helpers (`push_rect`, `push_rrect`, …)
//! live in `draw.rs`.

mod buttons;
mod draw;
mod dropdown_scroll;
mod sliders;
mod text_input;

use std::collections::HashMap;
use std::sync::Arc;

use space_soup::ui2d::{Align, Area, Color, Font, Item, Span, Text};

use crate::input::UiInput;
use crate::theme::{self, Theme};

use draw::{push_border, push_rect, push_rrect, push_rrect_clipped};

pub type Rect = [f32; 4];

pub(crate) fn in_rect(p: (f32, f32), r: Rect) -> bool {
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
pub(crate) struct WidgetState {
    pub(crate) slider_drag: Option<(f32, f32)>,
    pub(crate) dropdown_open: bool,
    pub(crate) scroll_y: f32,
    pub(crate) text: Option<String>,
    pub(crate) text_cursor: usize,
    pub(crate) text_anchor: Option<usize>,
}

pub(crate) struct PendingTooltip {
    pub(crate) rect: Rect,
    pub(crate) label: String,
}

pub struct Ui {
    pub theme: Theme,
    pub font: Arc<Font>,
    pub(crate) items: Vec<(Area, Item)>,
    pub(crate) state: HashMap<WidgetId, WidgetState>,
    pub(crate) input: UiInput,
    pub(crate) win_w: f32,
    pub(crate) win_h: f32,
    pub(crate) drag_owner: Option<WidgetId>,
    pub(crate) focused: Option<WidgetId>,
    pub(crate) elapsed: f32,
    pub(crate) tooltip: Option<PendingTooltip>,
}

impl Ui {
    pub fn new(scale: f32, font: Arc<Font>) -> Self {
        Self {
            theme: Theme::new(scale),
            font,
            items: Vec::with_capacity(256),
            state: HashMap::new(),
            input: UiInput::default(),
            win_w: 0.0,
            win_h: 0.0,
            drag_owner: None,
            focused: None,
            elapsed: 0.0,
            tooltip: None,
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

    pub fn is_hovered(&self, r: Rect) -> bool {
        in_rect(self.input.mouse_pos, r)
    }

    pub(crate) fn just_clicked(&self, r: Rect) -> bool {
        self.input.left_just_released() && in_rect(self.input.mouse_pos, r)
    }

    pub(crate) fn push_label(
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

    pub(crate) fn push_center_label(&mut self, r: Rect, text: &str, size_px: f32, color: Color) {
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

    /// Like `panel`, but clamped to `clip` (when given) — used for panels
    /// inside a scrollable area, where a row near the container's edge must
    /// not paint over whatever sits just outside it.
    pub fn panel_clipped(&mut self, r: Rect, color: Color, clip: Option<Rect>) {
        let radius = self.theme.px(theme::CORNER);
        push_rrect_clipped(&mut self.items, r, radius, color, clip);
    }

    pub fn card(&mut self, r: Rect) {
        let t = &self.theme;
        push_rrect(&mut self.items, r, t.px(theme::CORNER_LG), theme::SURFACE);
        push_border(&mut self.items, r, t.px(theme::CORNER_LG), theme::BORDER, t.px(1.0));
    }

    /// Like `card`, but with a caller-chosen fill color instead of the fixed
    /// `SURFACE` — fill and border share `CORNER_LG` so they align exactly,
    /// unlike combining `panel` (smaller `CORNER` radius) with `card_border`.
    pub fn panel_bordered(&mut self, r: Rect, color: Color) {
        let t = &self.theme;
        push_rrect(&mut self.items, r, t.px(theme::CORNER_LG), color);
        push_border(&mut self.items, r, t.px(theme::CORNER_LG), theme::BORDER, t.px(1.0));
    }

    pub fn card_border(&mut self, r: Rect) {
        let t = &self.theme;
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
}
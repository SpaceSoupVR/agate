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

pub fn in_rect(p: (f32, f32), r: Rect) -> bool {
    p.0 >= r[0] && p.0 <= r[0] + r[2] && p.1 >= r[1] && p.1 <= r[1] + r[3]
}

/// Overlap of two (x, y, w, h) rects. A zero (or negative) size means they
/// don't overlap — fine for clipping, since nothing then passes the bounds test.
fn intersect_rect(a: Rect, b: Rect) -> Rect {
    let x = a[0].max(b[0]);
    let y = a[1].max(b[1]);
    let w = (a[0] + a[2]).min(b[0] + b[2]) - x;
    let h = (a[1] + a[3]).min(b[1] + b[3]) - y;
    [x, y, w.max(0.0), h.max(0.0)]
}

/// Same as [`intersect_rect`] for `Area.bounds` tuples `(x, y, w, h)`.
fn intersect_bounds(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
    let r = intersect_rect([a.0, a.1, a.2, a.3], [b.0, b.1, b.2, b.3]);
    (r[0], r[1], r[2], r[3])
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
    /// Open clip scopes. Each entry is `(first item index in the scope, clip
    /// rect in (x, y, w, h))`. `clip` below is their running intersection.
    clip_scopes: Vec<(usize, Rect)>,
    /// Intersection of every open clip scope (`None` = unclipped). Used to
    /// hit-test: a widget scrolled outside the panel is both hidden and inert.
    clip: Option<Rect>,
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
            clip_scopes: Vec::new(),
            clip: None,
        }
    }

    /// Confine everything drawn until the matching `pop_clip` to `r` (an
    /// (x, y, w, h) rect, e.g. a scroll panel). Scopes nest — the effective clip
    /// is their intersection — and both hide and disable widgets outside it.
    pub fn push_clip(&mut self, r: Rect) {
        self.clip_scopes.push((self.items.len(), r));
        self.clip = Some(match self.clip {
            Some(c) => intersect_rect(c, r),
            None => r,
        });
    }

    /// Close the most recent clip scope, intersecting every item drawn inside it
    /// with the scope rect so none of it can escape the region. `Area.bounds` is
    /// (x, y, w, h); an item with no bounds of its own simply takes the scope's.
    pub fn pop_clip(&mut self) {
        let Some((start, r)) = self.clip_scopes.pop() else {
            return;
        };
        let scope = (r[0], r[1], r[2], r[3]);
        for (area, _) in self.items[start..].iter_mut() {
            area.bounds = Some(match area.bounds {
                Some(b) => intersect_bounds(b, scope),
                None => scope,
            });
        }
        // Rebuild the running clip from whatever scopes remain open.
        self.clip = self
            .clip_scopes
            .iter()
            .map(|(_, r)| *r)
            .reduce(intersect_rect);
    }

    /// The help label of whatever `tooltip`-registered widget is hovered this
    /// frame, if any. Lets a caller render the hint in a fixed info box instead
    /// of the floating popup — pair with `clear_tooltip` to suppress the popup.
    pub fn hovered_hint(&self) -> Option<String> {
        self.tooltip.as_ref().map(|t| t.label.clone())
    }

    /// Drop the pending floating tooltip so `finish` won't draw it.
    pub fn clear_tooltip(&mut self) {
        self.tooltip = None;
    }

    pub fn begin_frame(&mut self, win_w: f32, win_h: f32, input: &UiInput) {
        self.win_w = win_w;
        self.win_h = win_h;
        self.input = input.clone();
        self.items.clear();
        self.tooltip = None;
        self.clip_scopes.clear();
        self.clip = None;
        self.elapsed += input.dt;

        if input.left_just_released() {
            self.drag_owner = None;
        }
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

        let mx = (tt.rect[0] + tt.rect[2] * 0.5 - bw * 0.5).clamp(0.0, (self.win_w - bw).max(0.0));
        let my = (tt.rect[1] - bh - t.px(6.0)).max(0.0);

        push_rrect(
            &mut self.items,
            [mx, my, bw, bh],
            t.px(theme::CORNER_SM),
            theme::SURFACE_RAISED,
        );
        push_border(
            &mut self.items,
            [mx, my, bw, bh],
            t.px(theme::CORNER_SM),
            theme::BORDER,
            t.px(1.0),
        );
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

    /// Whether the pointer is inside the active clip region (always true when
    /// unclipped). A widget scrolled out of a `push_clip` panel is hidden, so it
    /// must not react to hover/clicks either — otherwise it would steal input
    /// from whatever is drawn in its place (e.g. the bar above the panel).
    pub(crate) fn pointer_in_clip(&self) -> bool {
        self.clip.map_or(true, |c| in_rect(self.input.mouse_pos, c))
    }

    pub fn is_hovered(&self, r: Rect) -> bool {
        in_rect(self.input.mouse_pos, r) && self.pointer_in_clip()
    }

    /// True while any text input widget has keyboard focus. Lets callers
    /// suppress single-key hotkeys (Space, Delete, ...) while the user types.
    pub fn text_focused(&self) -> bool {
        self.focused.is_some()
    }

    pub(crate) fn just_clicked(&self, r: Rect) -> bool {
        self.input.left_just_released() && in_rect(self.input.mouse_pos, r) && self.pointer_in_clip()
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
            Area {
                offset,
                bounds: clip,
            },
            Item::Text(Text::new(vec![span], max_w.max(1.0))),
        ));
    }

    pub(crate) fn push_center_label(&mut self, r: Rect, text: &str, size_px: f32, color: Color) {
        let text_w: f32 = text
            .chars()
            .map(|c| self.font.metrics(c, size_px).advance_width)
            .sum();
        let x = r[0] + (r[2] - text_w) * 0.5;
        let y = r[1] + (r[3] - size_px) * 0.5;
        let span =
            Span::new(text.to_string(), self.font.clone(), size_px, color).with_align(Align::Left);
        self.items.push((
            Area {
                offset: (x, y),
                // (x, y, w, h) — the button rect. Any active clip scope is
                // applied to this (and every item) when the scope closes.
                bounds: Some((r[0], r[1], r[2], r[3])),
            },
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

    pub fn panel_clipped(&mut self, r: Rect, color: Color, clip: Option<Rect>) {
        let radius = self.theme.px(theme::CORNER);
        push_rrect_clipped(&mut self.items, r, radius, color, clip);
    }

    pub fn card(&mut self, r: Rect) {
        let t = &self.theme;
        push_rrect(&mut self.items, r, t.px(theme::CORNER_LG), theme::SURFACE);
        push_border(
            &mut self.items,
            r,
            t.px(theme::CORNER_LG),
            theme::BORDER,
            t.px(1.0),
        );
    }

    pub fn panel_bordered(&mut self, r: Rect, color: Color) {
        let t = &self.theme;
        push_rrect(&mut self.items, r, t.px(theme::CORNER_LG), color);
        push_border(
            &mut self.items,
            r,
            t.px(theme::CORNER_LG),
            theme::BORDER,
            t.px(1.0),
        );
    }

    pub fn card_border(&mut self, r: Rect) {
        let t = &self.theme;
        push_border(
            &mut self.items,
            r,
            t.px(theme::CORNER_LG),
            theme::BORDER,
            t.px(1.0),
        );
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
        self.label_styled(
            x,
            y,
            text,
            self.theme.body(),
            theme::TEXT_PRIMARY,
            self.win_w - x,
            None,
        );
    }

    pub fn label_secondary(&mut self, x: f32, y: f32, text: &str) {
        let size = self.theme.small();
        self.label_styled(
            x,
            y,
            text,
            size,
            theme::TEXT_SECONDARY,
            self.win_w - x,
            None,
        );
    }

    pub fn label_styled(
        &mut self,
        x: f32,
        y: f32,
        text: &str,
        size_px: f32,
        color: Color,
        max_w: f32,
        clip: Option<Rect>,
    ) {
        // `Area.bounds` is (x, y, w, h) — hand the clip rect over unchanged.
        let bounds = clip.map(|r| (r[0], r[1], r[2], r[3]));
        self.push_label((x, y), text, size_px, color, Align::Left, max_w, bounds);
    }

    pub fn label_clipped(&mut self, x: f32, y: f32, text: &str, clip: Rect) {
        let size = self.theme.body();
        let max_w = clip[2];
        self.label_styled(x, y, text, size, theme::TEXT_PRIMARY, max_w, Some(clip));
    }
}

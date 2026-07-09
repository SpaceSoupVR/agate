use crate::widgets::Rect;

pub fn rect_from(x: f32, y: f32, w: f32, h: f32) -> Rect {
    [x, y, w, h]
}

/// A rect being carved into non-overlapping pieces. Every split returns the
/// slice taken plus a remainder whose origin is shifted past `slice + gap` —
/// there is no arithmetic path from these methods to an overlapping output.
#[derive(Clone, Copy)]
pub struct Region(Rect);

impl Region {
    pub fn new(r: Rect) -> Self {
        Self(r)
    }

    pub fn rect(&self) -> Rect {
        self.0
    }

    pub fn inset(&self, m: f32) -> Self {
        let r = self.0;
        Self(rect_from(
            r[0] + m,
            r[1] + m,
            (r[2] - 2.0 * m).max(0.0),
            (r[3] - 2.0 * m).max(0.0),
        ))
    }

    pub fn split_top(&self, amount: f32) -> (Rect, Self) {
        self.split_top_gap(amount, 0.0)
    }

    pub fn split_top_gap(&self, amount: f32, gap: f32) -> (Rect, Self) {
        let r = self.0;
        let amount = amount.clamp(0.0, r[3]);
        let slice = rect_from(r[0], r[1], r[2], amount);
        let rest = rect_from(r[0], r[1] + amount + gap, r[2], (r[3] - amount - gap).max(0.0));
        (slice, Self(rest))
    }

    pub fn split_bottom(&self, amount: f32) -> (Rect, Self) {
        self.split_bottom_gap(amount, 0.0)
    }

    pub fn split_bottom_gap(&self, amount: f32, gap: f32) -> (Rect, Self) {
        let r = self.0;
        let amount = amount.clamp(0.0, r[3]);
        let slice = rect_from(r[0], r[1] + r[3] - amount, r[2], amount);
        let rest = rect_from(r[0], r[1], r[2], (r[3] - amount - gap).max(0.0));
        (slice, Self(rest))
    }

    pub fn split_left(&self, amount: f32) -> (Rect, Self) {
        self.split_left_gap(amount, 0.0)
    }

    pub fn split_left_gap(&self, amount: f32, gap: f32) -> (Rect, Self) {
        let r = self.0;
        let amount = amount.clamp(0.0, r[2]);
        let slice = rect_from(r[0], r[1], amount, r[3]);
        let rest = rect_from(r[0] + amount + gap, r[1], (r[2] - amount - gap).max(0.0), r[3]);
        (slice, Self(rest))
    }

    pub fn split_right(&self, amount: f32) -> (Rect, Self) {
        self.split_right_gap(amount, 0.0)
    }

    pub fn split_right_gap(&self, amount: f32, gap: f32) -> (Rect, Self) {
        let r = self.0;
        let amount = amount.clamp(0.0, r[2]);
        let slice = rect_from(r[0] + r[2] - amount, r[1], amount, r[3]);
        let rest = rect_from(r[0], r[1], (r[2] - amount - gap).max(0.0), r[3]);
        (slice, Self(rest))
    }
}

enum Axis {
    Row,
    RowFromRight,
    Column,
}

/// A cursor walking one axis of a fixed cross-size. `.take()` returns the
/// next cell and advances monotonically past `len + gap`, so a sequence of
/// takes on one `Flow` cannot produce overlapping cells.
pub struct Flow {
    cursor: f32,
    cross_origin: f32,
    cross_len: f32,
    gap: f32,
    axis: Axis,
}

impl Flow {
    pub fn row(x: f32, y: f32, h: f32, gap: f32) -> Self {
        Self {
            cursor: x,
            cross_origin: y,
            cross_len: h,
            gap,
            axis: Axis::Row,
        }
    }

    pub fn row_from_right(right_x: f32, y: f32, h: f32, gap: f32) -> Self {
        Self {
            cursor: right_x,
            cross_origin: y,
            cross_len: h,
            gap,
            axis: Axis::RowFromRight,
        }
    }

    pub fn column(x: f32, y: f32, w: f32, gap: f32) -> Self {
        Self {
            cursor: y,
            cross_origin: x,
            cross_len: w,
            gap,
            axis: Axis::Column,
        }
    }

    pub fn take(&mut self, len: f32) -> Rect {
        match self.axis {
            Axis::Row => {
                let r = rect_from(self.cursor, self.cross_origin, len, self.cross_len);
                self.cursor += len + self.gap;
                r
            }
            Axis::RowFromRight => {
                self.cursor -= len;
                let r = rect_from(self.cursor, self.cross_origin, len, self.cross_len);
                self.cursor -= self.gap;
                r
            }
            Axis::Column => {
                let r = rect_from(self.cross_origin, self.cursor, self.cross_len, len);
                self.cursor += len + self.gap;
                r
            }
        }
    }

    /// Atomically builds the rect for `variants[active]` at the current
    /// cursor position and advances by that variant's length — for
    /// mutually-exclusive alternatives (only one of which is ever drawn)
    /// that share one slot. A separate peek/advance pair would let a future
    /// caller build a candidate rect and advance by the wrong length (or
    /// forget to advance at all); this can't.
    pub fn take_variant(&mut self, variants: &[f32], active: usize) -> Rect {
        self.take(variants[active])
    }
}

fn rects_overlap(a: Rect, b: Rect) -> bool {
    a[0] < b[0] + b[2] && a[0] + a[2] > b[0] && a[1] < b[1] + b[3] && a[1] + a[3] > b[1]
}

/// The explicit-crash backstop for rects that can't be made structurally
/// non-overlapping via `Region`/`Flow` — e.g. independently-anchored
/// floating regions that share no common parent split. Claiming an
/// intentionally-nested rect (a button drawn on its own backdrop) against
/// its container is a caller error, not a `Region`/`Flow` gap — only claim
/// true siblings into the same guard.
#[derive(Default)]
pub struct OverlapGuard {
    claims: Vec<(&'static str, Rect)>,
}

impl OverlapGuard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn claim(&mut self, name: &'static str, r: Rect) -> Rect {
        debug_assert!(
            r[2] >= 0.0 && r[3] >= 0.0,
            "layout: '{name}' has a negative dimension {r:?}"
        );
        for (other_name, other) in &self.claims {
            debug_assert!(
                !rects_overlap(r, *other),
                "layout: '{name}' {r:?} overlaps '{other_name}' {other:?}"
            );
        }
        self.claims.push((name, r));
        r
    }

    pub fn claim_group(&mut self, name: &'static str, rs: &[Rect]) {
        for r in rs {
            self.claim(name, *r);
        }
    }
}

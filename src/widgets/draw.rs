use space_soup::ui2d::{Area, Color, Font, Item, Shape, ShapeType};

use super::Rect;

type ClipBounds = Option<(f32, f32, f32, f32)>;

fn to_bounds(clip: Option<Rect>) -> ClipBounds {
    clip.map(|r| (r[0], r[1], r[0] + r[2], r[1] + r[3]))
}

pub(crate) fn push_rect(items: &mut Vec<(Area, Item)>, r: Rect, color: Color) {
    push_rect_clipped(items, r, color, None);
}

pub(crate) fn push_rect_clipped(
    items: &mut Vec<(Area, Item)>,
    r: Rect,
    color: Color,
    clip: Option<Rect>,
) {
    items.push((
        Area {
            offset: (r[0], r[1]),
            bounds: to_bounds(clip),
        },
        Item::Shape(Shape {
            shape: ShapeType::Rectangle(0.0, (r[2], r[3]), 0.0),
            color,
        }),
    ));
}

pub(crate) fn push_rrect(items: &mut Vec<(Area, Item)>, r: Rect, radius: f32, color: Color) {
    push_rrect_clipped(items, r, radius, color, None);
}

pub(crate) fn push_rrect_clipped(
    items: &mut Vec<(Area, Item)>,
    r: Rect,
    radius: f32,
    color: Color,
    clip: Option<Rect>,
) {
    items.push((
        Area {
            offset: (r[0], r[1]),
            bounds: to_bounds(clip),
        },
        Item::Shape(Shape {
            shape: ShapeType::RoundedRectangle(0.0, (r[2], r[3]), 0.0, radius),
            color,
        }),
    ));
}

pub(crate) fn push_border(
    items: &mut Vec<(Area, Item)>,
    r: Rect,
    radius: f32,
    color: Color,
    stroke: f32,
) {
    push_border_clipped(items, r, radius, color, stroke, None);
}

pub(crate) fn push_border_clipped(
    items: &mut Vec<(Area, Item)>,
    r: Rect,
    radius: f32,
    color: Color,
    stroke: f32,
    clip: Option<Rect>,
) {
    items.push((
        Area {
            offset: (r[0], r[1]),
            bounds: to_bounds(clip),
        },
        Item::Shape(Shape {
            shape: ShapeType::RoundedRectangle(stroke, (r[2], r[3]), 0.0, radius),
            color,
        }),
    ));
}

pub(crate) fn push_ellipse_outline(
    items: &mut Vec<(Area, Item)>,
    r: Rect,
    stroke: f32,
    color: Color,
) {
    items.push((
        Area {
            offset: (r[0], r[1]),
            bounds: None,
        },
        Item::Shape(Shape {
            shape: ShapeType::Ellipse(stroke, (r[2], r[3]), 0.0),
            color,
        }),
    ));
}

pub(crate) fn push_ellipse_outline_rotated(
    items: &mut Vec<(Area, Item)>,
    r: Rect,
    stroke: f32,
    color: Color,
    angle_deg: f32,
) {
    items.push((
        Area {
            offset: (r[0], r[1]),
            bounds: None,
        },
        Item::Shape(Shape {
            shape: ShapeType::Ellipse(stroke, (r[2], r[3]), angle_deg),
            color,
        }),
    ));
}

pub(crate) fn brighten_color(c: Color, factor: f32) -> Color {
    Color(
        (c.0 as f32 * factor).min(255.0) as u8,
        (c.1 as f32 * factor).min(255.0) as u8,
        (c.2 as f32 * factor).min(255.0) as u8,
        c.3,
    )
}

pub(crate) fn dim_color(c: Color, factor: f32) -> Color {
    brighten_color(c, factor)
}

pub(crate) fn byte_idx_of(s: &str, col: usize) -> usize {
    s.char_indices().nth(col).map(|(i, _)| i).unwrap_or(s.len())
}

pub(crate) fn text_advance_width(s: &str, up_to_col: usize, font: &Font, size_px: f32) -> f32 {
    s.chars()
        .take(up_to_col)
        .map(|c| font.metrics(c, size_px).advance_width)
        .sum()
}

pub(crate) fn char_index_for_x(text: &str, rel_x: f32, font: &Font, size_px: f32) -> usize {
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

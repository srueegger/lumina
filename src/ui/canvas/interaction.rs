use crate::model::geometry::{Point, Rect};
use crate::ui::canvas::selection::HandlePosition;

#[derive(Debug, Clone, Copy)]
pub enum DragOperation {
    Move { start_x: f64, start_y: f64, orig_bounds: Rect },
    Resize { handle: HandlePosition, orig_bounds: Rect },
}

impl DragOperation {
    pub fn apply(&self, dx: f64, dy: f64) -> Rect {
        match self {
            DragOperation::Move { orig_bounds, .. } => Rect::new(
                orig_bounds.origin.x + dx,
                orig_bounds.origin.y + dy,
                orig_bounds.size.width,
                orig_bounds.size.height,
            ),
            DragOperation::Resize { handle, orig_bounds } => {
                resize_bounds(orig_bounds, *handle, dx, dy)
            }
        }
    }
}

fn resize_bounds(orig: &Rect, handle: HandlePosition, dx: f64, dy: f64) -> Rect {
    let mut x = orig.origin.x;
    let mut y = orig.origin.y;
    let mut w = orig.size.width;
    let mut h = orig.size.height;

    match handle {
        HandlePosition::TopLeft => {
            x += dx;
            y += dy;
            w -= dx;
            h -= dy;
        }
        HandlePosition::TopCenter => {
            y += dy;
            h -= dy;
        }
        HandlePosition::TopRight => {
            y += dy;
            w += dx;
            h -= dy;
        }
        HandlePosition::MiddleLeft => {
            x += dx;
            w -= dx;
        }
        HandlePosition::MiddleRight => {
            w += dx;
        }
        HandlePosition::BottomLeft => {
            x += dx;
            w -= dx;
            h += dy;
        }
        HandlePosition::BottomCenter => {
            h += dy;
        }
        HandlePosition::BottomRight => {
            w += dx;
            h += dy;
        }
    }

    // Enforce minimum size
    let min_size = 20.0;
    if w < min_size {
        w = min_size;
    }
    if h < min_size {
        h = min_size;
    }

    Rect::new(x, y, w, h)
}

pub fn widget_to_slide_coords(
    widget_x: f64,
    widget_y: f64,
    scale: f64,
    offset_x: f64,
    offset_y: f64,
) -> Point {
    Point::new(
        (widget_x - offset_x) / scale,
        (widget_y - offset_y) / scale,
    )
}

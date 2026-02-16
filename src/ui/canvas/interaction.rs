use crate::model::geometry::{Point, Rect};
use crate::ui::canvas::selection::HandlePosition;
use crate::ui::canvas::tool::Tool;

#[derive(Debug, Clone, Copy)]
pub enum DragOperation {
    Move { start_x: f64, start_y: f64, orig_bounds: Rect },
    Resize { handle: HandlePosition, orig_bounds: Rect },
    Create { tool: Tool, start: Point },
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
            DragOperation::Create { start, .. } => {
                normalize_rect(start.x, start.y, start.x + dx, start.y + dy)
            }
        }
    }
}

/// Create a normalized rect from two corners (handles negative width/height from dragging up/left)
pub fn normalize_rect(x1: f64, y1: f64, x2: f64, y2: f64) -> Rect {
    let x = x1.min(x2);
    let y = y1.min(y2);
    let w = (x2 - x1).abs().max(1.0);
    let h = (y2 - y1).abs().max(1.0);
    Rect::new(x, y, w, h)
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

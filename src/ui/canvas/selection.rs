use uuid::Uuid;

use crate::model::geometry::{Point, Rect};

const HANDLE_SIZE: f64 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandlePosition {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl HandlePosition {
    pub fn all() -> &'static [HandlePosition] {
        &[
            HandlePosition::TopLeft,
            HandlePosition::TopCenter,
            HandlePosition::TopRight,
            HandlePosition::MiddleLeft,
            HandlePosition::MiddleRight,
            HandlePosition::BottomLeft,
            HandlePosition::BottomCenter,
            HandlePosition::BottomRight,
        ]
    }

    pub fn rect_for_bounds(&self, bounds: &Rect) -> Rect {
        let half = HANDLE_SIZE / 2.0;
        let (cx, cy) = match self {
            HandlePosition::TopLeft => (bounds.origin.x, bounds.origin.y),
            HandlePosition::TopCenter => (bounds.center().x, bounds.origin.y),
            HandlePosition::TopRight => (bounds.right(), bounds.origin.y),
            HandlePosition::MiddleLeft => (bounds.origin.x, bounds.center().y),
            HandlePosition::MiddleRight => (bounds.right(), bounds.center().y),
            HandlePosition::BottomLeft => (bounds.origin.x, bounds.bottom()),
            HandlePosition::BottomCenter => (bounds.center().x, bounds.bottom()),
            HandlePosition::BottomRight => (bounds.right(), bounds.bottom()),
        };
        Rect::new(cx - half, cy - half, HANDLE_SIZE, HANDLE_SIZE)
    }
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub element_id: Option<Uuid>,
}

impl Selection {
    pub fn new() -> Self {
        Self { element_id: None }
    }

    pub fn select(&mut self, id: Uuid) {
        self.element_id = Some(id);
    }

    pub fn deselect(&mut self) {
        self.element_id = None;
    }

    pub fn is_selected(&self, id: Uuid) -> bool {
        self.element_id == Some(id)
    }

    pub fn has_selection(&self) -> bool {
        self.element_id.is_some()
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_selection_handles(cr: &cairo::Context, bounds: &Rect) {
    // Bounding box
    cr.set_source_rgba(0.2, 0.52, 0.89, 0.8);
    cr.set_line_width(1.5);
    cr.rectangle(
        bounds.origin.x,
        bounds.origin.y,
        bounds.size.width,
        bounds.size.height,
    );
    let _ = cr.stroke();

    // Handles
    for pos in HandlePosition::all() {
        let handle = pos.rect_for_bounds(bounds);

        // White fill
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.rectangle(
            handle.origin.x,
            handle.origin.y,
            handle.size.width,
            handle.size.height,
        );
        let _ = cr.fill_preserve();

        // Blue border
        cr.set_source_rgba(0.2, 0.52, 0.89, 0.8);
        cr.set_line_width(1.5);
        let _ = cr.stroke();
    }
}

pub fn hit_test_handle(point: Point, bounds: &Rect) -> Option<HandlePosition> {
    for pos in HandlePosition::all() {
        let handle = pos.rect_for_bounds(bounds);
        // Expand hit area slightly for easier grabbing
        let expanded = Rect::new(
            handle.origin.x - 4.0,
            handle.origin.y - 4.0,
            handle.size.width + 8.0,
            handle.size.height + 8.0,
        );
        if expanded.contains(point) {
            return Some(*pos);
        }
    }
    None
}

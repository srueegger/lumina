use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::geometry::Rect;
use super::style::{FillStyle, StrokeStyle};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeElement {
    pub id: Uuid,
    pub bounds: Rect,
    pub rotation: f64,
    pub shape_type: ShapeType,
    pub fill: Option<FillStyle>,
    pub stroke: Option<StrokeStyle>,
}

impl ShapeElement {
    pub fn new(bounds: Rect, shape_type: ShapeType) -> Self {
        let (fill, stroke) = match shape_type {
            ShapeType::Line => (None, Some(StrokeStyle::default())),
            _ => (
                Some(FillStyle::new(super::style::Color::from_hex("#4a86cf").unwrap())),
                Some(StrokeStyle::default()),
            ),
        };

        Self {
            id: Uuid::new_v4(),
            bounds,
            rotation: 0.0,
            shape_type,
            fill,
            stroke,
        }
    }
}

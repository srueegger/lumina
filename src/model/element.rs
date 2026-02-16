use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::geometry::Rect;
use super::image::ImageElement;
use super::shape::ShapeElement;
use super::text::TextElement;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlideElement {
    Text(TextElement),
    Image(ImageElement),
    Shape(ShapeElement),
}

impl SlideElement {
    pub fn id(&self) -> Uuid {
        match self {
            SlideElement::Text(e) => e.id,
            SlideElement::Image(e) => e.id,
            SlideElement::Shape(e) => e.id,
        }
    }

    pub fn bounds(&self) -> &Rect {
        match self {
            SlideElement::Text(e) => &e.bounds,
            SlideElement::Image(e) => &e.bounds,
            SlideElement::Shape(e) => &e.bounds,
        }
    }

    pub fn bounds_mut(&mut self) -> &mut Rect {
        match self {
            SlideElement::Text(e) => &mut e.bounds,
            SlideElement::Image(e) => &mut e.bounds,
            SlideElement::Shape(e) => &mut e.bounds,
        }
    }

    pub fn rotation(&self) -> f64 {
        match self {
            SlideElement::Text(e) => e.rotation,
            SlideElement::Image(e) => e.rotation,
            SlideElement::Shape(e) => e.rotation,
        }
    }
}

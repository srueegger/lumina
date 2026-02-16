use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::geometry::Rect;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageData {
    Embedded { data: Vec<u8>, mime: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ScaleMode {
    Fit,
    Fill,
    Stretch,
}

impl Default for ScaleMode {
    fn default() -> Self {
        Self::Fit
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageElement {
    pub id: Uuid,
    pub bounds: Rect,
    pub rotation: f64,
    pub image_data: ImageData,
    pub scale_mode: ScaleMode,
}

impl ImageElement {
    pub fn new(bounds: Rect, data: Vec<u8>, mime: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            bounds,
            rotation: 0.0,
            image_data: ImageData::Embedded { data, mime },
            scale_mode: ScaleMode::Fit,
        }
    }
}

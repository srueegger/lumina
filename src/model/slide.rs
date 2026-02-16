use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::element::SlideElement;
use super::style::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Background {
    Solid(Color),
}

impl Default for Background {
    fn default() -> Self {
        Background::Solid(Color::white())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    pub id: Uuid,
    pub elements: Vec<SlideElement>,
    pub background: Background,
    pub notes: String,
}

impl Slide {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            elements: Vec::new(),
            background: Background::default(),
            notes: String::new(),
        }
    }

    pub fn with_background(background: Background) -> Self {
        Self {
            id: Uuid::new_v4(),
            elements: Vec::new(),
            background,
            notes: String::new(),
        }
    }

    pub fn add_element(&mut self, element: SlideElement) {
        self.elements.push(element);
    }

    pub fn remove_element(&mut self, id: Uuid) -> Option<SlideElement> {
        if let Some(pos) = self.elements.iter().position(|e| e.id() == id) {
            Some(self.elements.remove(pos))
        } else {
            None
        }
    }

    pub fn find_element_at(
        &self,
        point: super::geometry::Point,
    ) -> Option<(usize, &SlideElement)> {
        // Iterate in reverse to find topmost element first
        for (i, element) in self.elements.iter().enumerate().rev() {
            if element.bounds().contains(point) {
                return Some((i, element));
            }
        }
        None
    }
}

impl Default for Slide {
    fn default() -> Self {
        Self::new()
    }
}

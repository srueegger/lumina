use serde::{Deserialize, Serialize};

use super::geometry::{Size, DEFAULT_SLIDE_SIZE};
use super::slide::Slide;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub author: String,
    pub created: String,
    pub modified: String,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            author: String::new(),
            created: String::new(),
            modified: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub title: String,
    pub slides: Vec<Slide>,
    pub slide_size: Size,
    pub metadata: DocumentMetadata,
}

impl Document {
    pub fn new() -> Self {
        Self {
            title: "Untitled Presentation".to_string(),
            slides: vec![Slide::new()],
            slide_size: DEFAULT_SLIDE_SIZE,
            metadata: DocumentMetadata::default(),
        }
    }

    pub fn add_slide(&mut self) -> usize {
        self.slides.push(Slide::new());
        self.slides.len() - 1
    }

    pub fn insert_slide(&mut self, index: usize) -> usize {
        let idx = index.min(self.slides.len());
        self.slides.insert(idx, Slide::new());
        idx
    }

    pub fn remove_slide(&mut self, index: usize) -> Option<Slide> {
        if self.slides.len() > 1 && index < self.slides.len() {
            Some(self.slides.remove(index))
        } else {
            None
        }
    }

    pub fn move_slide(&mut self, from: usize, to: usize) {
        if from < self.slides.len() && to < self.slides.len() && from != to {
            let slide = self.slides.remove(from);
            self.slides.insert(to, slide);
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

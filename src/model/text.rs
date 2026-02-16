use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::geometry::Rect;
use super::style::{FillStyle, FontStyle};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

impl Default for TextAlignment {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRun {
    pub text: String,
    pub font: FontStyle,
}

impl TextRun {
    pub fn new(text: impl Into<String>, font: FontStyle) -> Self {
        Self {
            text: text.into(),
            font,
        }
    }

    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font: FontStyle::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextParagraph {
    pub runs: Vec<TextRun>,
}

impl TextParagraph {
    pub fn new(runs: Vec<TextRun>) -> Self {
        Self { runs }
    }

    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            runs: vec![TextRun::plain(text)],
        }
    }

    pub fn full_text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextElement {
    pub id: Uuid,
    pub bounds: Rect,
    pub rotation: f64,
    pub paragraphs: Vec<TextParagraph>,
    pub alignment: TextAlignment,
    pub fill: Option<FillStyle>,
}

impl TextElement {
    pub fn new(bounds: Rect, text: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            bounds,
            rotation: 0.0,
            paragraphs: vec![TextParagraph::plain(text)],
            alignment: TextAlignment::Left,
            fill: None,
        }
    }
}

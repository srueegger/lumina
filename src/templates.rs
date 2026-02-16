use serde::{Deserialize, Serialize};

use crate::model::document::Document;
use crate::model::element::SlideElement;
use crate::model::geometry::Rect;
use crate::model::shape::{ShapeElement, ShapeType};
use crate::model::slide::{Background, Slide};
use crate::model::style::{Color, FillStyle, FontStyle, StrokeStyle};
use crate::model::text::{TextAlignment, TextElement, TextParagraph, TextRun};

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateDefinition {
    pub name: String,
    pub description: String,
    pub slides: Vec<TemplateSlide>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateSlide {
    pub background: String,
    pub elements: Vec<TemplateElement>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateElement {
    #[serde(rename = "type")]
    pub element_type: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    #[serde(default)]
    pub text: String,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub alignment: String,
    #[serde(default)]
    pub shape: String,
    #[serde(default)]
    pub fill_color: String,
    #[serde(default)]
    pub stroke_color: String,
    #[serde(default)]
    pub stroke_width: f64,
}

fn default_font_family() -> String {
    "Sans".to_string()
}
fn default_font_size() -> f64 {
    24.0
}
fn default_color() -> String {
    "#000000".to_string()
}

pub fn built_in_templates() -> Vec<TemplateDefinition> {
    let templates_json = [
        include_str!("../data/resources/templates/blank.json"),
        include_str!("../data/resources/templates/title-content.json"),
        include_str!("../data/resources/templates/photo-album.json"),
    ];

    templates_json
        .iter()
        .filter_map(|json| serde_json::from_str(json).ok())
        .collect()
}

pub fn create_document_from_template(template: &TemplateDefinition) -> Document {
    let mut doc = Document::new();
    doc.slides.clear();

    for tmpl_slide in &template.slides {
        let mut slide = Slide::new();

        if let Some(bg_color) = Color::from_hex(&tmpl_slide.background) {
            slide.background = Background::Solid(bg_color);
        }

        for tmpl_elem in &tmpl_slide.elements {
            let bounds = Rect::new(tmpl_elem.x, tmpl_elem.y, tmpl_elem.w, tmpl_elem.h);

            match tmpl_elem.element_type.as_str() {
                "text" => {
                    let font = FontStyle {
                        family: tmpl_elem.font_family.clone(),
                        size: tmpl_elem.font_size,
                        bold: tmpl_elem.bold,
                        italic: tmpl_elem.italic,
                        color: Color::from_hex(&tmpl_elem.color).unwrap_or_else(Color::black),
                    };
                    let mut text = TextElement::new(bounds, "");
                    text.paragraphs = vec![TextParagraph::new(vec![TextRun::new(
                        tmpl_elem.text.clone(),
                        font,
                    )])];
                    text.alignment = match tmpl_elem.alignment.as_str() {
                        "center" => TextAlignment::Center,
                        "right" => TextAlignment::Right,
                        _ => TextAlignment::Left,
                    };
                    slide.add_element(SlideElement::Text(text));
                }
                "shape" => {
                    let shape_type = match tmpl_elem.shape.as_str() {
                        "ellipse" | "circle" => ShapeType::Ellipse,
                        "line" => ShapeType::Line,
                        _ => ShapeType::Rectangle,
                    };
                    let mut shape = ShapeElement::new(bounds, shape_type);
                    if !tmpl_elem.fill_color.is_empty() {
                        shape.fill = Color::from_hex(&tmpl_elem.fill_color)
                            .map(FillStyle::new);
                    }
                    if !tmpl_elem.stroke_color.is_empty() {
                        shape.stroke = Color::from_hex(&tmpl_elem.stroke_color).map(|c| {
                            StrokeStyle::new(c, if tmpl_elem.stroke_width > 0.0 { tmpl_elem.stroke_width } else { 2.0 })
                        });
                    }
                    slide.add_element(SlideElement::Shape(shape));
                }
                _ => {}
            }
        }

        doc.slides.push(slide);
    }

    if doc.slides.is_empty() {
        doc.slides.push(Slide::new());
    }

    doc
}

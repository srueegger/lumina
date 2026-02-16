use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::io::{self, Read};
use std::path::Path;
use zip::ZipArchive;

use crate::model::document::Document;
use crate::model::element::SlideElement;
use crate::model::geometry::{Rect, Size};
use crate::model::image::ImageElement;
use crate::model::shape::{ShapeElement, ShapeType};
use crate::model::style::{Color, FillStyle, FontStyle, StrokeStyle};
use crate::model::text::{TextAlignment, TextElement, TextParagraph, TextRun};

use super::constants::*;

pub fn load_document(path: &Path) -> io::Result<Document> {
    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Read content.xml
    let content_xml = read_zip_entry(&mut archive, "content.xml")?;

    // Read styles.xml for page layout
    let styles_xml = read_zip_entry(&mut archive, "styles.xml").unwrap_or_default();

    // Parse slide size from styles
    let slide_size = parse_slide_size(&styles_xml);

    // Parse content
    let mut doc = parse_content(&content_xml, &mut archive)?;
    doc.slide_size = slide_size;

    Ok(doc)
}

fn read_zip_entry<R: Read + io::Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
) -> io::Result<String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))?;
    let mut content = String::new();
    entry.read_to_string(&mut content)?;
    Ok(content)
}

fn read_zip_entry_bytes<R: Read + io::Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
) -> io::Result<Vec<u8>> {
    let mut entry = archive
        .by_name(name)
        .map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))?;
    let mut data = Vec::new();
    entry.read_to_end(&mut data)?;
    Ok(data)
}

fn parse_slide_size(styles_xml: &str) -> Size {
    let mut reader = Reader::from_str(styles_xml);
    let mut buf = Vec::new();
    let mut width = 960.0_f64;
    let mut height = 540.0_f64;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if name == "page-layout-properties" {
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        if key == "page-width" {
                            if let Some(w) = parse_cm(&val) {
                                width = w;
                            }
                        } else if key == "page-height" {
                            if let Some(h) = parse_cm(&val) {
                                height = h;
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Size::new(width, height)
}

struct StyleInfo {
    fill_color: Option<Color>,
    stroke_color: Option<Color>,
    stroke_width: Option<f64>,
    has_fill: bool,
    has_stroke: bool,
    font_size: Option<f64>,
    font_color: Option<Color>,
    font_family: Option<String>,
    font_bold: bool,
    font_italic: bool,
    text_align: Option<TextAlignment>,
}

impl Default for StyleInfo {
    fn default() -> Self {
        Self {
            fill_color: None,
            stroke_color: None,
            stroke_width: None,
            has_fill: false,
            has_stroke: false,
            font_size: None,
            font_color: None,
            font_family: None,
            font_bold: false,
            font_italic: false,
            text_align: None,
        }
    }
}

fn parse_content<R: Read + io::Seek>(
    content_xml: &str,
    archive: &mut ZipArchive<R>,
) -> io::Result<Document> {
    let mut doc = Document::new();
    doc.slides.clear();

    let mut reader = Reader::from_str(content_xml);
    let mut buf = Vec::new();

    // First pass: collect styles
    let mut styles: HashMap<String, StyleInfo> = HashMap::new();
    let mut in_auto_styles = false;
    let mut current_style_name = String::new();
    let mut current_style = StyleInfo::default();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if name == "automatic-styles" {
                    in_auto_styles = true;
                } else if in_auto_styles && name == "style" {
                    current_style = StyleInfo::default();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
                        if key == "name" {
                            current_style_name = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if in_auto_styles {
                    if name == "graphic-properties" {
                        parse_graphic_props(e, &mut current_style);
                    } else if name == "text-properties" {
                        parse_text_props(e, &mut current_style);
                    } else if name == "paragraph-properties" {
                        parse_paragraph_props(e, &mut current_style);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if name == "automatic-styles" {
                    in_auto_styles = false;
                } else if in_auto_styles && name == "style" {
                    if !current_style_name.is_empty() {
                        styles.insert(
                            current_style_name.clone(),
                            std::mem::take(&mut current_style),
                        );
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    // Second pass: parse slides and elements
    let mut reader = Reader::from_str(content_xml);
    buf.clear();

    let mut in_presentation = false;
    let mut in_page = false;
    let mut in_text_box = false;
    let mut in_paragraph = false;
    let mut in_span = false;
    let mut current_elements: Vec<SlideElement> = Vec::new();
    let mut current_paragraphs: Vec<TextParagraph> = Vec::new();
    let mut current_runs: Vec<TextRun> = Vec::new();
    let mut current_run_text = String::new();
    let mut current_run_style = FontStyle::default();
    let mut current_text_align = TextAlignment::Left;
    let mut frame_bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
    let mut in_frame = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "presentation" => in_presentation = true,
                    "page" if in_presentation => {
                        in_page = true;
                        current_elements.clear();
                    }
                    "frame" if in_page => {
                        in_frame = true;
                        frame_bounds = parse_bounds(e);
                    }
                    "text-box" if in_frame => {
                        in_text_box = true;
                        current_paragraphs.clear();
                    }
                    "p" if in_text_box => {
                        in_paragraph = true;
                        current_runs.clear();
                        let ps_name = get_attr(e, "style-name");
                        current_text_align = styles
                            .get(&ps_name)
                            .and_then(|s| s.text_align)
                            .unwrap_or(TextAlignment::Left);
                    }
                    "span" if in_paragraph => {
                        in_span = true;
                        current_run_text.clear();
                        let ts_name = get_attr(e, "style-name");
                        if let Some(style) = styles.get(&ts_name) {
                            current_run_style = FontStyle {
                                family: style
                                    .font_family
                                    .clone()
                                    .unwrap_or_else(|| "Sans".to_string()),
                                size: style.font_size.unwrap_or(24.0),
                                bold: style.font_bold,
                                italic: style.font_italic,
                                color: style
                                    .font_color
                                    .clone()
                                    .unwrap_or_else(Color::black),
                            };
                        } else {
                            current_run_style = FontStyle::default();
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "rect" if in_page => {
                        let bounds = parse_bounds(e);
                        let style_name = get_attr(e, "style-name");
                        let shape = build_shape(ShapeType::Rectangle, bounds, &style_name, &styles);
                        current_elements.push(SlideElement::Shape(shape));
                    }
                    "ellipse" if in_page => {
                        let bounds = parse_bounds(e);
                        let style_name = get_attr(e, "style-name");
                        let shape = build_shape(ShapeType::Ellipse, bounds, &style_name, &styles);
                        current_elements.push(SlideElement::Shape(shape));
                    }
                    "line" if in_page => {
                        let bounds = parse_line_bounds(e);
                        let style_name = get_attr(e, "style-name");
                        let shape = build_shape(ShapeType::Line, bounds, &style_name, &styles);
                        current_elements.push(SlideElement::Shape(shape));
                    }
                    "image" if in_frame => {
                        let href = get_attr(e, "href");
                        if !href.is_empty() {
                            if let Ok(data) = read_zip_entry_bytes(archive, &href) {
                                let mime = guess_mime(&href);
                                let img = ImageElement::new(frame_bounds, data, mime.to_string());
                                current_elements.push(SlideElement::Image(img));
                                // Skip creating a text element for this frame
                                in_text_box = false;
                                in_frame = false;
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_span {
                    if let Ok(text) = e.unescape() {
                        current_run_text.push_str(&text);
                    }
                } else if in_paragraph && !in_span {
                    // Bare text in paragraph (no span)
                    if let Ok(text) = e.unescape() {
                        let text_str = text.to_string();
                        if !text_str.trim().is_empty() {
                            current_runs.push(TextRun::new(text_str, FontStyle::default()));
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "presentation" => in_presentation = false,
                    "page" if in_page => {
                        in_page = false;
                        let mut slide = crate::model::slide::Slide::new();
                        slide.elements = current_elements.drain(..).collect();
                        doc.slides.push(slide);
                    }
                    "frame" if in_frame => {
                        in_frame = false;
                    }
                    "text-box" if in_text_box => {
                        in_text_box = false;
                        let mut text = TextElement::new(frame_bounds, "");
                        text.paragraphs = current_paragraphs.drain(..).collect();
                        text.alignment = current_text_align;
                        if !text.paragraphs.is_empty() {
                            current_elements.push(SlideElement::Text(text));
                        }
                    }
                    "p" if in_paragraph => {
                        in_paragraph = false;
                        let para = TextParagraph::new(current_runs.drain(..).collect());
                        current_paragraphs.push(para);
                    }
                    "span" if in_span => {
                        in_span = false;
                        if !current_run_text.is_empty() {
                            current_runs.push(TextRun::new(
                                std::mem::take(&mut current_run_text),
                                current_run_style.clone(),
                            ));
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    // Ensure at least one slide
    if doc.slides.is_empty() {
        doc.slides.push(crate::model::slide::Slide::new());
    }

    Ok(doc)
}

fn parse_graphic_props(e: &quick_xml::events::BytesStart, style: &mut StyleInfo) {
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        match key.as_str() {
            "fill" => style.has_fill = val == "solid",
            "fill-color" => style.fill_color = parse_color(&val),
            "stroke" => style.has_stroke = val == "solid",
            "stroke-color" => style.stroke_color = parse_color(&val),
            "stroke-width" => style.stroke_width = parse_cm(&val),
            _ => {}
        }
    }
}

fn parse_text_props(e: &quick_xml::events::BytesStart, style: &mut StyleInfo) {
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        match key.as_str() {
            "font-size" => {
                if let Some(size) = val.strip_suffix("pt") {
                    style.font_size = size.parse().ok();
                }
            }
            "color" => style.font_color = parse_color(&val),
            "font-name" | "font-family" => style.font_family = Some(val),
            "font-weight" => style.font_bold = val == "bold",
            "font-style" => style.font_italic = val == "italic",
            _ => {}
        }
    }
}

fn parse_paragraph_props(e: &quick_xml::events::BytesStart, style: &mut StyleInfo) {
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        if key == "text-align" {
            style.text_align = Some(match val.as_str() {
                "center" => TextAlignment::Center,
                "end" | "right" => TextAlignment::Right,
                _ => TextAlignment::Left,
            });
        }
    }
}

fn parse_bounds(e: &quick_xml::events::BytesStart) -> Rect {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut w = 100.0;
    let mut h = 100.0;

    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        match key.as_str() {
            "x" => {
                if let Some(v) = parse_cm(&val) {
                    x = v;
                }
            }
            "y" => {
                if let Some(v) = parse_cm(&val) {
                    y = v;
                }
            }
            "width" => {
                if let Some(v) = parse_cm(&val) {
                    w = v;
                }
            }
            "height" => {
                if let Some(v) = parse_cm(&val) {
                    h = v;
                }
            }
            _ => {}
        }
    }

    Rect::new(x, y, w, h)
}

fn parse_line_bounds(e: &quick_xml::events::BytesStart) -> Rect {
    let mut x1 = 0.0;
    let mut y1 = 0.0;
    let mut x2 = 100.0;
    let mut y2 = 0.0;

    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        match key.as_str() {
            "x1" => {
                if let Some(v) = parse_cm(&val) {
                    x1 = v;
                }
            }
            "y1" => {
                if let Some(v) = parse_cm(&val) {
                    y1 = v;
                }
            }
            "x2" => {
                if let Some(v) = parse_cm(&val) {
                    x2 = v;
                }
            }
            "y2" => {
                if let Some(v) = parse_cm(&val) {
                    y2 = v;
                }
            }
            _ => {}
        }
    }

    let x = x1.min(x2);
    let y = y1.min(y2);
    Rect::new(x, y, (x2 - x1).abs(), (y2 - y1).abs())
}

fn get_attr(e: &quick_xml::events::BytesStart, local_name: &str) -> String {
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
        if key == local_name {
            return String::from_utf8_lossy(&attr.value).to_string();
        }
    }
    String::new()
}

fn parse_color(hex: &str) -> Option<Color> {
    Color::from_hex(hex)
}

fn build_shape(
    shape_type: ShapeType,
    bounds: Rect,
    style_name: &str,
    styles: &HashMap<String, StyleInfo>,
) -> ShapeElement {
    let mut shape = ShapeElement::new(bounds, shape_type);

    if let Some(style) = styles.get(style_name) {
        if style.has_fill {
            shape.fill = style.fill_color.as_ref().map(|c| FillStyle::new(c.clone()));
        } else {
            shape.fill = None;
        }
        if style.has_stroke {
            shape.stroke = Some(StrokeStyle::new(
                style.stroke_color.clone().unwrap_or_else(Color::black),
                style.stroke_width.unwrap_or(2.0),
            ));
        } else {
            shape.stroke = None;
        }
    }

    shape
}

fn guess_mime(path: &str) -> &str {
    if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else {
        "image/png"
    }
}

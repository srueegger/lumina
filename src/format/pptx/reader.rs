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

    // Parse presentation.xml for slide size and slide list
    let presentation_xml = read_zip_entry(&mut archive, "ppt/presentation.xml")?;
    let (slide_size, slide_refs) = parse_presentation(&presentation_xml);

    // Parse presentation.xml.rels for slide paths
    let pres_rels = read_zip_entry(&mut archive, "ppt/_rels/presentation.xml.rels")
        .unwrap_or_default();
    let rel_map = parse_rels(&pres_rels);

    let mut doc = Document::new();
    doc.slide_size = slide_size;
    doc.slides.clear();

    for slide_ref in &slide_refs {
        let slide_path = rel_map
            .get(slide_ref)
            .map(|p| format!("ppt/{}", p))
            .unwrap_or_default();

        if slide_path.is_empty() {
            doc.slides.push(crate::model::slide::Slide::new());
            continue;
        }

        // Parse slide relationships for images
        let slide_rels_path = slide_path
            .replace("slides/", "slides/_rels/")
            + ".rels";
        let slide_rels_xml = read_zip_entry(&mut archive, &slide_rels_path).unwrap_or_default();
        let slide_rel_map = parse_rels(&slide_rels_xml);

        let slide_xml = match read_zip_entry(&mut archive, &slide_path) {
            Ok(xml) => xml,
            Err(_) => {
                doc.slides.push(crate::model::slide::Slide::new());
                continue;
            }
        };

        let slide = parse_slide(&slide_xml, &slide_rel_map, &slide_path, &mut archive);
        doc.slides.push(slide);
    }

    if doc.slides.is_empty() {
        doc.slides.push(crate::model::slide::Slide::new());
    }

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

fn read_zip_bytes<R: Read + io::Seek>(
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

fn parse_presentation(xml: &str) -> (Size, Vec<String>) {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut width = 960.0_f64;
    let mut height = 540.0_f64;
    let mut slide_refs: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "sldSz" => {
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.local_name().as_ref())
                                .to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "cx" {
                                if let Ok(emu) = val.parse::<i64>() {
                                    width = emu_to_pt(emu);
                                }
                            } else if key == "cy" {
                                if let Ok(emu) = val.parse::<i64>() {
                                    height = emu_to_pt(emu);
                                }
                            }
                        }
                    }
                    "sldId" => {
                        for attr in e.attributes().flatten() {
                            let full_key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if full_key.ends_with(":id") || full_key == "r:id" {
                                slide_refs
                                    .push(String::from_utf8_lossy(&attr.value).to_string());
                            }
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

    (Size::new(width, height), slide_refs)
}

fn parse_rels(xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if name == "Relationship" {
                    let mut id = String::new();
                    let mut target = String::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.local_name().as_ref())
                            .to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "Id" => id = val,
                            "Target" => target = val,
                            _ => {}
                        }
                    }
                    if !id.is_empty() && !target.is_empty() {
                        map.insert(id, target);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    map
}

fn parse_slide<R: Read + io::Seek>(
    xml: &str,
    rels: &HashMap<String, String>,
    slide_path: &str,
    archive: &mut ZipArchive<R>,
) -> crate::model::slide::Slide {
    let mut slide = crate::model::slide::Slide::new();
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let mut in_sp = false; // shape
    let mut in_pic = false; // picture
    let mut in_tx_body = false;
    let mut in_p = false;
    let mut in_r = false;

    let mut sp_bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
    let mut _sp_is_text_box = false;
    let mut sp_shape_type: Option<ShapeType> = None;
    let mut sp_fill_color: Option<Color> = None;
    let mut sp_stroke_color: Option<Color> = None;
    let mut sp_stroke_width: Option<f64> = None;

    let mut text_paragraphs: Vec<TextParagraph> = Vec::new();
    let mut text_runs: Vec<TextRun> = Vec::new();
    let mut run_text = String::new();
    let mut run_font = FontStyle::default();
    let mut para_align = TextAlignment::Left;

    let mut pic_bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
    let mut pic_rel_id = String::new();

    let slide_dir = if let Some(idx) = slide_path.rfind('/') {
        &slide_path[..idx + 1]
    } else {
        ""
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "sp" => {
                        in_sp = true;
                        _sp_is_text_box = false;
                        sp_shape_type = None;
                        sp_fill_color = None;
                        sp_stroke_color = None;
                        sp_stroke_width = None;
                        text_paragraphs.clear();
                    }
                    "pic" => {
                        in_pic = true;
                        pic_rel_id.clear();
                    }
                    "txBody" if in_sp || in_pic => {
                        in_tx_body = true;
                        text_paragraphs.clear();
                    }
                    "p" if in_tx_body => {
                        in_p = true;
                        text_runs.clear();
                        para_align = TextAlignment::Left;
                    }
                    "r" if in_p => {
                        in_r = true;
                        run_text.clear();
                        run_font = FontStyle::default();
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "off" if in_sp || in_pic => {
                        let (x, y) = parse_emu_position(e);
                        if in_pic {
                            pic_bounds.origin.x = x;
                            pic_bounds.origin.y = y;
                        } else {
                            sp_bounds.origin.x = x;
                            sp_bounds.origin.y = y;
                        }
                    }
                    "ext" if in_sp || in_pic => {
                        let (w, h) = parse_emu_size(e);
                        if in_pic {
                            pic_bounds.size.width = w;
                            pic_bounds.size.height = h;
                        } else {
                            sp_bounds.size.width = w;
                            sp_bounds.size.height = h;
                        }
                    }
                    "prstGeom" if in_sp => {
                        for attr in e.attributes().flatten() {
                            let key =
                                String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
                            if key == "prst" {
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                sp_shape_type = match val.as_str() {
                                    "rect" | "roundRect" | "snip1Rect" | "snip2SameRect" => {
                                        Some(ShapeType::Rectangle)
                                    }
                                    "ellipse" | "circle" => Some(ShapeType::Ellipse),
                                    "line" | "straightConnector1" => Some(ShapeType::Line),
                                    _ => Some(ShapeType::Rectangle),
                                };
                            }
                        }
                    }
                    "srgbClr" if in_sp && !in_tx_body => {
                        for attr in e.attributes().flatten() {
                            let key =
                                String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
                            if key == "val" {
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                let color = Color::from_hex(&val);
                                // Simple heuristic: first color found is fill, second is stroke
                                if sp_fill_color.is_none() {
                                    sp_fill_color = color;
                                } else {
                                    sp_stroke_color = color;
                                }
                            }
                        }
                    }
                    "pPr" if in_p => {
                        for attr in e.attributes().flatten() {
                            let key =
                                String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
                            if key == "algn" {
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                para_align = match val.as_str() {
                                    "ctr" => TextAlignment::Center,
                                    "r" => TextAlignment::Right,
                                    _ => TextAlignment::Left,
                                };
                            }
                        }
                    }
                    "rPr" if in_r => {
                        parse_run_properties(e, &mut run_font);
                    }
                    "latin" | "cs" if in_r => {
                        for attr in e.attributes().flatten() {
                            let key =
                                String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
                            if key == "typeface" {
                                run_font.family =
                                    String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                    "blipFill" | "blip" if in_pic => {
                        for attr in e.attributes().flatten() {
                            let full_key =
                                String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if full_key.ends_with(":embed") || full_key == "r:embed" {
                                pic_rel_id =
                                    String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_r {
                    if let Ok(text) = e.unescape() {
                        run_text.push_str(&text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "sp" => {
                        in_sp = false;
                        if !text_paragraphs.is_empty() {
                            let has_text = text_paragraphs
                                .iter()
                                .any(|p| !p.full_text().trim().is_empty());
                            if has_text {
                                let mut text_elem = TextElement::new(sp_bounds, "");
                                text_elem.paragraphs = text_paragraphs.drain(..).collect();
                                text_elem.alignment = para_align;
                                slide.add_element(SlideElement::Text(text_elem));
                            } else if let Some(shape_type) = sp_shape_type {
                                let mut shape = ShapeElement::new(sp_bounds, shape_type);
                                shape.fill = sp_fill_color.as_ref().map(|c| FillStyle::new(c.clone()));
                                if let Some(sc) = &sp_stroke_color {
                                    shape.stroke = Some(StrokeStyle::new(
                                        sc.clone(),
                                        sp_stroke_width.unwrap_or(2.0),
                                    ));
                                }
                                slide.add_element(SlideElement::Shape(shape));
                            }
                        } else if let Some(shape_type) = sp_shape_type {
                            let mut shape = ShapeElement::new(sp_bounds, shape_type);
                            shape.fill = sp_fill_color.as_ref().map(|c| FillStyle::new(c.clone()));
                            if let Some(sc) = &sp_stroke_color {
                                shape.stroke = Some(StrokeStyle::new(
                                    sc.clone(),
                                    sp_stroke_width.unwrap_or(2.0),
                                ));
                            }
                            slide.add_element(SlideElement::Shape(shape));
                        }
                    }
                    "pic" => {
                        in_pic = false;
                        if !pic_rel_id.is_empty() {
                            if let Some(rel_target) = rels.get(&pic_rel_id) {
                                let img_path = resolve_path(slide_dir, rel_target);
                                if let Ok(data) = read_zip_bytes(archive, &img_path) {
                                    let mime = guess_mime(&img_path);
                                    let img =
                                        ImageElement::new(pic_bounds, data, mime.to_string());
                                    slide.add_element(SlideElement::Image(img));
                                }
                            }
                        }
                    }
                    "txBody" => in_tx_body = false,
                    "p" if in_p => {
                        in_p = false;
                        let para = TextParagraph::new(text_runs.drain(..).collect());
                        text_paragraphs.push(para);
                    }
                    "r" if in_r => {
                        in_r = false;
                        if !run_text.is_empty() {
                            text_runs.push(TextRun::new(
                                std::mem::take(&mut run_text),
                                run_font.clone(),
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

    slide
}

fn parse_emu_position(e: &quick_xml::events::BytesStart) -> (f64, f64) {
    let mut x = 0.0;
    let mut y = 0.0;
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        if key == "x" {
            if let Ok(emu) = val.parse::<i64>() {
                x = emu_to_pt(emu);
            }
        } else if key == "y" {
            if let Ok(emu) = val.parse::<i64>() {
                y = emu_to_pt(emu);
            }
        }
    }
    (x, y)
}

fn parse_emu_size(e: &quick_xml::events::BytesStart) -> (f64, f64) {
    let mut w = 100.0;
    let mut h = 100.0;
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        if key == "cx" {
            if let Ok(emu) = val.parse::<i64>() {
                w = emu_to_pt(emu);
            }
        } else if key == "cy" {
            if let Ok(emu) = val.parse::<i64>() {
                h = emu_to_pt(emu);
            }
        }
    }
    (w, h)
}

fn parse_run_properties(e: &quick_xml::events::BytesStart, font: &mut FontStyle) {
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.local_name().as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value).to_string();
        match key.as_str() {
            "sz" => {
                // Font size in hundredths of a point
                if let Ok(sz) = val.parse::<f64>() {
                    font.size = half_pt_to_pt(sz);
                }
            }
            "b" => font.bold = val == "1" || val == "true",
            "i" => font.italic = val == "1" || val == "true",
            _ => {}
        }
    }
}

fn resolve_path(base_dir: &str, relative: &str) -> String {
    if relative.starts_with("../") {
        // Go up one directory
        let parent = if let Some(idx) = base_dir.trim_end_matches('/').rfind('/') {
            &base_dir[..idx + 1]
        } else {
            ""
        };
        format!("{}{}", parent, &relative[3..])
    } else {
        format!("{}{}", base_dir, relative)
    }
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
    } else if path.ends_with(".emf") || path.ends_with(".wmf") {
        "image/png" // Fallback - these won't render properly
    } else {
        "image/png"
    }
}

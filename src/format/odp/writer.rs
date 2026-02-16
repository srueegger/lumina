use std::io::{self, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::model::document::Document;
use crate::model::element::SlideElement;
use crate::model::shape::ShapeType;
use crate::model::style::Color;
use crate::model::text::TextAlignment;

use super::constants::*;

pub fn save_document(doc: &Document, path: &Path) -> io::Result<()> {
    let file = std::fs::File::create(path)?;
    let mut zip = ZipWriter::new(file);

    // mimetype must be first entry, uncompressed
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    zip.start_file("mimetype", options)?;
    zip.write_all(ODP_MIMETYPE.as_bytes())?;

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // META-INF/manifest.xml
    let manifest = build_manifest(doc);
    zip.start_file("META-INF/manifest.xml", options)?;
    zip.write_all(manifest.as_bytes())?;

    // meta.xml
    let meta = build_meta(doc);
    zip.start_file("meta.xml", options)?;
    zip.write_all(meta.as_bytes())?;

    // styles.xml
    let styles = build_styles(doc);
    zip.start_file("styles.xml", options)?;
    zip.write_all(styles.as_bytes())?;

    // content.xml
    let (content, images) = build_content(doc);
    zip.start_file("content.xml", options)?;
    zip.write_all(content.as_bytes())?;

    // Write embedded images
    for (img_path, img_data) in &images {
        zip.start_file(img_path, options)?;
        zip.write_all(img_data)?;
    }

    zip.finish()?;
    Ok(())
}

fn build_manifest(doc: &Document) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(&format!(
        "<manifest:manifest xmlns:manifest=\"{}\" manifest:version=\"1.2\">\n",
        NS_MANIFEST
    ));
    xml.push_str(&format!(
        "  <manifest:file-entry manifest:full-path=\"/\" manifest:version=\"1.2\" manifest:media-type=\"{}\"/>\n",
        ODP_MIMETYPE
    ));
    xml.push_str("  <manifest:file-entry manifest:full-path=\"content.xml\" manifest:media-type=\"text/xml\"/>\n");
    xml.push_str("  <manifest:file-entry manifest:full-path=\"styles.xml\" manifest:media-type=\"text/xml\"/>\n");
    xml.push_str("  <manifest:file-entry manifest:full-path=\"meta.xml\" manifest:media-type=\"text/xml\"/>\n");

    // Add image entries
    let mut img_idx = 0;
    for slide in &doc.slides {
        for element in &slide.elements {
            if let SlideElement::Image(img) = element {
                let ext = mime_to_ext(&img.image_data);
                let mime = mime_from_data(&img.image_data);
                xml.push_str(&format!(
                    "  <manifest:file-entry manifest:full-path=\"Pictures/image{}.{}\" manifest:media-type=\"{}\"/>\n",
                    img_idx, ext, mime
                ));
                img_idx += 1;
            }
        }
    }

    xml.push_str("</manifest:manifest>\n");
    xml
}

fn build_meta(_doc: &Document) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(&format!(
        "<office:document-meta xmlns:office=\"{}\" xmlns:meta=\"{}\" xmlns:dc=\"{}\" office:version=\"1.2\">\n",
        NS_OFFICE, NS_META, NS_DC
    ));
    xml.push_str("  <office:meta>\n");
    xml.push_str("    <meta:generator>Lumina</meta:generator>\n");
    xml.push_str("  </office:meta>\n");
    xml.push_str("</office:document-meta>\n");
    xml
}

fn build_styles(doc: &Document) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(&format!(
        "<office:document-styles xmlns:office=\"{}\" xmlns:style=\"{}\" xmlns:fo=\"{}\" xmlns:draw=\"{}\" xmlns:presentation=\"{}\" xmlns:svg=\"{}\" office:version=\"1.2\">\n",
        NS_OFFICE, NS_STYLE, NS_FO, NS_DRAW, NS_PRESENTATION, NS_SVG
    ));

    // Page layout
    xml.push_str("  <office:automatic-styles>\n");
    xml.push_str("    <style:page-layout style:name=\"PM1\">\n");
    xml.push_str(&format!(
        "      <style:page-layout-properties fo:page-width=\"{}\" fo:page-height=\"{}\" style:print-orientation=\"landscape\" fo:margin-top=\"0cm\" fo:margin-bottom=\"0cm\" fo:margin-left=\"0cm\" fo:margin-right=\"0cm\"/>\n",
        format_cm(doc.slide_size.width),
        format_cm(doc.slide_size.height)
    ));
    xml.push_str("    </style:page-layout>\n");
    xml.push_str("  </office:automatic-styles>\n");

    // Master pages
    xml.push_str("  <office:master-styles>\n");
    xml.push_str("    <style:master-page style:name=\"Default\" style:page-layout-name=\"PM1\" draw:style-name=\"dp1\"/>\n");
    xml.push_str("  </office:master-styles>\n");

    xml.push_str("</office:document-styles>\n");
    xml
}

fn build_content(doc: &Document) -> (String, Vec<(String, Vec<u8>)>) {
    let mut xml = String::new();
    let mut images: Vec<(String, Vec<u8>)> = Vec::new();
    let mut img_idx = 0;
    let mut style_idx = 0;

    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(&format!(
        "<office:document-content xmlns:office=\"{}\" xmlns:style=\"{}\" xmlns:text=\"{}\" xmlns:draw=\"{}\" xmlns:presentation=\"{}\" xmlns:fo=\"{}\" xmlns:svg=\"{}\" xmlns:xlink=\"{}\" office:version=\"1.2\">\n",
        NS_OFFICE, NS_STYLE, NS_TEXT, NS_DRAW, NS_PRESENTATION, NS_FO, NS_SVG, NS_XLINK
    ));

    // Collect styles needed
    let mut auto_styles = String::new();
    let mut body = String::new();

    // Drawing page style
    auto_styles.push_str("    <style:style style:name=\"dp1\" style:family=\"drawing-page\">\n");
    auto_styles.push_str("      <style:drawing-page-properties draw:fill=\"solid\" draw:fill-color=\"#ffffff\"/>\n");
    auto_styles.push_str("    </style:style>\n");

    body.push_str("  <office:body>\n");
    body.push_str("    <office:presentation>\n");

    for (slide_idx, slide) in doc.slides.iter().enumerate() {
        body.push_str(&format!(
            "      <draw:page draw:name=\"Slide{}\" draw:style-name=\"dp1\" draw:master-page-name=\"Default\" presentation:presentation-page-layout-name=\"AL1T0\">\n",
            slide_idx + 1
        ));

        for element in &slide.elements {
            match element {
                SlideElement::Text(text) => {
                    let style_name = format!("gr{}", style_idx);
                    style_idx += 1;

                    // Style for text frame
                    auto_styles.push_str(&format!(
                        "    <style:style style:name=\"{}\" style:family=\"graphic\" style:parent-style-name=\"standard\">\n",
                        style_name
                    ));
                    auto_styles.push_str("      <style:graphic-properties draw:stroke=\"none\" draw:fill=\"none\" draw:textarea-vertical-align=\"top\" fo:padding=\"0cm\"/>\n");
                    auto_styles.push_str("    </style:style>\n");

                    // Text paragraph styles
                    let mut para_styles = Vec::new();
                    for (pi, para) in text.paragraphs.iter().enumerate() {
                        let ps_name = format!("P{}_{}", slide_idx, pi);
                        let align = match text.alignment {
                            TextAlignment::Left => "start",
                            TextAlignment::Center => "center",
                            TextAlignment::Right => "end",
                        };
                        auto_styles.push_str(&format!(
                            "    <style:style style:name=\"{}\" style:family=\"paragraph\">\n",
                            ps_name
                        ));
                        auto_styles.push_str(&format!(
                            "      <style:paragraph-properties fo:text-align=\"{}\"/>\n",
                            align
                        ));
                        auto_styles.push_str("    </style:style>\n");

                        // Text run styles
                        let mut run_styles = Vec::new();
                        for (ri, run) in para.runs.iter().enumerate() {
                            let ts_name = format!("T{}_{}_{}", slide_idx, pi, ri);
                            auto_styles.push_str(&format!(
                                "    <style:style style:name=\"{}\" style:family=\"text\">\n",
                                ts_name
                            ));
                            auto_styles.push_str(&format!(
                                "      <style:text-properties fo:font-size=\"{}pt\" fo:color=\"{}\" style:font-name=\"{}\"{}{}/>",
                                run.font.size,
                                color_to_hex(&run.font.color),
                                xml_escape(&run.font.family),
                                if run.font.bold { " fo:font-weight=\"bold\"" } else { "" },
                                if run.font.italic { " fo:font-style=\"italic\"" } else { "" },
                            ));
                            auto_styles.push('\n');
                            auto_styles.push_str("    </style:style>\n");
                            run_styles.push(ts_name);
                        }
                        para_styles.push((ps_name, run_styles));
                    }

                    body.push_str(&format!(
                        "        <draw:frame draw:style-name=\"{}\" svg:x=\"{}\" svg:y=\"{}\" svg:width=\"{}\" svg:height=\"{}\">\n",
                        style_name,
                        format_cm(text.bounds.origin.x),
                        format_cm(text.bounds.origin.y),
                        format_cm(text.bounds.size.width),
                        format_cm(text.bounds.size.height)
                    ));
                    body.push_str("          <draw:text-box>\n");

                    for (pi, para) in text.paragraphs.iter().enumerate() {
                        let (ref ps_name, ref run_styles) = para_styles[pi];
                        body.push_str(&format!(
                            "            <text:p text:style-name=\"{}\">\n",
                            ps_name
                        ));
                        for (ri, run) in para.runs.iter().enumerate() {
                            body.push_str(&format!(
                                "              <text:span text:style-name=\"{}\">{}</text:span>\n",
                                run_styles[ri],
                                xml_escape(&run.text)
                            ));
                        }
                        body.push_str("            </text:p>\n");
                    }

                    body.push_str("          </draw:text-box>\n");
                    body.push_str("        </draw:frame>\n");
                }
                SlideElement::Shape(shape) => {
                    let style_name = format!("gr{}", style_idx);
                    style_idx += 1;

                    // Style
                    auto_styles.push_str(&format!(
                        "    <style:style style:name=\"{}\" style:family=\"graphic\">\n",
                        style_name
                    ));
                    auto_styles.push_str("      <style:graphic-properties");
                    if let Some(fill) = &shape.fill {
                        auto_styles.push_str(&format!(
                            " draw:fill=\"solid\" draw:fill-color=\"{}\"",
                            color_to_hex(&fill.color)
                        ));
                    } else {
                        auto_styles.push_str(" draw:fill=\"none\"");
                    }
                    if let Some(stroke) = &shape.stroke {
                        auto_styles.push_str(&format!(
                            " draw:stroke=\"solid\" svg:stroke-color=\"{}\" svg:stroke-width=\"{}\"",
                            color_to_hex(&stroke.color),
                            format_cm(stroke.width)
                        ));
                    } else {
                        auto_styles.push_str(" draw:stroke=\"none\"");
                    }
                    auto_styles.push_str("/>\n");
                    auto_styles.push_str("    </style:style>\n");

                    match shape.shape_type {
                        ShapeType::Rectangle => {
                            body.push_str(&format!(
                                "        <draw:rect draw:style-name=\"{}\" svg:x=\"{}\" svg:y=\"{}\" svg:width=\"{}\" svg:height=\"{}\"/>\n",
                                style_name,
                                format_cm(shape.bounds.origin.x),
                                format_cm(shape.bounds.origin.y),
                                format_cm(shape.bounds.size.width),
                                format_cm(shape.bounds.size.height)
                            ));
                        }
                        ShapeType::Ellipse => {
                            body.push_str(&format!(
                                "        <draw:ellipse draw:style-name=\"{}\" svg:x=\"{}\" svg:y=\"{}\" svg:width=\"{}\" svg:height=\"{}\"/>\n",
                                style_name,
                                format_cm(shape.bounds.origin.x),
                                format_cm(shape.bounds.origin.y),
                                format_cm(shape.bounds.size.width),
                                format_cm(shape.bounds.size.height)
                            ));
                        }
                        ShapeType::Line => {
                            let x1 = shape.bounds.origin.x;
                            let y1 = shape.bounds.origin.y;
                            let x2 = x1 + shape.bounds.size.width;
                            let y2 = y1 + shape.bounds.size.height;
                            body.push_str(&format!(
                                "        <draw:line draw:style-name=\"{}\" svg:x1=\"{}\" svg:y1=\"{}\" svg:x2=\"{}\" svg:y2=\"{}\"/>\n",
                                style_name,
                                format_cm(x1),
                                format_cm(y1),
                                format_cm(x2),
                                format_cm(y2)
                            ));
                        }
                    }
                }
                SlideElement::Image(img) => {
                    let ext = mime_to_ext(&img.image_data);
                    let img_path = format!("Pictures/image{}.{}", img_idx, ext);

                    let style_name = format!("gr{}", style_idx);
                    style_idx += 1;

                    auto_styles.push_str(&format!(
                        "    <style:style style:name=\"{}\" style:family=\"graphic\">\n",
                        style_name
                    ));
                    auto_styles.push_str("      <style:graphic-properties draw:stroke=\"none\" draw:fill=\"none\"/>\n");
                    auto_styles.push_str("    </style:style>\n");

                    body.push_str(&format!(
                        "        <draw:frame draw:style-name=\"{}\" svg:x=\"{}\" svg:y=\"{}\" svg:width=\"{}\" svg:height=\"{}\">\n",
                        style_name,
                        format_cm(img.bounds.origin.x),
                        format_cm(img.bounds.origin.y),
                        format_cm(img.bounds.size.width),
                        format_cm(img.bounds.size.height)
                    ));
                    body.push_str(&format!(
                        "          <draw:image xlink:href=\"{}\" xlink:type=\"simple\" xlink:show=\"embed\" xlink:actuate=\"onLoad\"/>\n",
                        img_path
                    ));
                    body.push_str("        </draw:frame>\n");

                    let crate::model::image::ImageData::Embedded { data, .. } = &img.image_data;
                    images.push((img_path, data.clone()));
                    img_idx += 1;
                }
            }
        }

        body.push_str("      </draw:page>\n");
    }

    body.push_str("    </office:presentation>\n");
    body.push_str("  </office:body>\n");

    xml.push_str("  <office:automatic-styles>\n");
    xml.push_str(&auto_styles);
    xml.push_str("  </office:automatic-styles>\n");
    xml.push_str(&body);
    xml.push_str("</office:document-content>\n");

    (xml, images)
}

fn color_to_hex(color: &Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn mime_to_ext(img_data: &crate::model::image::ImageData) -> &'static str {
    match img_data {
        crate::model::image::ImageData::Embedded { mime, .. } => match mime.as_str() {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/svg+xml" => "svg",
            "image/webp" => "webp",
            _ => "png",
        },
    }
}

fn mime_from_data(img_data: &crate::model::image::ImageData) -> &str {
    match img_data {
        crate::model::image::ImageData::Embedded { mime, .. } => mime.as_str(),
    }
}

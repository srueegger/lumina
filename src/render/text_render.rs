use cairo::Context;
use pango::FontDescription;

use crate::model::style::FontStyle;
use crate::model::text::{TextAlignment, TextElement};

fn to_pango_alignment(alignment: TextAlignment) -> pango::Alignment {
    match alignment {
        TextAlignment::Left => pango::Alignment::Left,
        TextAlignment::Center => pango::Alignment::Center,
        TextAlignment::Right => pango::Alignment::Right,
    }
}

pub fn render_text(cr: &Context, text: &TextElement) {
    let bounds = &text.bounds;

    cr.save().expect("cairo save");
    cr.translate(bounds.origin.x, bounds.origin.y);

    if text.rotation != 0.0 {
        cr.translate(bounds.size.width / 2.0, bounds.size.height / 2.0);
        cr.rotate(text.rotation.to_radians());
        cr.translate(-bounds.size.width / 2.0, -bounds.size.height / 2.0);
    }

    if let Some(fill) = &text.fill {
        cr.set_source_rgba(fill.color.r, fill.color.g, fill.color.b, fill.color.a);
        cr.rectangle(0.0, 0.0, bounds.size.width, bounds.size.height);
        let _ = cr.fill();
    }

    let layout = pangocairo::functions::create_layout(cr);
    layout.set_width((bounds.size.width * pango::SCALE as f64) as i32);
    layout.set_alignment(to_pango_alignment(text.alignment));
    layout.set_wrap(pango::WrapMode::WordChar);

    let mut y_offset = 0.0;
    for paragraph in &text.paragraphs {
        for run in &paragraph.runs {
            let font_desc = build_font_description(&run.font);
            layout.set_font_description(Some(&font_desc));
            layout.set_text(&run.text);

            cr.move_to(0.0, y_offset);
            cr.set_source_rgba(
                run.font.color.r,
                run.font.color.g,
                run.font.color.b,
                run.font.color.a,
            );
            pangocairo::functions::show_layout(cr, &layout);

            let (_, logical_rect) = layout.pixel_extents();
            y_offset += logical_rect.height() as f64;
        }
    }

    cr.restore().expect("cairo restore");
}

fn build_font_description(font: &FontStyle) -> FontDescription {
    let mut desc = FontDescription::new();
    desc.set_family(&font.family);
    desc.set_size((font.size * pango::SCALE as f64) as i32);
    if font.bold {
        desc.set_weight(pango::Weight::Bold);
    }
    if font.italic {
        desc.set_style(pango::Style::Italic);
    }
    desc
}

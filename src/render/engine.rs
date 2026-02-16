use cairo::Context;

use crate::model::element::SlideElement;
use crate::model::geometry::Size;
use crate::model::slide::{Background, Slide};

use super::image_render;
use super::shape_render;
use super::text_render;

pub fn render_slide(cr: &Context, slide: &Slide, size: &Size) {
    render_background(cr, &slide.background, size);

    for element in &slide.elements {
        match element {
            SlideElement::Text(text) => text_render::render_text(cr, text),
            SlideElement::Image(img) => image_render::render_image(cr, img),
            SlideElement::Shape(shape) => shape_render::render_shape(cr, shape),
        }
    }
}

fn render_background(cr: &Context, bg: &Background, size: &Size) {
    match bg {
        Background::Solid(color) => {
            cr.set_source_rgba(color.r, color.g, color.b, color.a);
            cr.rectangle(0.0, 0.0, size.width, size.height);
            let _ = cr.fill();
        }
    }
}

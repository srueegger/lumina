use cairo::Context;
use std::f64::consts::PI;

use crate::model::shape::{ShapeElement, ShapeType};

pub fn render_shape(cr: &Context, shape: &ShapeElement) {
    let bounds = &shape.bounds;

    cr.save().expect("cairo save");
    cr.translate(bounds.origin.x, bounds.origin.y);

    if shape.rotation != 0.0 {
        cr.translate(bounds.size.width / 2.0, bounds.size.height / 2.0);
        cr.rotate(shape.rotation.to_radians());
        cr.translate(-bounds.size.width / 2.0, -bounds.size.height / 2.0);
    }

    match shape.shape_type {
        ShapeType::Rectangle => {
            cr.rectangle(0.0, 0.0, bounds.size.width, bounds.size.height);
        }
        ShapeType::Ellipse => {
            let cx = bounds.size.width / 2.0;
            let cy = bounds.size.height / 2.0;
            cr.save().expect("cairo save");
            cr.translate(cx, cy);
            cr.scale(bounds.size.width / 2.0, bounds.size.height / 2.0);
            cr.arc(0.0, 0.0, 1.0, 0.0, 2.0 * PI);
            cr.restore().expect("cairo restore");
        }
        ShapeType::Line => {
            cr.move_to(0.0, 0.0);
            cr.line_to(bounds.size.width, bounds.size.height);
        }
    }

    if shape.shape_type != ShapeType::Line {
        if let Some(fill) = &shape.fill {
            cr.set_source_rgba(fill.color.r, fill.color.g, fill.color.b, fill.color.a);
            let _ = cr.fill_preserve();
        }
    }

    if let Some(stroke) = &shape.stroke {
        cr.set_source_rgba(
            stroke.color.r,
            stroke.color.g,
            stroke.color.b,
            stroke.color.a,
        );
        cr.set_line_width(stroke.width);
        let _ = cr.stroke();
    } else {
        cr.new_path();
    }

    cr.restore().expect("cairo restore");
}

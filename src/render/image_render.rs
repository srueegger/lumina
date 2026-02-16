use cairo::Context;
use gdk_pixbuf::prelude::*;

use crate::model::image::{ImageData, ImageElement, ScaleMode};

pub fn render_image(cr: &Context, image: &ImageElement) {
    let bounds = &image.bounds;

    cr.save().expect("cairo save");
    cr.translate(bounds.origin.x, bounds.origin.y);

    if image.rotation != 0.0 {
        cr.translate(bounds.size.width / 2.0, bounds.size.height / 2.0);
        cr.rotate(image.rotation.to_radians());
        cr.translate(-bounds.size.width / 2.0, -bounds.size.height / 2.0);
    }

    let ImageData::Embedded { ref data, ref mime } = image.image_data;
    let _ = mime;

    let pixbuf_loader = gdk_pixbuf::PixbufLoader::new();
    if pixbuf_loader.write(data).is_ok() {
        let _ = pixbuf_loader.close();
        if let Some(pixbuf) = pixbuf_loader.pixbuf() {
            let img_width = pixbuf.width() as f64;
            let img_height = pixbuf.height() as f64;

            let (scale_x, scale_y, offset_x, offset_y) = match image.scale_mode {
                ScaleMode::Stretch => {
                    let sx = bounds.size.width / img_width;
                    let sy = bounds.size.height / img_height;
                    (sx, sy, 0.0, 0.0)
                }
                ScaleMode::Fit => {
                    let scale =
                        (bounds.size.width / img_width).min(bounds.size.height / img_height);
                    let offset_x = (bounds.size.width - img_width * scale) / 2.0;
                    let offset_y = (bounds.size.height - img_height * scale) / 2.0;
                    (scale, scale, offset_x, offset_y)
                }
                ScaleMode::Fill => {
                    let scale =
                        (bounds.size.width / img_width).max(bounds.size.height / img_height);
                    let offset_x = (bounds.size.width - img_width * scale) / 2.0;
                    let offset_y = (bounds.size.height - img_height * scale) / 2.0;
                    (scale, scale, offset_x, offset_y)
                }
            };

            // Clip to bounds
            cr.rectangle(0.0, 0.0, bounds.size.width, bounds.size.height);
            cr.clip();

            cr.translate(offset_x, offset_y);
            cr.scale(scale_x, scale_y);

            // Convert Pixbuf to Cairo ImageSurface
            if let Some(surface) = pixbuf_to_surface(&pixbuf) {
                cr.set_source_surface(&surface, 0.0, 0.0)
                    .expect("set source surface");
                let _ = cr.paint();
            }
        }
    } else {
        let _ = pixbuf_loader.close();
    }

    cr.restore().expect("cairo restore");
}

fn pixbuf_to_surface(pixbuf: &gdk_pixbuf::Pixbuf) -> Option<cairo::ImageSurface> {
    let width = pixbuf.width();
    let height = pixbuf.height();
    let has_alpha = pixbuf.has_alpha();
    let src_stride = pixbuf.rowstride() as usize;
    let pixels = unsafe { pixbuf.pixels() };

    let format = if has_alpha {
        cairo::Format::ARgb32
    } else {
        cairo::Format::Rgb24
    };

    let mut surface = cairo::ImageSurface::create(format, width, height).ok()?;
    let dst_stride = surface.stride() as usize;

    {
        let mut data = surface.data().ok()?;
        for y in 0..height as usize {
            let src_row = &pixels[y * src_stride..];
            let dst_row = &mut data[y * dst_stride..];

            for x in 0..width as usize {
                let (r, g, b, a) = if has_alpha {
                    let offset = x * 4;
                    (
                        src_row[offset] as u32,
                        src_row[offset + 1] as u32,
                        src_row[offset + 2] as u32,
                        src_row[offset + 3] as u32,
                    )
                } else {
                    let offset = x * 3;
                    (
                        src_row[offset] as u32,
                        src_row[offset + 1] as u32,
                        src_row[offset + 2] as u32,
                        255u32,
                    )
                };

                // Cairo expects premultiplied ARGB in native byte order
                let pr = r * a / 255;
                let pg = g * a / 255;
                let pb = b * a / 255;

                let dst_offset = x * 4;
                // ARGB32 in little-endian: BGRA byte order
                dst_row[dst_offset] = pb as u8;
                dst_row[dst_offset + 1] = pg as u8;
                dst_row[dst_offset + 2] = pr as u8;
                dst_row[dst_offset + 3] = a as u8;
            }
        }
    }

    Some(surface)
}

use std::io;
use std::path::Path;

use crate::model::document::Document;

use super::engine;

pub fn export_pdf(doc: &Document, path: &Path) -> io::Result<()> {
    let slide_size = &doc.slide_size;
    let pdf_width = slide_size.width;
    let pdf_height = slide_size.height;

    let surface = cairo::PdfSurface::new(pdf_width, pdf_height, path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Cairo PDF error: {}", e)))?;

    let cr = cairo::Context::new(&surface)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Cairo context error: {}", e)))?;

    for (i, slide) in doc.slides.iter().enumerate() {
        if i > 0 {
            cr.show_page()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        }

        engine::render_slide(&cr, slide, slide_size);
    }

    cr.show_page()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    surface.finish();
    Ok(())
}

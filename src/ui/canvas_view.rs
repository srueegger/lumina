use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::model::document::Document;
use crate::model::geometry::Size;
use crate::render::engine;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct CanvasView {
        pub drawing_area: gtk::DrawingArea,
        pub document: RefCell<Option<Rc<RefCell<Document>>>>,
        pub current_slide_index: Cell<usize>,
    }

    impl Default for CanvasView {
        fn default() -> Self {
            Self {
                drawing_area: gtk::DrawingArea::new(),
                document: RefCell::new(None),
                current_slide_index: Cell::new(0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CanvasView {
        const NAME: &'static str = "LuminaCanvasView";
        type Type = super::CanvasView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }

    impl ObjectImpl for CanvasView {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            self.drawing_area.set_parent(&*obj);
            self.drawing_area.set_hexpand(true);
            self.drawing_area.set_vexpand(true);
        }

        fn dispose(&self) {
            self.drawing_area.unparent();
        }
    }

    impl WidgetImpl for CanvasView {}
}

glib::wrapper! {
    pub struct CanvasView(ObjectSubclass<imp::CanvasView>)
        @extends gtk::Widget;
}

impl CanvasView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_document(&self, doc: Rc<RefCell<Document>>) {
        let imp = self.imp();

        let doc_clone = doc.clone();
        let slide_index = imp.current_slide_index.clone();

        imp.drawing_area
            .set_draw_func(move |_area, cr, width, height| {
                let doc = doc_clone.borrow();
                let idx = slide_index.get();

                if idx >= doc.slides.len() {
                    return;
                }

                let slide = &doc.slides[idx];
                let slide_size = &doc.slide_size;

                draw_canvas_background(cr, width as f64, height as f64);
                draw_slide_with_shadow(cr, slide, slide_size, width as f64, height as f64);
            });

        *imp.document.borrow_mut() = Some(doc);
    }

    pub fn set_current_slide(&self, index: usize) {
        self.imp().current_slide_index.set(index);
        self.queue_draw();
    }

    pub fn current_slide_index(&self) -> usize {
        self.imp().current_slide_index.get()
    }

    pub fn queue_draw(&self) {
        self.imp().drawing_area.queue_draw();
    }

    pub fn slide_transform(&self) -> (f64, f64, f64) {
        let width = self.imp().drawing_area.width() as f64;
        let height = self.imp().drawing_area.height() as f64;

        let doc_ref = self.imp().document.borrow();
        if let Some(doc) = doc_ref.as_ref() {
            let doc = doc.borrow();
            let slide_size = &doc.slide_size;
            compute_slide_transform(slide_size, width, height)
        } else {
            (1.0, 0.0, 0.0)
        }
    }
}

fn compute_slide_transform(slide_size: &Size, width: f64, height: f64) -> (f64, f64, f64) {
    let padding = 0.9;
    let scale_x = width / slide_size.width;
    let scale_y = height / slide_size.height;
    let scale = scale_x.min(scale_y) * padding;

    let offset_x = (width - slide_size.width * scale) / 2.0;
    let offset_y = (height - slide_size.height * scale) / 2.0;

    (scale, offset_x, offset_y)
}

fn draw_canvas_background(cr: &cairo::Context, width: f64, height: f64) {
    cr.set_source_rgb(0.92, 0.92, 0.92);
    cr.rectangle(0.0, 0.0, width, height);
    let _ = cr.fill();
}

fn draw_slide_with_shadow(
    cr: &cairo::Context,
    slide: &crate::model::slide::Slide,
    slide_size: &Size,
    width: f64,
    height: f64,
) {
    let (scale, offset_x, offset_y) = compute_slide_transform(slide_size, width, height);

    cr.save().expect("cairo save");
    cr.translate(offset_x, offset_y);
    cr.scale(scale, scale);

    // Drop shadow
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.12);
    cr.rectangle(6.0, 6.0, slide_size.width, slide_size.height);
    let _ = cr.fill();

    // Slide border
    cr.set_source_rgb(0.78, 0.78, 0.78);
    cr.rectangle(-0.5, -0.5, slide_size.width + 1.0, slide_size.height + 1.0);
    let _ = cr.stroke();

    engine::render_slide(cr, slide, slide_size);

    cr.restore().expect("cairo restore");
}

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::model::document::Document;
use crate::render::engine;

mod imp {
    use super::*;

    pub struct SlidePanel {
        pub scrolled_window: gtk::ScrolledWindow,
        pub list_box: gtk::Box,
        pub document: RefCell<Option<Rc<RefCell<Document>>>>,
        pub selected_index: Cell<usize>,
        pub on_slide_selected: RefCell<Option<Box<dyn Fn(usize)>>>,
        pub thumbnails: RefCell<Vec<gtk::DrawingArea>>,
    }

    impl std::fmt::Debug for SlidePanel {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("SlidePanel").finish()
        }
    }

    impl Default for SlidePanel {
        fn default() -> Self {
            let list_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
            list_box.set_margin_start(8);
            list_box.set_margin_end(8);
            list_box.set_margin_top(8);
            list_box.set_margin_bottom(8);

            let scrolled_window = gtk::ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::Never)
                .vscrollbar_policy(gtk::PolicyType::Automatic)
                .child(&list_box)
                .build();

            Self {
                scrolled_window,
                list_box,
                document: RefCell::new(None),
                selected_index: Cell::new(0),
                on_slide_selected: RefCell::new(None),
                thumbnails: RefCell::new(Vec::new()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SlidePanel {
        const NAME: &'static str = "LuminaSlidePanel";
        type Type = super::SlidePanel;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }

    impl ObjectImpl for SlidePanel {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            self.scrolled_window.set_parent(&*obj);
        }

        fn dispose(&self) {
            self.scrolled_window.unparent();
        }
    }

    impl WidgetImpl for SlidePanel {}
}

glib::wrapper! {
    pub struct SlidePanel(ObjectSubclass<imp::SlidePanel>)
        @extends gtk::Widget;
}

impl SlidePanel {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_document(&self, doc: Rc<RefCell<Document>>) {
        *self.imp().document.borrow_mut() = Some(doc);
        self.rebuild_thumbnails();
    }

    pub fn connect_slide_selected<F: Fn(usize) + 'static>(&self, callback: F) {
        *self.imp().on_slide_selected.borrow_mut() = Some(Box::new(callback));
    }

    pub fn set_selected_index(&self, index: usize) {
        let prev = self.imp().selected_index.get();
        self.imp().selected_index.set(index);

        let thumbnails = self.imp().thumbnails.borrow();
        if prev < thumbnails.len() {
            update_thumbnail_style(&thumbnails[prev], false);
        }
        if index < thumbnails.len() {
            update_thumbnail_style(&thumbnails[index], true);
        }
    }

    pub fn rebuild_thumbnails(&self) {
        let imp = self.imp();
        let list_box = &imp.list_box;

        // Clear existing thumbnails
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
        imp.thumbnails.borrow_mut().clear();

        let doc_ref = imp.document.borrow();
        let Some(doc) = doc_ref.as_ref() else {
            return;
        };

        let doc_borrowed = doc.borrow();
        let slide_count = doc_borrowed.slides.len();
        let slide_size = doc_borrowed.slide_size;
        drop(doc_borrowed);

        let thumb_width = 200;
        let thumb_height = (thumb_width as f64 * slide_size.height / slide_size.width) as i32;

        for i in 0..slide_count {
            let frame = gtk::Box::new(gtk::Orientation::Vertical, 2);

            let label = gtk::Label::new(Some(&format!("{}", i + 1)));
            label.add_css_class("caption");
            label.set_opacity(0.6);

            let drawing_area = gtk::DrawingArea::new();
            drawing_area.set_content_width(thumb_width);
            drawing_area.set_content_height(thumb_height);

            let doc_clone = doc.clone();
            let slide_idx = i;
            drawing_area.set_draw_func(move |_area, cr, width, height| {
                let doc = doc_clone.borrow();
                if slide_idx >= doc.slides.len() {
                    return;
                }

                let slide = &doc.slides[slide_idx];
                let slide_size = &doc.slide_size;

                // White background
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.rectangle(0.0, 0.0, width as f64, height as f64);
                let _ = cr.fill();

                // Scale to fit
                let scale_x = width as f64 / slide_size.width;
                let scale_y = height as f64 / slide_size.height;
                let scale = scale_x.min(scale_y);

                cr.save().expect("save");
                cr.scale(scale, scale);
                engine::render_slide(cr, slide, slide_size);
                cr.restore().expect("restore");

                // Border
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.15);
                cr.rectangle(0.0, 0.0, width as f64, height as f64);
                cr.set_line_width(1.0);
                let _ = cr.stroke();
            });

            // Click handler
            let gesture = gtk::GestureClick::new();
            let panel = self.clone();
            let idx = i;
            gesture.connect_released(move |_, _, _, _| {
                panel.set_selected_index(idx);
                let cb = panel.imp().on_slide_selected.borrow();
                if let Some(callback) = cb.as_ref() {
                    callback(idx);
                }
            });
            frame.add_controller(gesture);

            frame.append(&drawing_area);
            frame.append(&label);
            list_box.append(&frame);

            self.imp().thumbnails.borrow_mut().push(drawing_area);
        }

        // Highlight the selected one
        let selected = imp.selected_index.get().min(slide_count.saturating_sub(1));
        let thumbnails = imp.thumbnails.borrow();
        if selected < thumbnails.len() {
            update_thumbnail_style(&thumbnails[selected], true);
        }
    }

    pub fn queue_draw_all(&self) {
        for thumb in self.imp().thumbnails.borrow().iter() {
            thumb.queue_draw();
        }
    }
}

fn update_thumbnail_style(drawing_area: &gtk::DrawingArea, selected: bool) {
    if selected {
        drawing_area.add_css_class("selected-thumbnail");
    } else {
        drawing_area.remove_css_class("selected-thumbnail");
    }
}

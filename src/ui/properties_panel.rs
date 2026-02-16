use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;

use crate::model::document::Document;
use crate::model::element::SlideElement;
use crate::model::style::Color;

mod imp {
    use super::*;

    pub struct PropertiesPanel {
        pub scrolled_window: gtk::ScrolledWindow,
        pub content_box: gtk::Box,
        pub document: RefCell<Option<Rc<RefCell<Document>>>>,
        pub selected_id: RefCell<Option<Uuid>>,
        pub slide_index: RefCell<usize>,
        pub on_property_changed: Rc<RefCell<Option<Box<dyn Fn()>>>>,
        pub updating: RefCell<bool>,
    }

    impl std::fmt::Debug for PropertiesPanel {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("PropertiesPanel").finish()
        }
    }

    impl Default for PropertiesPanel {
        fn default() -> Self {
            let content_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
            content_box.set_margin_start(12);
            content_box.set_margin_end(12);
            content_box.set_margin_top(12);
            content_box.set_margin_bottom(12);

            let scrolled_window = gtk::ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::Never)
                .vscrollbar_policy(gtk::PolicyType::Automatic)
                .child(&content_box)
                .build();

            Self {
                scrolled_window,
                content_box,
                document: RefCell::new(None),
                selected_id: RefCell::new(None),
                slide_index: RefCell::new(0),
                on_property_changed: Rc::new(RefCell::new(None)),
                updating: RefCell::new(false),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PropertiesPanel {
        const NAME: &'static str = "LuminaPropertiesPanel";
        type Type = super::PropertiesPanel;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }

    impl ObjectImpl for PropertiesPanel {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            self.scrolled_window.set_parent(&*obj);
        }

        fn dispose(&self) {
            self.scrolled_window.unparent();
        }
    }

    impl WidgetImpl for PropertiesPanel {}
}

glib::wrapper! {
    pub struct PropertiesPanel(ObjectSubclass<imp::PropertiesPanel>)
        @extends gtk::Widget;
}

impl PropertiesPanel {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_document(&self, doc: Rc<RefCell<Document>>) {
        *self.imp().document.borrow_mut() = Some(doc);
    }

    pub fn set_slide_index(&self, idx: usize) {
        *self.imp().slide_index.borrow_mut() = idx;
    }

    pub fn connect_property_changed<F: Fn() + 'static>(&self, callback: F) {
        *self.imp().on_property_changed.borrow_mut() = Some(Box::new(callback));
    }

    pub fn update_for_selection(&self, element_id: Option<Uuid>) {
        let imp = self.imp();
        *imp.selected_id.borrow_mut() = element_id;
        self.rebuild_ui();
    }

    fn rebuild_ui(&self) {
        let imp = self.imp();
        let content = &imp.content_box;

        // Clear existing children
        while let Some(child) = content.first_child() {
            content.remove(&child);
        }

        let sel_id = *imp.selected_id.borrow();
        let Some(sel_id) = sel_id else {
            let label = gtk::Label::new(Some(&gettext("No selection")));
            label.add_css_class("dim-label");
            label.set_margin_top(24);
            content.append(&label);
            return;
        };

        let doc_ref = imp.document.borrow();
        let Some(doc_rc) = doc_ref.as_ref() else { return };
        let doc = doc_rc.borrow();
        let idx = *imp.slide_index.borrow();
        if idx >= doc.slides.len() {
            return;
        }

        let slide = &doc.slides[idx];
        let Some(element) = slide.elements.iter().find(|e| e.id() == sel_id) else {
            return;
        };

        // Position & Size section
        self.build_position_section(content, element);

        // Type-specific properties
        match element {
            SlideElement::Text(text) => {
                self.build_text_properties(content, text);
            }
            SlideElement::Shape(shape) => {
                self.build_shape_properties(content, shape);
            }
            SlideElement::Image(_) => {
                let label = gtk::Label::new(Some(&gettext("Image")));
                label.add_css_class("heading");
                label.set_halign(gtk::Align::Start);
                content.append(&label);
            }
        }
    }

    fn build_position_section(&self, content: &gtk::Box, element: &SlideElement) {
        let imp = self.imp();
        let bounds = *element.bounds();

        let section_label = gtk::Label::new(Some(&gettext("Position & Size")));
        section_label.add_css_class("heading");
        section_label.set_halign(gtk::Align::Start);
        content.append(&section_label);

        let grid = gtk::Grid::new();
        grid.set_row_spacing(6);
        grid.set_column_spacing(8);

        let fields: Vec<(&str, f64)> = vec![
            ("X", bounds.origin.x),
            ("Y", bounds.origin.y),
            ("W", bounds.size.width),
            ("H", bounds.size.height),
        ];

        for (row, (label_text, value)) in fields.iter().enumerate() {
            let label = gtk::Label::new(Some(label_text));
            label.set_halign(gtk::Align::End);
            label.add_css_class("dim-label");
            label.set_width_chars(2);

            let spin = gtk::SpinButton::with_range(0.0, 10000.0, 1.0);
            spin.set_value(*value);
            spin.set_digits(1);
            spin.set_hexpand(true);

            let doc_rc = imp.document.borrow().clone();
            let sel_id = *imp.selected_id.borrow();
            let slide_idx = *imp.slide_index.borrow();
            let on_changed = imp.on_property_changed.clone();
            let updating = imp.updating.clone();
            let field_idx = row;

            spin.connect_value_changed(move |spin| {
                if *updating.borrow() {
                    return;
                }
                let Some(doc_rc) = doc_rc.as_ref() else { return };
                let Some(sel_id) = sel_id else { return };
                let mut doc = doc_rc.borrow_mut();
                if slide_idx >= doc.slides.len() {
                    return;
                }
                let slide = &mut doc.slides[slide_idx];
                if let Some(element) = slide.elements.iter_mut().find(|e| e.id() == sel_id) {
                    let bounds = element.bounds_mut();
                    let val = spin.value();
                    match field_idx {
                        0 => bounds.origin.x = val,
                        1 => bounds.origin.y = val,
                        2 => bounds.size.width = val,
                        3 => bounds.size.height = val,
                        _ => {}
                    }
                    if let Some(cb) = on_changed.borrow().as_ref() {
                        cb();
                    }
                }
            });

            grid.attach(&label, 0, row as i32, 1, 1);
            grid.attach(&spin, 1, row as i32, 1, 1);
        }

        content.append(&grid);

        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        sep.set_margin_top(8);
        sep.set_margin_bottom(4);
        content.append(&sep);
    }

    fn build_text_properties(
        &self,
        content: &gtk::Box,
        text: &crate::model::text::TextElement,
    ) {
        let imp = self.imp();

        let section_label = gtk::Label::new(Some(&gettext("Text")));
        section_label.add_css_class("heading");
        section_label.set_halign(gtk::Align::Start);
        content.append(&section_label);

        // Get font info from first run of first paragraph
        let (font_family, font_size, bold, italic, text_color) =
            if let Some(para) = text.paragraphs.first() {
                if let Some(run) = para.runs.first() {
                    (
                        run.font.family.clone(),
                        run.font.size,
                        run.font.bold,
                        run.font.italic,
                        run.font.color.clone(),
                    )
                } else {
                    default_font_info()
                }
            } else {
                default_font_info()
            };

        // Font family
        let font_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let font_label = gtk::Label::new(Some(&gettext("Font")));
        font_label.add_css_class("dim-label");
        font_label.set_width_chars(5);
        font_label.set_halign(gtk::Align::Start);

        let font_entry = gtk::Entry::new();
        font_entry.set_text(&font_family);
        font_entry.set_hexpand(true);

        let doc_rc = imp.document.borrow().clone();
        let sel_id = *imp.selected_id.borrow();
        let slide_idx = *imp.slide_index.borrow();
        let on_changed = imp.on_property_changed.clone();

        font_entry.connect_activate(move |entry| {
            let Some(doc_rc) = doc_rc.as_ref() else { return };
            let Some(sel_id) = sel_id else { return };
            let family = entry.text().to_string();
            let mut doc = doc_rc.borrow_mut();
            if slide_idx >= doc.slides.len() {
                return;
            }
            let slide = &mut doc.slides[slide_idx];
            if let Some(SlideElement::Text(text)) =
                slide.elements.iter_mut().find(|e| e.id() == sel_id)
            {
                for para in &mut text.paragraphs {
                    for run in &mut para.runs {
                        run.font.family = family.clone();
                    }
                }
                if let Some(cb) = on_changed.borrow().as_ref() {
                    cb();
                }
            }
        });

        font_row.append(&font_label);
        font_row.append(&font_entry);
        content.append(&font_row);

        // Font size
        let size_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let size_label = gtk::Label::new(Some(&gettext("Size")));
        size_label.add_css_class("dim-label");
        size_label.set_width_chars(5);
        size_label.set_halign(gtk::Align::Start);

        let size_spin = gtk::SpinButton::with_range(1.0, 500.0, 1.0);
        size_spin.set_value(font_size);
        size_spin.set_digits(0);
        size_spin.set_hexpand(true);

        let doc_rc = imp.document.borrow().clone();
        let sel_id = *imp.selected_id.borrow();
        let slide_idx = *imp.slide_index.borrow();
        let on_changed = imp.on_property_changed.clone();
        let updating = imp.updating.clone();

        size_spin.connect_value_changed(move |spin| {
            if *updating.borrow() {
                return;
            }
            let Some(doc_rc) = doc_rc.as_ref() else { return };
            let Some(sel_id) = sel_id else { return };
            let size = spin.value();
            let mut doc = doc_rc.borrow_mut();
            if slide_idx >= doc.slides.len() {
                return;
            }
            let slide = &mut doc.slides[slide_idx];
            if let Some(SlideElement::Text(text)) =
                slide.elements.iter_mut().find(|e| e.id() == sel_id)
            {
                for para in &mut text.paragraphs {
                    for run in &mut para.runs {
                        run.font.size = size;
                    }
                }
                if let Some(cb) = on_changed.borrow().as_ref() {
                    cb();
                }
            }
        });

        size_row.append(&size_label);
        size_row.append(&size_spin);
        content.append(&size_row);

        // Bold / Italic toggles
        let style_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let style_label = gtk::Label::new(Some(&gettext("Style")));
        style_label.add_css_class("dim-label");
        style_label.set_width_chars(5);
        style_label.set_halign(gtk::Align::Start);
        style_row.append(&style_label);

        let bold_btn = gtk::ToggleButton::new();
        bold_btn.set_icon_name("format-text-bold-symbolic");
        bold_btn.set_active(bold);

        let doc_rc = imp.document.borrow().clone();
        let sel_id = *imp.selected_id.borrow();
        let slide_idx = *imp.slide_index.borrow();
        let on_changed = imp.on_property_changed.clone();
        let updating = imp.updating.clone();

        bold_btn.connect_toggled(move |btn| {
            if *updating.borrow() {
                return;
            }
            let Some(doc_rc) = doc_rc.as_ref() else { return };
            let Some(sel_id) = sel_id else { return };
            let is_bold = btn.is_active();
            let mut doc = doc_rc.borrow_mut();
            if slide_idx >= doc.slides.len() {
                return;
            }
            let slide = &mut doc.slides[slide_idx];
            if let Some(SlideElement::Text(text)) =
                slide.elements.iter_mut().find(|e| e.id() == sel_id)
            {
                for para in &mut text.paragraphs {
                    for run in &mut para.runs {
                        run.font.bold = is_bold;
                    }
                }
                if let Some(cb) = on_changed.borrow().as_ref() {
                    cb();
                }
            }
        });

        let italic_btn = gtk::ToggleButton::new();
        italic_btn.set_icon_name("format-text-italic-symbolic");
        italic_btn.set_active(italic);

        let doc_rc = imp.document.borrow().clone();
        let sel_id = *imp.selected_id.borrow();
        let slide_idx = *imp.slide_index.borrow();
        let on_changed = imp.on_property_changed.clone();
        let updating = imp.updating.clone();

        italic_btn.connect_toggled(move |btn| {
            if *updating.borrow() {
                return;
            }
            let Some(doc_rc) = doc_rc.as_ref() else { return };
            let Some(sel_id) = sel_id else { return };
            let is_italic = btn.is_active();
            let mut doc = doc_rc.borrow_mut();
            if slide_idx >= doc.slides.len() {
                return;
            }
            let slide = &mut doc.slides[slide_idx];
            if let Some(SlideElement::Text(text)) =
                slide.elements.iter_mut().find(|e| e.id() == sel_id)
            {
                for para in &mut text.paragraphs {
                    for run in &mut para.runs {
                        run.font.italic = is_italic;
                    }
                }
                if let Some(cb) = on_changed.borrow().as_ref() {
                    cb();
                }
            }
        });

        style_row.append(&bold_btn);
        style_row.append(&italic_btn);
        content.append(&style_row);

        // Text color
        self.build_color_row(content, &gettext("Color"), &text_color, move |color| {
            // Color change callback - will be wired separately
            color
        });
    }

    fn build_shape_properties(
        &self,
        content: &gtk::Box,
        shape: &crate::model::shape::ShapeElement,
    ) {
        let imp = self.imp();

        let section_label = gtk::Label::new(Some(&gettext("Shape")));
        section_label.add_css_class("heading");
        section_label.set_halign(gtk::Align::Start);
        content.append(&section_label);

        // Fill color
        if let Some(fill) = &shape.fill {
            let doc_rc = imp.document.borrow().clone();
            let sel_id = *imp.selected_id.borrow();
            let slide_idx = *imp.slide_index.borrow();
            let on_changed = imp.on_property_changed.clone();

            self.build_color_button_row(content, &gettext("Fill"), &fill.color, move |color| {
                let Some(doc_rc) = doc_rc.as_ref() else { return };
                let Some(sel_id) = sel_id else { return };
                let mut doc = doc_rc.borrow_mut();
                if slide_idx >= doc.slides.len() {
                    return;
                }
                let slide = &mut doc.slides[slide_idx];
                if let Some(SlideElement::Shape(shape)) =
                    slide.elements.iter_mut().find(|e| e.id() == sel_id)
                {
                    if let Some(fill) = &mut shape.fill {
                        fill.color = color;
                    }
                    if let Some(cb) = on_changed.borrow().as_ref() {
                        cb();
                    }
                }
            });
        }

        // Stroke color & width
        if let Some(stroke) = &shape.stroke {
            let doc_rc = imp.document.borrow().clone();
            let sel_id = *imp.selected_id.borrow();
            let slide_idx = *imp.slide_index.borrow();
            let on_changed = imp.on_property_changed.clone();

            self.build_color_button_row(content, &gettext("Stroke"), &stroke.color, move |color| {
                let Some(doc_rc) = doc_rc.as_ref() else { return };
                let Some(sel_id) = sel_id else { return };
                let mut doc = doc_rc.borrow_mut();
                if slide_idx >= doc.slides.len() {
                    return;
                }
                let slide = &mut doc.slides[slide_idx];
                if let Some(SlideElement::Shape(shape)) =
                    slide.elements.iter_mut().find(|e| e.id() == sel_id)
                {
                    if let Some(stroke) = &mut shape.stroke {
                        stroke.color = color;
                    }
                    if let Some(cb) = on_changed.borrow().as_ref() {
                        cb();
                    }
                }
            });

            // Stroke width
            let width_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let width_label = gtk::Label::new(Some(&gettext("Width")));
            width_label.add_css_class("dim-label");
            width_label.set_width_chars(6);
            width_label.set_halign(gtk::Align::Start);

            let width_spin = gtk::SpinButton::with_range(0.5, 50.0, 0.5);
            width_spin.set_value(stroke.width);
            width_spin.set_digits(1);
            width_spin.set_hexpand(true);

            let doc_rc = imp.document.borrow().clone();
            let sel_id = *imp.selected_id.borrow();
            let slide_idx = *imp.slide_index.borrow();
            let on_changed = imp.on_property_changed.clone();
            let updating = imp.updating.clone();

            width_spin.connect_value_changed(move |spin| {
                if *updating.borrow() {
                    return;
                }
                let Some(doc_rc) = doc_rc.as_ref() else { return };
                let Some(sel_id) = sel_id else { return };
                let mut doc = doc_rc.borrow_mut();
                if slide_idx >= doc.slides.len() {
                    return;
                }
                let slide = &mut doc.slides[slide_idx];
                if let Some(SlideElement::Shape(shape)) =
                    slide.elements.iter_mut().find(|e| e.id() == sel_id)
                {
                    if let Some(stroke) = &mut shape.stroke {
                        stroke.width = spin.value();
                    }
                    if let Some(cb) = on_changed.borrow().as_ref() {
                        cb();
                    }
                }
            });

            width_row.append(&width_label);
            width_row.append(&width_spin);
            content.append(&width_row);
        }
    }

    fn build_color_row<F: Fn(Color) -> Color + 'static>(
        &self,
        content: &gtk::Box,
        label_text: &str,
        color: &Color,
        _transform: F,
    ) {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let label = gtk::Label::new(Some(label_text));
        label.add_css_class("dim-label");
        label.set_width_chars(5);
        label.set_halign(gtk::Align::Start);

        let rgba = gdk::RGBA::new(color.r as f32, color.g as f32, color.b as f32, color.a as f32);
        let color_dialog = gtk::ColorDialog::new();
        let color_btn = gtk::ColorDialogButton::new(Some(color_dialog));
        color_btn.set_rgba(&rgba);

        row.append(&label);
        row.append(&color_btn);
        content.append(&row);
    }

    fn build_color_button_row<F: Fn(Color) + 'static>(
        &self,
        content: &gtk::Box,
        label_text: &str,
        color: &Color,
        on_color_set: F,
    ) {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let label = gtk::Label::new(Some(label_text));
        label.add_css_class("dim-label");
        label.set_width_chars(6);
        label.set_halign(gtk::Align::Start);

        let rgba = gdk::RGBA::new(color.r as f32, color.g as f32, color.b as f32, color.a as f32);
        let color_dialog = gtk::ColorDialog::new();
        let color_btn = gtk::ColorDialogButton::new(Some(color_dialog));
        color_btn.set_rgba(&rgba);
        color_btn.set_hexpand(true);

        let on_color_set = Rc::new(on_color_set);
        color_btn.connect_rgba_notify(move |btn| {
            let rgba = btn.rgba();
            let color = Color::new(
                rgba.red() as f64,
                rgba.green() as f64,
                rgba.blue() as f64,
                rgba.alpha() as f64,
            );
            on_color_set(color);
        });

        row.append(&label);
        row.append(&color_btn);
        content.append(&row);
    }
}

fn default_font_info() -> (String, f64, bool, bool, Color) {
    ("Sans".to_string(), 24.0, false, false, Color::black())
}

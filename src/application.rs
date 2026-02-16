use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::gio;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config;
use crate::model::document::Document;
use crate::model::element::SlideElement;
use crate::model::geometry::Rect;
use crate::model::shape::{ShapeElement, ShapeType};
use crate::model::style::{Color, FillStyle, FontStyle, StrokeStyle};
use crate::model::text::{TextAlignment, TextElement, TextParagraph, TextRun};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct LuminaApplication;

    #[glib::object_subclass]
    impl ObjectSubclass for LuminaApplication {
        const NAME: &'static str = "LuminaApplication";
        type Type = super::LuminaApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for LuminaApplication {}

    impl ApplicationImpl for LuminaApplication {
        fn activate(&self) {
            let app = self.obj();

            let window = adw::ApplicationWindow::builder()
                .application(&*app)
                .default_width(1200)
                .default_height(800)
                .title("Lumina")
                .build();

            let header = adw::HeaderBar::new();
            let title = adw::WindowTitle::new("Lumina", "Presentation");
            header.set_title_widget(Some(&title));

            let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
            content.append(&header);

            let document = create_demo_document();
            let doc = Rc::new(RefCell::new(document));

            let drawing_area = gtk::DrawingArea::new();
            drawing_area.set_vexpand(true);
            drawing_area.set_hexpand(true);

            let doc_clone = doc.clone();
            drawing_area.set_draw_func(move |_area, cr, width, height| {
                let doc = doc_clone.borrow();
                if doc.slides.is_empty() {
                    return;
                }

                let slide = &doc.slides[0];
                let slide_size = &doc.slide_size;

                // Draw grey surround
                cr.set_source_rgb(0.85, 0.85, 0.85);
                cr.rectangle(0.0, 0.0, width as f64, height as f64);
                let _ = cr.fill();

                // Scale to fit the slide within the window
                let scale_x = width as f64 / slide_size.width;
                let scale_y = height as f64 / slide_size.height;
                let scale = scale_x.min(scale_y) * 0.9;

                let offset_x = (width as f64 - slide_size.width * scale) / 2.0;
                let offset_y = (height as f64 - slide_size.height * scale) / 2.0;

                cr.translate(offset_x, offset_y);
                cr.scale(scale, scale);

                // Drop shadow
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.15);
                cr.rectangle(4.0, 4.0, slide_size.width, slide_size.height);
                let _ = cr.fill();

                crate::render::engine::render_slide(cr, slide, slide_size);
            });

            content.append(&drawing_area);
            window.set_content(Some(&content));
            window.present();
        }
    }

    impl GtkApplicationImpl for LuminaApplication {}
    impl AdwApplicationImpl for LuminaApplication {}
}

fn create_demo_document() -> Document {
    let mut doc = Document::new();

    let slide = &mut doc.slides[0];

    // Title text
    let mut title = TextElement::new(
        Rect::new(80.0, 40.0, 800.0, 80.0),
        "",
    );
    title.paragraphs = vec![TextParagraph::new(vec![TextRun::new(
        "Welcome to Lumina",
        FontStyle {
            family: "Sans".to_string(),
            size: 48.0,
            bold: true,
            italic: false,
            color: Color::from_hex("#1c1c1c").unwrap(),
        },
    )])];
    title.alignment = TextAlignment::Center;
    slide.add_element(SlideElement::Text(title));

    // Subtitle
    let mut subtitle = TextElement::new(
        Rect::new(160.0, 140.0, 640.0, 50.0),
        "",
    );
    subtitle.paragraphs = vec![TextParagraph::new(vec![TextRun::new(
        "A modern presentation app for the GNOME desktop",
        FontStyle {
            family: "Sans".to_string(),
            size: 20.0,
            bold: false,
            italic: true,
            color: Color::from_hex("#555555").unwrap(),
        },
    )])];
    subtitle.alignment = TextAlignment::Center;
    slide.add_element(SlideElement::Text(subtitle));

    // Blue rectangle
    let mut rect = ShapeElement::new(
        Rect::new(100.0, 240.0, 340.0, 220.0),
        ShapeType::Rectangle,
    );
    rect.fill = Some(FillStyle::new(Color::from_hex("#3584e4").unwrap()));
    rect.stroke = None;
    slide.add_element(SlideElement::Shape(rect));

    // Yellow ellipse
    let mut ellipse = ShapeElement::new(
        Rect::new(520.0, 240.0, 340.0, 220.0),
        ShapeType::Ellipse,
    );
    ellipse.fill = Some(FillStyle::new(Color::from_hex("#f5c211").unwrap()));
    ellipse.stroke = Some(StrokeStyle::new(Color::from_hex("#a48102").unwrap(), 3.0));
    slide.add_element(SlideElement::Shape(ellipse));

    // Text on the blue rectangle
    let mut box_label = TextElement::new(
        Rect::new(140.0, 320.0, 260.0, 50.0),
        "",
    );
    box_label.paragraphs = vec![TextParagraph::new(vec![TextRun::new(
        "Shapes",
        FontStyle {
            family: "Sans".to_string(),
            size: 28.0,
            bold: true,
            italic: false,
            color: Color::white(),
        },
    )])];
    box_label.alignment = TextAlignment::Center;
    slide.add_element(SlideElement::Text(box_label));

    // Text on the yellow ellipse
    let mut ellipse_label = TextElement::new(
        Rect::new(560.0, 320.0, 260.0, 50.0),
        "",
    );
    ellipse_label.paragraphs = vec![TextParagraph::new(vec![TextRun::new(
        "Elements",
        FontStyle {
            family: "Sans".to_string(),
            size: 28.0,
            bold: true,
            italic: false,
            color: Color::from_hex("#1c1c1c").unwrap(),
        },
    )])];
    ellipse_label.alignment = TextAlignment::Center;
    slide.add_element(SlideElement::Text(ellipse_label));

    doc
}

glib::wrapper! {
    pub struct LuminaApplication(ObjectSubclass<imp::LuminaApplication>)
        @extends adw::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl LuminaApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("flags", gio::ApplicationFlags::FLAGS_NONE)
            .build()
    }
}

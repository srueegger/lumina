use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::gio;
use std::cell::RefCell;
use std::rc::Rc;

use crate::model::document::Document;
use crate::model::element::SlideElement;
use crate::model::geometry::Rect;
use crate::model::image::ImageElement;
use crate::model::shape::{ShapeElement, ShapeType};
use crate::model::style::{Color, FillStyle, FontStyle, StrokeStyle};
use crate::model::text::{TextAlignment, TextElement, TextParagraph, TextRun};
use crate::ui::canvas::tool::Tool;
use crate::ui::canvas_view::CanvasView;
use crate::ui::slide_panel::SlidePanel;

mod imp {
    use super::*;

    pub struct LuminaWindow {
        pub document: Rc<RefCell<Document>>,
        pub canvas: CanvasView,
        pub slide_panel: SlidePanel,
        pub header: adw::HeaderBar,
        pub tool_buttons: RefCell<Vec<(Tool, gtk::ToggleButton)>>,
    }

    impl std::fmt::Debug for LuminaWindow {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("LuminaWindow").finish()
        }
    }

    impl Default for LuminaWindow {
        fn default() -> Self {
            Self {
                document: Rc::new(RefCell::new(Document::new())),
                canvas: CanvasView::new(),
                slide_panel: SlidePanel::new(),
                header: adw::HeaderBar::new(),
                tool_buttons: RefCell::new(Vec::new()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LuminaWindow {
        const NAME: &'static str = "LuminaWindow";
        type Type = super::LuminaWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for LuminaWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_ui();
        }
    }

    impl WidgetImpl for LuminaWindow {}
    impl WindowImpl for LuminaWindow {}
    impl ApplicationWindowImpl for LuminaWindow {}
    impl AdwApplicationWindowImpl for LuminaWindow {}
}

glib::wrapper! {
    pub struct LuminaWindow(ObjectSubclass<imp::LuminaWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl LuminaWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window: Self = glib::Object::builder()
            .property("application", app)
            .property("default-width", 1200)
            .property("default-height", 800)
            .property("title", "Lumina")
            .build();

        window
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        // Create demo document
        let doc = create_demo_document();
        let doc = Rc::new(RefCell::new(doc));

        // Header bar
        let title = adw::WindowTitle::new("Lumina", "Untitled Presentation");
        imp.header.set_title_widget(Some(&title));

        // Add slide button in header
        let add_slide_btn = gtk::Button::from_icon_name("list-add-symbolic");
        add_slide_btn.set_tooltip_text(Some("Add Slide"));
        imp.header.pack_start(&add_slide_btn);

        // Separator
        let sep = gtk::Separator::new(gtk::Orientation::Vertical);
        imp.header.pack_start(&sep);

        // Tool buttons
        self.setup_tool_buttons(doc.clone());

        // Menu button
        let menu_btn = gtk::MenuButton::new();
        menu_btn.set_icon_name("open-menu-symbolic");
        menu_btn.set_tooltip_text(Some("Menu"));

        let menu = gio::Menu::new();
        menu.append(Some("About Lumina"), Some("app.about"));
        menu_btn.set_menu_model(Some(&menu));
        imp.header.pack_end(&menu_btn);

        // Main layout
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_box.append(&imp.header);

        // Content area: sidebar + canvas
        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.set_vexpand(true);
        paned.set_position(220);
        paned.set_shrink_start_child(false);
        paned.set_shrink_end_child(false);
        paned.set_resize_start_child(false);

        // Sidebar
        let sidebar_frame = gtk::Frame::new(None);
        sidebar_frame.set_child(Some(&imp.slide_panel));
        sidebar_frame.set_width_request(180);
        paned.set_start_child(Some(&sidebar_frame));

        // Canvas
        imp.canvas.set_hexpand(true);
        imp.canvas.set_vexpand(true);
        paned.set_end_child(Some(&imp.canvas));

        main_box.append(&paned);
        self.set_content(Some(&main_box));

        // Connect document
        imp.slide_panel.set_document(doc.clone());
        imp.canvas.set_document(doc.clone());

        // Slide selection
        let canvas = imp.canvas.clone();
        imp.slide_panel.connect_slide_selected(move |index| {
            canvas.set_current_slide(index);
        });

        // Refresh thumbnails when selection changes (element created/deleted)
        let panel_for_sel = imp.slide_panel.clone();
        imp.canvas.connect_selection_changed(move |_| {
            panel_for_sel.queue_draw_all();
        });

        // Add slide button
        let doc_clone = doc.clone();
        let panel_clone = imp.slide_panel.clone();
        let canvas_clone = imp.canvas.clone();
        add_slide_btn.connect_clicked(move |_| {
            let new_idx = {
                let mut doc = doc_clone.borrow_mut();
                let current = canvas_clone.current_slide_index();
                doc.insert_slide(current + 1)
            };
            panel_clone.rebuild_thumbnails();
            panel_clone.set_selected_index(new_idx);
            canvas_clone.set_current_slide(new_idx);
        });

        // Apply custom CSS
        let provider = gtk::CssProvider::new();
        provider.load_from_string(
            "
            .selected-thumbnail {
                outline: 3px solid @accent_color;
                outline-offset: 2px;
                border-radius: 4px;
            }
            .tool-active {
                background: alpha(@accent_color, 0.2);
            }
            ",
        );
        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().expect("display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn setup_tool_buttons(&self, doc: Rc<RefCell<Document>>) {
        let imp = self.imp();

        let tools: Vec<(Tool, &str, &str)> = vec![
            (Tool::Pointer, "edit-select-symbolic", "Pointer (Esc)"),
            (Tool::Text, "insert-text-symbolic", "Text"),
            (
                Tool::Shape(ShapeType::Rectangle),
                "checkbox-symbolic",
                "Rectangle",
            ),
            (
                Tool::Shape(ShapeType::Ellipse),
                "color-select-symbolic",
                "Ellipse",
            ),
            (
                Tool::Shape(ShapeType::Line),
                "format-text-strikethrough-symbolic",
                "Line",
            ),
            (Tool::Image, "insert-image-symbolic", "Image"),
        ];

        let pointer_btn = gtk::ToggleButton::new();
        pointer_btn.set_icon_name(tools[0].1);
        pointer_btn.set_tooltip_text(Some(tools[0].2));
        pointer_btn.set_active(true);
        imp.header.pack_start(&pointer_btn);

        let mut all_buttons: Vec<(Tool, gtk::ToggleButton)> = vec![];
        all_buttons.push((Tool::Pointer, pointer_btn.clone()));

        for (tool, icon, tooltip) in tools.iter().skip(1) {
            let btn = gtk::ToggleButton::new();
            btn.set_icon_name(icon);
            btn.set_tooltip_text(Some(tooltip));
            btn.set_group(Some(&pointer_btn));
            imp.header.pack_start(&btn);
            all_buttons.push((*tool, btn));
        }

        // Connect tool button clicks
        let canvas = imp.canvas.clone();
        let doc_for_image = doc;
        let buttons_rc = Rc::new(RefCell::new(all_buttons.clone()));

        for (tool, btn) in &all_buttons {
            let tool = *tool;
            let canvas = canvas.clone();
            let doc_for_image = doc_for_image.clone();
            let buttons = buttons_rc.clone();

            btn.connect_toggled(move |btn| {
                if !btn.is_active() {
                    return;
                }

                if matches!(tool, Tool::Image) {
                    // Image tool: open file chooser immediately, then reset to pointer
                    Self::open_image_dialog(&canvas, &doc_for_image, &buttons);
                    return;
                }

                canvas.set_current_tool(tool);
            });
        }

        // Listen for tool changes from canvas (e.g., after element creation)
        let buttons_for_cb = buttons_rc;
        imp.canvas.connect_tool_changed(move |tool| {
            let buttons = buttons_for_cb.borrow();
            for (t, btn) in buttons.iter() {
                if *t == tool {
                    btn.set_active(true);
                    break;
                }
            }
        });

        *imp.tool_buttons.borrow_mut() = all_buttons;
    }

    fn open_image_dialog(
        canvas: &CanvasView,
        doc: &Rc<RefCell<Document>>,
        buttons: &Rc<RefCell<Vec<(Tool, gtk::ToggleButton)>>>,
    ) {
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Images"));
        filter.add_mime_type("image/png");
        filter.add_mime_type("image/jpeg");
        filter.add_mime_type("image/svg+xml");
        filter.add_mime_type("image/webp");

        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);

        let dialog = gtk::FileDialog::builder()
            .title("Insert Image")
            .filters(&filters)
            .build();

        let canvas = canvas.clone();
        let doc = doc.clone();
        let buttons = buttons.clone();

        let window = canvas
            .root()
            .and_then(|r| r.downcast::<gtk::Window>().ok());

        dialog.open(window.as_ref(), gio::Cancellable::NONE, move |result| {
            // Reset to pointer tool regardless
            canvas.set_current_tool(Tool::Pointer);
            let btns = buttons.borrow();
            for (t, btn) in btns.iter() {
                if matches!(t, Tool::Pointer) {
                    btn.set_active(true);
                    break;
                }
            }

            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    if let Ok(data) = std::fs::read(&path) {
                        let mime = match path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                        {
                            "png" => "image/png",
                            "jpg" | "jpeg" => "image/jpeg",
                            "svg" => "image/svg+xml",
                            "webp" => "image/webp",
                            _ => "image/png",
                        };

                        let bounds = Rect::new(100.0, 100.0, 400.0, 300.0);
                        let element = ImageElement::new(bounds, data, mime.to_string());
                        let element_id = element.id;

                        let idx = canvas.current_slide_index();
                        {
                            let mut doc = doc.borrow_mut();
                            if idx < doc.slides.len() {
                                doc.slides[idx].add_element(SlideElement::Image(element));
                            }
                        }

                        canvas.selection().borrow_mut().select(element_id);
                        canvas.queue_draw();
                    }
                }
            }
        });
    }
}

fn create_demo_document() -> Document {
    let mut doc = Document::new();

    // Slide 1: Title slide
    {
        let slide = &mut doc.slides[0];

        let mut title = TextElement::new(Rect::new(80.0, 160.0, 800.0, 80.0), "");
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

        let mut subtitle = TextElement::new(Rect::new(160.0, 260.0, 640.0, 50.0), "");
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
    }

    // Slide 2: Shapes demo
    doc.add_slide();
    {
        let slide = &mut doc.slides[1];

        let mut heading = TextElement::new(Rect::new(40.0, 30.0, 880.0, 60.0), "");
        heading.paragraphs = vec![TextParagraph::new(vec![TextRun::new(
            "Shape Elements",
            FontStyle {
                family: "Sans".to_string(),
                size: 36.0,
                bold: true,
                italic: false,
                color: Color::from_hex("#1c1c1c").unwrap(),
            },
        )])];
        slide.add_element(SlideElement::Text(heading));

        let mut rect = ShapeElement::new(Rect::new(60.0, 130.0, 250.0, 180.0), ShapeType::Rectangle);
        rect.fill = Some(FillStyle::new(Color::from_hex("#3584e4").unwrap()));
        rect.stroke = None;
        slide.add_element(SlideElement::Shape(rect));

        let mut ellipse =
            ShapeElement::new(Rect::new(355.0, 130.0, 250.0, 180.0), ShapeType::Ellipse);
        ellipse.fill = Some(FillStyle::new(Color::from_hex("#f5c211").unwrap()));
        ellipse.stroke = Some(StrokeStyle::new(Color::from_hex("#a48102").unwrap(), 3.0));
        slide.add_element(SlideElement::Shape(ellipse));

        let mut rect2 =
            ShapeElement::new(Rect::new(650.0, 130.0, 250.0, 180.0), ShapeType::Rectangle);
        rect2.fill = Some(FillStyle::new(Color::from_hex("#33d17a").unwrap()));
        rect2.stroke = None;
        slide.add_element(SlideElement::Shape(rect2));

        let mut line = ShapeElement::new(Rect::new(60.0, 360.0, 840.0, 0.0), ShapeType::Line);
        line.stroke = Some(StrokeStyle::new(Color::from_hex("#c01c28").unwrap(), 3.0));
        slide.add_element(SlideElement::Shape(line));

        let mut footer = TextElement::new(Rect::new(60.0, 400.0, 840.0, 40.0), "");
        footer.paragraphs = vec![TextParagraph::new(vec![TextRun::new(
            "Lumina supports rectangles, ellipses, lines, and more.",
            FontStyle {
                family: "Sans".to_string(),
                size: 16.0,
                bold: false,
                italic: false,
                color: Color::from_hex("#555555").unwrap(),
            },
        )])];
        footer.alignment = TextAlignment::Center;
        slide.add_element(SlideElement::Text(footer));
    }

    // Slide 3: Empty slide
    doc.add_slide();

    doc
}

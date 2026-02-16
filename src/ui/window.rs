use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::gio;
use std::cell::RefCell;
use std::rc::Rc;

use crate::format::odp;
use crate::render::pdf_export;
use crate::model::document::Document;
use crate::model::element::SlideElement;
use crate::model::geometry::Rect;
use crate::model::image::ImageElement;
use crate::model::shape::{ShapeElement, ShapeType};
use crate::model::style::{Color, FillStyle, FontStyle, StrokeStyle};
use crate::model::text::{TextAlignment, TextElement, TextParagraph, TextRun};
use crate::ui::canvas::tool::Tool;
use crate::ui::canvas_view::CanvasView;
use crate::ui::properties_panel::PropertiesPanel;
use crate::ui::slide_panel::SlidePanel;

mod imp {
    use super::*;

    pub struct LuminaWindow {
        pub document: Rc<RefCell<Document>>,
        pub canvas: CanvasView,
        pub slide_panel: SlidePanel,
        pub properties_panel: PropertiesPanel,
        pub header: adw::HeaderBar,
        pub title_widget: RefCell<Option<adw::WindowTitle>>,
        pub tool_buttons: RefCell<Vec<(Tool, gtk::ToggleButton)>>,
        pub file_path: Rc<RefCell<Option<std::path::PathBuf>>>,
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
                properties_panel: PropertiesPanel::new(),
                header: adw::HeaderBar::new(),
                title_widget: RefCell::new(None),
                tool_buttons: RefCell::new(Vec::new()),
                file_path: Rc::new(RefCell::new(None)),
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
        *imp.title_widget.borrow_mut() = Some(title);

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
        let file_section = gio::Menu::new();
        file_section.append(Some("Open..."), Some("win.open"));
        file_section.append(Some("Save"), Some("win.save"));
        file_section.append(Some("Save As..."), Some("win.save-as"));
        menu.append_section(None, &file_section);
        let export_section = gio::Menu::new();
        export_section.append(Some("Export as PDF..."), Some("win.export-pdf"));
        menu.append_section(None, &export_section);
        let about_section = gio::Menu::new();
        about_section.append(Some("About Lumina"), Some("app.about"));
        menu.append_section(None, &about_section);
        menu_btn.set_menu_model(Some(&menu));
        imp.header.pack_end(&menu_btn);

        // Main layout
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_box.append(&imp.header);

        // Content area: sidebar + canvas + properties
        let left_paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        left_paned.set_vexpand(true);
        left_paned.set_position(220);
        left_paned.set_shrink_start_child(false);
        left_paned.set_shrink_end_child(false);
        left_paned.set_resize_start_child(false);

        // Sidebar
        let sidebar_frame = gtk::Frame::new(None);
        sidebar_frame.set_child(Some(&imp.slide_panel));
        sidebar_frame.set_width_request(180);
        left_paned.set_start_child(Some(&sidebar_frame));

        // Right paned: canvas + properties panel
        let right_paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        right_paned.set_shrink_start_child(false);
        right_paned.set_shrink_end_child(false);
        right_paned.set_resize_end_child(false);

        // Canvas
        imp.canvas.set_hexpand(true);
        imp.canvas.set_vexpand(true);
        right_paned.set_start_child(Some(&imp.canvas));

        // Properties panel
        let props_frame = gtk::Frame::new(None);
        props_frame.set_child(Some(&imp.properties_panel));
        props_frame.set_width_request(240);
        right_paned.set_end_child(Some(&props_frame));

        left_paned.set_end_child(Some(&right_paned));

        main_box.append(&left_paned);
        self.set_content(Some(&main_box));

        // Connect document
        imp.slide_panel.set_document(doc.clone());
        imp.canvas.set_document(doc.clone());
        imp.properties_panel.set_document(doc.clone());

        // Slide selection
        let canvas = imp.canvas.clone();
        imp.slide_panel.connect_slide_selected(move |index| {
            canvas.set_current_slide(index);
        });

        // Refresh thumbnails and properties panel when selection changes
        let panel_for_sel = imp.slide_panel.clone();
        let props_for_sel = imp.properties_panel.clone();
        let canvas_for_sel = imp.canvas.clone();
        imp.canvas.connect_selection_changed(move |sel_id| {
            panel_for_sel.queue_draw_all();
            props_for_sel.set_slide_index(canvas_for_sel.current_slide_index());
            props_for_sel.update_for_selection(sel_id);
        });

        // When properties change, redraw canvas and thumbnails
        let canvas_for_props = imp.canvas.clone();
        let panel_for_props = imp.slide_panel.clone();
        imp.properties_panel.connect_property_changed(move || {
            canvas_for_props.queue_draw();
            panel_for_props.queue_draw_all();
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

        // File actions
        self.setup_file_actions(doc);
    }

    fn setup_file_actions(&self, doc: Rc<RefCell<Document>>) {
        let imp = self.imp();

        // Save action
        let save_action = gio::ActionEntry::builder("save")
            .activate({
                let doc = doc.clone();
                let file_path = imp.file_path.clone();
                move |win: &LuminaWindow, _, _| {
                    let path = file_path.borrow().clone();
                    if let Some(path) = path {
                        let doc = doc.borrow();
                        if let Err(e) = odp::writer::save_document(&doc, &path) {
                            eprintln!("Save error: {}", e);
                        }
                    } else {
                        // No file path yet, trigger Save As
                        gio::prelude::ActionGroupExt::activate_action(win, "save-as", None);
                    }
                }
            })
            .build();

        // Save As action
        let save_as_action = gio::ActionEntry::builder("save-as")
            .activate({
                let doc = doc.clone();
                let file_path = imp.file_path.clone();
                let title_widget = imp.title_widget.clone();
                move |win: &LuminaWindow, _, _| {
                    let filter = gtk::FileFilter::new();
                    filter.set_name(Some("ODP Presentation"));
                    filter.add_mime_type("application/vnd.oasis.opendocument.presentation");
                    filter.add_pattern("*.odp");

                    let filters = gio::ListStore::new::<gtk::FileFilter>();
                    filters.append(&filter);

                    let dialog = gtk::FileDialog::builder()
                        .title("Save Presentation")
                        .filters(&filters)
                        .initial_name("presentation.odp")
                        .build();

                    let doc = doc.clone();
                    let file_path = file_path.clone();
                    let title_widget = title_widget.clone();
                    dialog.save(Some(win), gio::Cancellable::NONE, move |result| {
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                let doc = doc.borrow();
                                if let Err(e) = odp::writer::save_document(&doc, &path) {
                                    eprintln!("Save error: {}", e);
                                    return;
                                }
                                let filename = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Untitled");
                                if let Some(title) = title_widget.borrow().as_ref() {
                                    title.set_subtitle(filename);
                                }
                                *file_path.borrow_mut() = Some(path);
                            }
                        }
                    });
                }
            })
            .build();

        // Open action
        let open_action = gio::ActionEntry::builder("open")
            .activate({
                let doc = doc.clone();
                let file_path = imp.file_path.clone();
                let title_widget = imp.title_widget.clone();
                let slide_panel = imp.slide_panel.clone();
                let canvas = imp.canvas.clone();
                let props = imp.properties_panel.clone();
                move |win: &LuminaWindow, _, _| {
                    let odp_filter = gtk::FileFilter::new();
                    odp_filter.set_name(Some("ODP Presentation"));
                    odp_filter.add_mime_type("application/vnd.oasis.opendocument.presentation");
                    odp_filter.add_pattern("*.odp");

                    let pptx_filter = gtk::FileFilter::new();
                    pptx_filter.set_name(Some("PowerPoint Presentation"));
                    pptx_filter.add_mime_type("application/vnd.openxmlformats-officedocument.presentationml.presentation");
                    pptx_filter.add_pattern("*.pptx");

                    let all_filter = gtk::FileFilter::new();
                    all_filter.set_name(Some("All Presentations"));
                    all_filter.add_pattern("*.odp");
                    all_filter.add_pattern("*.pptx");

                    let filters = gio::ListStore::new::<gtk::FileFilter>();
                    filters.append(&all_filter);
                    filters.append(&odp_filter);
                    filters.append(&pptx_filter);

                    let dialog = gtk::FileDialog::builder()
                        .title("Open Presentation")
                        .filters(&filters)
                        .build();

                    let doc = doc.clone();
                    let file_path = file_path.clone();
                    let title_widget = title_widget.clone();
                    let slide_panel = slide_panel.clone();
                    let canvas = canvas.clone();
                    let props = props.clone();

                    dialog.open(Some(win), gio::Cancellable::NONE, move |result| {
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                let load_result = if path.extension().and_then(|e| e.to_str()) == Some("pptx") {
                                    crate::format::pptx::reader::load_document(&path)
                                } else {
                                    odp::reader::load_document(&path)
                                };
                                let is_pptx = path.extension().and_then(|e| e.to_str()) == Some("pptx");
                                match load_result {
                                    Ok(loaded_doc) => {
                                        *doc.borrow_mut() = loaded_doc;
                                        let filename = path
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("Untitled");
                                        if let Some(title) = title_widget.borrow().as_ref() {
                                            title.set_subtitle(filename);
                                        }
                                        // Don't set file_path for PPTX (import only)
                                        if !is_pptx {
                                            *file_path.borrow_mut() = Some(path);
                                        } else {
                                            *file_path.borrow_mut() = None;
                                        }
                                        slide_panel.rebuild_thumbnails();
                                        canvas.set_current_slide(0);
                                        props.update_for_selection(None);
                                    }
                                    Err(e) => {
                                        eprintln!("Open error: {}", e);
                                    }
                                }
                            }
                        }
                    });
                }
            })
            .build();

        // Export PDF action
        let export_pdf_action = gio::ActionEntry::builder("export-pdf")
            .activate({
                let doc = doc.clone();
                move |win: &LuminaWindow, _, _| {
                    let filter = gtk::FileFilter::new();
                    filter.set_name(Some("PDF Document"));
                    filter.add_mime_type("application/pdf");
                    filter.add_pattern("*.pdf");

                    let filters = gio::ListStore::new::<gtk::FileFilter>();
                    filters.append(&filter);

                    let dialog = gtk::FileDialog::builder()
                        .title("Export as PDF")
                        .filters(&filters)
                        .initial_name("presentation.pdf")
                        .build();

                    let doc = doc.clone();

                    dialog.save(Some(win), gio::Cancellable::NONE, move |result| {
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                let doc = doc.borrow();
                                if let Err(e) = pdf_export::export_pdf(&doc, &path) {
                                    eprintln!("PDF export error: {}", e);
                                }
                            }
                        }
                    });
                }
            })
            .build();

        self.add_action_entries([save_action, save_as_action, open_action, export_pdf_action]);
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

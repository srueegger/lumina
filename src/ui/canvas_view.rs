use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::model::document::Document;
use crate::model::element::SlideElement;
use crate::model::geometry::{Rect, Size};
use crate::model::shape::ShapeElement;
use crate::model::text::TextElement;
use crate::render::engine;
use crate::ui::canvas::interaction::{self, DragOperation};
use crate::ui::canvas::selection::{self, Selection};
use crate::ui::canvas::tool::Tool;

mod imp {
    use super::*;

    pub struct CanvasView {
        pub drawing_area: gtk::DrawingArea,
        pub document: RefCell<Option<Rc<RefCell<Document>>>>,
        pub current_slide_index: Cell<usize>,
        pub selection: Rc<RefCell<Selection>>,
        pub drag_op: Rc<RefCell<Option<DragOperation>>>,
        pub current_tool: Rc<Cell<Tool>>,
        pub on_selection_changed: Rc<RefCell<Option<Box<dyn Fn(Option<uuid::Uuid>)>>>>,
        pub on_tool_changed: Rc<RefCell<Option<Box<dyn Fn(Tool)>>>>,
    }

    impl std::fmt::Debug for CanvasView {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("CanvasView").finish()
        }
    }

    impl Default for CanvasView {
        fn default() -> Self {
            Self {
                drawing_area: gtk::DrawingArea::new(),
                document: RefCell::new(None),
                current_slide_index: Cell::new(0),
                selection: Rc::new(RefCell::new(Selection::new())),
                drag_op: Rc::new(RefCell::new(None)),
                current_tool: Rc::new(Cell::new(Tool::Pointer)),
                on_selection_changed: Rc::new(RefCell::new(None)),
                on_tool_changed: Rc::new(RefCell::new(None)),
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
            self.drawing_area.set_focusable(true);
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
        let selection = imp.selection.clone();
        let drag_op_for_draw = imp.drag_op.clone();
        let current_tool_for_draw = imp.current_tool.clone();

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

                let (scale, offset_x, offset_y) =
                    compute_slide_transform(slide_size, width as f64, height as f64);

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

                // Draw selection handles
                let sel = selection.borrow();
                if let Some(sel_id) = sel.element_id {
                    for element in &slide.elements {
                        if element.id() == sel_id {
                            selection::render_selection_handles(cr, element.bounds());
                            break;
                        }
                    }
                }

                let _ = (&drag_op_for_draw, &current_tool_for_draw);

                cr.restore().expect("cairo restore");
            });

        // Set up click handler
        self.setup_click_handler(doc.clone());
        self.setup_drag_handler(doc.clone());
        self.setup_key_handler(doc.clone());

        *imp.document.borrow_mut() = Some(doc);
    }

    fn setup_click_handler(&self, doc: Rc<RefCell<Document>>) {
        let imp = self.imp();
        let gesture = gtk::GestureClick::new();

        let selection = imp.selection.clone();
        let slide_index = imp.current_slide_index.clone();
        let drawing_area = imp.drawing_area.clone();
        let on_changed = imp.on_selection_changed.clone();
        let current_tool = imp.current_tool.clone();

        gesture.connect_pressed(move |_gesture, _n_press, x, y| {
            let tool = current_tool.get();

            // For creation tools, clicking is handled by drag handler
            if !matches!(tool, Tool::Pointer) {
                return;
            }

            let doc = doc.borrow();
            let idx = slide_index.get();
            if idx >= doc.slides.len() {
                return;
            }

            let slide = &doc.slides[idx];
            let slide_size = &doc.slide_size;
            let width = drawing_area.width() as f64;
            let height = drawing_area.height() as f64;
            let (scale, offset_x, offset_y) = compute_slide_transform(slide_size, width, height);

            let slide_point = interaction::widget_to_slide_coords(x, y, scale, offset_x, offset_y);

            let mut sel = selection.borrow_mut();

            if let Some((_idx, element)) = slide.find_element_at(slide_point) {
                sel.select(element.id());
                let id = Some(element.id());
                if let Some(cb) = on_changed.borrow().as_ref() {
                    cb(id);
                }
            } else {
                sel.deselect();
                if let Some(cb) = on_changed.borrow().as_ref() {
                    cb(None);
                }
            }

            drawing_area.queue_draw();
        });

        imp.drawing_area.add_controller(gesture);
    }

    fn setup_drag_handler(&self, doc: Rc<RefCell<Document>>) {
        let imp = self.imp();
        let gesture = gtk::GestureDrag::new();

        let selection = imp.selection.clone();
        let drag_op = imp.drag_op.clone();
        let slide_index = imp.current_slide_index.clone();
        let drawing_area = imp.drawing_area.clone();
        let current_tool = imp.current_tool.clone();
        let doc_for_drag = doc.clone();
        let doc_for_update = doc.clone();
        let doc_for_end = doc;

        let selection_start = selection.clone();
        let drag_op_start = drag_op.clone();
        let slide_index_start = slide_index.clone();
        let drawing_area_start = drawing_area.clone();
        let current_tool_start = current_tool.clone();

        gesture.connect_drag_begin(move |_gesture, x, y| {
            let doc = doc_for_drag.borrow();
            let idx = slide_index_start.get();
            if idx >= doc.slides.len() {
                return;
            }

            let slide = &doc.slides[idx];
            let slide_size = &doc.slide_size;
            let width = drawing_area_start.width() as f64;
            let height = drawing_area_start.height() as f64;
            let (scale, offset_x, offset_y) = compute_slide_transform(slide_size, width, height);

            let slide_point = interaction::widget_to_slide_coords(x, y, scale, offset_x, offset_y);

            let tool = current_tool_start.get();

            // Creation tools: start a create drag
            if !matches!(tool, Tool::Pointer) {
                *drag_op_start.borrow_mut() = Some(DragOperation::Create {
                    tool,
                    start: slide_point,
                });
                return;
            }

            // Pointer tool: move/resize existing elements
            let sel = selection_start.borrow();
            if let Some(sel_id) = sel.element_id {
                for element in &slide.elements {
                    if element.id() == sel_id {
                        if let Some(handle) =
                            selection::hit_test_handle(slide_point, element.bounds())
                        {
                            *drag_op_start.borrow_mut() = Some(DragOperation::Resize {
                                handle,
                                orig_bounds: *element.bounds(),
                            });
                            return;
                        }

                        if element.bounds().contains(slide_point) {
                            *drag_op_start.borrow_mut() = Some(DragOperation::Move {
                                start_x: slide_point.x,
                                start_y: slide_point.y,
                                orig_bounds: *element.bounds(),
                            });
                            return;
                        }
                    }
                }
            }
        });

        let selection_update = selection.clone();
        let drag_op_update = drag_op.clone();
        let slide_index_update = slide_index.clone();
        let drawing_area_update = drawing_area.clone();

        gesture.connect_drag_update(move |_gesture, offset_x, offset_y| {
            let op = drag_op_update.borrow();
            if op.is_none() {
                return;
            }

            let is_create = matches!(op.as_ref(), Some(DragOperation::Create { .. }));

            if is_create {
                // For creation, just redraw to show preview
                drop(op);
                // We update the drag offset in a different way for create:
                // store the offset so draw_func can compute the preview rect
                drawing_area_update.queue_draw();
                return;
            }

            let mut doc = doc_for_update.borrow_mut();
            let idx = slide_index_update.get();
            if idx >= doc.slides.len() {
                return;
            }

            let slide_size = doc.slide_size;
            let width = drawing_area_update.width() as f64;
            let height = drawing_area_update.height() as f64;
            let (scale, _, _) = compute_slide_transform(&slide_size, width, height);

            let dx = offset_x / scale;
            let dy = offset_y / scale;

            let sel = selection_update.borrow();
            if let Some(sel_id) = sel.element_id {
                if let Some(op) = op.as_ref() {
                    let new_bounds = op.apply(dx, dy);

                    let slide = &mut doc.slides[idx];
                    for element in &mut slide.elements {
                        if element.id() == sel_id {
                            *element.bounds_mut() = new_bounds;
                            break;
                        }
                    }
                }
            }

            drawing_area_update.queue_draw();
        });

        let drag_op_end = drag_op.clone();
        let selection_end = selection;
        let slide_index_end = slide_index;
        let drawing_area_end = drawing_area.clone();
        let current_tool_end = current_tool.clone();
        let on_changed_end = imp.on_selection_changed.clone();
        let on_tool_changed_end = imp.on_tool_changed.clone();

        gesture.connect_drag_end(move |gesture, offset_x, offset_y| {
            let op = drag_op_end.borrow().clone();
            *drag_op_end.borrow_mut() = None;

            if let Some(DragOperation::Create { tool, start }) = op {
                let slide_size;
                let scale;
                {
                    let doc = doc_for_end.borrow();
                    slide_size = doc.slide_size;
                    let width = drawing_area_end.width() as f64;
                    let height = drawing_area_end.height() as f64;
                    let transform = compute_slide_transform(&slide_size, width, height);
                    scale = transform.0;
                }

                let dx = offset_x / scale;
                let dy = offset_y / scale;

                // Require minimum drag distance to create element
                if dx.abs() < 5.0 && dy.abs() < 5.0 {
                    return;
                }

                let bounds = interaction::normalize_rect(
                    start.x,
                    start.y,
                    start.x + dx,
                    start.y + dy,
                );

                let element = create_element_for_tool(tool, bounds);
                if let Some(element) = element {
                    let element_id = element.id();
                    {
                        let mut doc = doc_for_end.borrow_mut();
                        let idx = slide_index_end.get();
                        if idx < doc.slides.len() {
                            doc.slides[idx].add_element(element);
                        }
                    }

                    // Select the newly created element
                    selection_end.borrow_mut().select(element_id);
                    if let Some(cb) = on_changed_end.borrow().as_ref() {
                        cb(Some(element_id));
                    }

                    // Switch back to pointer tool
                    current_tool_end.set(Tool::Pointer);
                    if let Some(cb) = on_tool_changed_end.borrow().as_ref() {
                        cb(Tool::Pointer);
                    }
                }

                drawing_area_end.queue_draw();

                // Cancel the gesture to avoid interfering with click handler
                gesture.set_state(gtk::EventSequenceState::Claimed);
            }
        });

        imp.drawing_area.add_controller(gesture);
    }

    fn setup_key_handler(&self, doc: Rc<RefCell<Document>>) {
        let imp = self.imp();
        let key_controller = gtk::EventControllerKey::new();

        let selection = imp.selection.clone();
        let slide_index = imp.current_slide_index.clone();
        let drawing_area = imp.drawing_area.clone();
        let on_changed = imp.on_selection_changed.clone();
        let current_tool = imp.current_tool.clone();
        let on_tool_changed = imp.on_tool_changed.clone();

        key_controller.connect_key_pressed(move |_, keyval, _, _| {
            if keyval == gdk::Key::Delete || keyval == gdk::Key::BackSpace {
                let mut sel = selection.borrow_mut();
                if let Some(sel_id) = sel.element_id {
                    let mut doc = doc.borrow_mut();
                    let idx = slide_index.get();
                    if idx < doc.slides.len() {
                        doc.slides[idx].remove_element(sel_id);
                        sel.deselect();
                        if let Some(cb) = on_changed.borrow().as_ref() {
                            cb(None);
                        }
                        drawing_area.queue_draw();
                    }
                }
                return glib::Propagation::Stop;
            }
            if keyval == gdk::Key::Escape {
                // If a creation tool is active, switch back to pointer
                let tool = current_tool.get();
                if !matches!(tool, Tool::Pointer) {
                    current_tool.set(Tool::Pointer);
                    if let Some(cb) = on_tool_changed.borrow().as_ref() {
                        cb(Tool::Pointer);
                    }
                    return glib::Propagation::Stop;
                }

                let mut sel = selection.borrow_mut();
                sel.deselect();
                if let Some(cb) = on_changed.borrow().as_ref() {
                    cb(None);
                }
                drawing_area.queue_draw();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });

        imp.drawing_area.add_controller(key_controller);
    }

    pub fn connect_selection_changed<F: Fn(Option<uuid::Uuid>) + 'static>(&self, callback: F) {
        *self.imp().on_selection_changed.borrow_mut() = Some(Box::new(callback));
    }

    pub fn connect_tool_changed<F: Fn(Tool) + 'static>(&self, callback: F) {
        *self.imp().on_tool_changed.borrow_mut() = Some(Box::new(callback));
    }

    pub fn set_current_tool(&self, tool: Tool) {
        self.imp().current_tool.set(tool);
    }

    pub fn current_tool(&self) -> Tool {
        self.imp().current_tool.get()
    }

    pub fn set_current_slide(&self, index: usize) {
        let imp = self.imp();
        imp.current_slide_index.set(index);
        imp.selection.borrow_mut().deselect();
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

    pub fn selection(&self) -> Rc<RefCell<Selection>> {
        self.imp().selection.clone()
    }

    pub fn drawing_area(&self) -> &gtk::DrawingArea {
        &self.imp().drawing_area
    }

    pub fn document(&self) -> Option<Rc<RefCell<Document>>> {
        self.imp().document.borrow().clone()
    }
}

fn create_element_for_tool(tool: Tool, bounds: Rect) -> Option<SlideElement> {
    match tool {
        Tool::Pointer => None,
        Tool::Text => {
            let text = TextElement::new(bounds, "Text");
            Some(SlideElement::Text(text))
        }
        Tool::Shape(shape_type) => {
            let shape = ShapeElement::new(bounds, shape_type);
            Some(SlideElement::Shape(shape))
        }
        Tool::Image => None, // Image creation is handled separately via file chooser
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

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::gio;

use crate::config;

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

            let placeholder = gtk::Label::builder()
                .label("Welcome to Lumina")
                .vexpand(true)
                .hexpand(true)
                .css_classes(["title-1"])
                .build();
            content.append(&placeholder);

            window.set_content(Some(&content));
            window.present();
        }
    }

    impl GtkApplicationImpl for LuminaApplication {}
    impl AdwApplicationImpl for LuminaApplication {}
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

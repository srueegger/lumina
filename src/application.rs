use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::gio;

use crate::config;
use crate::ui::window::LuminaWindow;

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
            let window = LuminaWindow::new(&app.upcast_ref());
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

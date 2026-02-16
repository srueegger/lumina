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
        let app: Self = glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("flags", gio::ApplicationFlags::FLAGS_NONE)
            .build();

        app.setup_actions();
        app.setup_accels();
        app
    }

    fn setup_actions(&self) {
        let about_action = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| {
                app.show_about_dialog();
            })
            .build();

        let quit_action = gio::ActionEntry::builder("quit")
            .activate(|app: &Self, _, _| {
                app.quit();
            })
            .build();

        self.add_action_entries([about_action, quit_action]);
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("win.new-presentation", &["<Control>n"]);
        self.set_accels_for_action("win.open", &["<Control>o"]);
        self.set_accels_for_action("win.save", &["<Control>s"]);
        self.set_accels_for_action("win.save-as", &["<Control><Shift>s"]);
        self.set_accels_for_action("win.export-pdf", &["<Control><Shift>e"]);
        self.set_accels_for_action("app.quit", &["<Control>q"]);
    }

    fn show_about_dialog(&self) {
        let window = self.active_window();

        let dialog = adw::AboutDialog::builder()
            .application_name("Lumina")
            .application_icon(config::APP_ID)
            .version(config::VERSION)
            .developer_name("Samuel Rüegger")
            .license_type(gtk::License::Gpl20Only)
            .website("https://rueegger.me")
            .issue_url("https://github.com/srueegger/lumina/issues")
            .build();

        dialog.add_link("Email", "mailto:samuel@rueegger.me");

        if let Some(win) = window {
            dialog.present(Some(&win));
        }
    }
}

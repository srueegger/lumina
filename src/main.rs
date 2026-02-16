mod application;
mod config;
mod i18n;

use gtk::prelude::*;

fn main() -> glib::ExitCode {
    i18n::init();

    let app = application::LuminaApplication::new();
    app.run()
}

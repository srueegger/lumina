mod application;
mod config;
mod i18n;
mod model;
mod render;

use gtk::prelude::*;

fn main() -> glib::ExitCode {
    i18n::init();

    let app = application::LuminaApplication::new();
    app.run()
}

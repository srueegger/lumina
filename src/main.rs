mod application;
mod config;
mod format;
mod i18n;
mod model;
mod render;
mod ui;

use gtk::prelude::*;

fn main() -> glib::ExitCode {
    i18n::init();

    let app = application::LuminaApplication::new();
    app.run()
}

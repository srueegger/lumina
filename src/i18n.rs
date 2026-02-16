use gettextrs::{bindtextdomain, setlocale, textdomain, LocaleCategory};

use crate::config;

pub fn init() {
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain("lumina", config::LOCALEDIR).expect("Unable to bind text domain");
    textdomain("lumina").expect("Unable to set text domain");
}

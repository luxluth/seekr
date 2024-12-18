use fragile::Fragile;
use gtk::gio::Icon;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// inspired from https://github.com/aeghn/rglauncher/blob/2789af0c36f5929a448807584aaaf57685162891/crates/rglauncher-gtk/src/iconcache.rs#L12

pub static ICON_MAP: Lazy<Arc<Fragile<RwLock<HashMap<String, Icon>>>>> =
    Lazy::new(|| Arc::new(Fragile::new(RwLock::new(HashMap::new()))));

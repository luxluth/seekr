use fragile::Fragile;
use gtk::{gdk_pixbuf::Pixbuf, gio::Icon};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

// inspired from https://github.com/aeghn/rglauncher/blob/2789af0c36f5929a448807584aaaf57685162891/crates/rglauncher-gtk/src/iconcache.rs#L12
lazy_static! {
    pub static ref DIVIDE_ICON: Arc<Fragile<Icon>> =
        load_svg_resource(include_bytes!("assets/divide.svg"), (24, 24));
    pub static ref ICON_MAP: Arc<Fragile<RwLock<HashMap<String, Option<PathBuf>>>>> =
        Arc::new(Fragile::new(RwLock::new(HashMap::new())));
}

pub fn load_svg_resource(content: &[u8], size: (i32, i32)) -> Arc<Fragile<Icon>> {
    let cursor = gtk::gio::MemoryInputStream::from_bytes(&gtk::glib::Bytes::from(content));
    Arc::new(Fragile::new(Icon::from(
        Pixbuf::from_stream_at_scale(
            &cursor,
            size.0,
            size.1,
            true,
            None::<&gtk::gio::Cancellable>,
        )
        .unwrap(),
    )))
}

use fragile::Fragile;
use gtk::{gdk_pixbuf::Pixbuf, gio::Icon};
use lazy_static::lazy_static;
use std::sync::Arc;

lazy_static! {
    pub static ref DIVIDE_ICON: Arc<Fragile<Icon>> =
        load_svg_resource(include_bytes!("assets/divide.svg"), (24, 24));
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

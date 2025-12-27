use gtk::prelude::*;

pub struct InfoBox {
    pub widget: gtk::Box,
}

impl InfoBox {
    pub fn new(title: &str, body: &str) -> Self {
        let container = gtk::Box::builder()
            .css_name("infoBox")
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .spacing(10)
            .margin_top(10)
            .margin_bottom(10)
            .margin_start(10)
            .margin_end(10)
            .build();

        let title_label = gtk::Label::builder()
            .label(title)
            .selectable(true)
            .css_classes(["info_title", "title-1"])
            .halign(gtk::Align::Start)
            .build();

        let body_label = gtk::Label::builder()
            .use_markup(true)
            .selectable(true)
            .hexpand(true)
            .label(body)
            .css_classes(["info_body", "body-1"])
            .halign(gtk::Align::Start)
            .wrap(true)
            .build();

        container.append(&title_label);
        container.append(&body_label);

        Self { widget: container }
    }
}

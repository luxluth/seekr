use crate::app::AppEntry;
use crate::icons;
use crate::search::{self, SearchEvent};
use gtk::glib;
use gtk::prelude::*;
use std::sync::mpsc::Sender;

#[allow(non_snake_case)]
pub fn EntryButton(entry: AppEntry, sender: &Sender<SearchEvent>) -> gtk::Button {
    let tomanager = sender.clone();

    let entry_button = gtk::Button::builder()
        .vexpand(false)
        .hexpand(true)
        .can_focus(true)
        .focus_on_click(true)
        .sensitive(true)
        .css_name("entry")
        .name("Entry")
        .build();

    let focus_controller = gtk::EventControllerFocus::new();
    focus_controller.set_name(Some("gtk-box-focus-controller"));

    focus_controller.connect_enter(glib::clone!(
        #[weak]
        entry_button,
        move |_| {
            entry_button.add_css_class("focused");
        }
    ));

    focus_controller.connect_leave(glib::clone!(
        #[weak]
        entry_button,
        move |_| {
            entry_button.remove_css_class("focused");
        }
    ));

    entry_button.connect_clicked(glib::clone!(
        #[strong]
        entry,
        #[strong]
        tomanager,
        move |_| {
            entry.launch(None, None);
            let _ = tomanager.send(search::SearchEvent::RequestClose);
        }
    ));

    entry_button.add_controller(focus_controller);
    entry_button.set_focusable(true);

    let icon_image = gtk::Image::builder().css_name("entryIcon").build();
    icon_image.set_from_file(match icons::get_icon(&entry.icon) {
        Some(path) => Some(path),
        None => icons::get_icon("application-x-executable"),
    });

    let name = gtk::Label::builder()
        .css_name("entryName")
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .halign(gtk::Align::Start)
        .build();
    name.set_label(&entry.name);

    let desc = gtk::Label::builder()
        .css_name("entryDescription")
        .halign(gtk::Align::Start)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();
    desc.set_label(&entry.description);

    let labels = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .hexpand(true)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Start)
        .css_name("entryLabels")
        .build();

    labels.append(&name);
    labels.append(&desc);

    let content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .hexpand(true)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Start)
        .build();
    content_box.append(&icon_image);
    content_box.append(&labels);

    entry_button.set_child(Some(&content_box));

    entry_button
}

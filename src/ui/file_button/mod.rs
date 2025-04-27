use crate::conf::Config;
use crate::localsearch::FileData;
use crate::search::{self, SearchEvent};
use chrono::{DateTime, Local};
use gtk::{glib, prelude::*};
use std::sync::mpsc::Sender;

fn human_readable_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if size >= TB {
        format!("{:.2} To", size as f64 / TB as f64)
    } else if size >= GB {
        format!("{:.2} Go", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} Mo", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} Ko", size as f64 / KB as f64)
    } else {
        format!("{} o", size)
    }
}

#[allow(non_snake_case)]
pub fn FileButton(
    config: &Config,
    file_data: FileData,
    sender: &Sender<SearchEvent>,
) -> gtk::Button {
    let tomanager = sender.clone();

    let file_button = gtk::Button::builder()
        .vexpand(false)
        .hexpand(true)
        .can_focus(true)
        .focus_on_click(true)
        .sensitive(true)
        .css_name("file")
        .name("File")
        .build();

    let focus_controller = gtk::EventControllerFocus::new();
    focus_controller.set_name(Some("gtk-file-box-focus-controller"));

    focus_controller.connect_enter(glib::clone!(
        #[weak]
        file_button,
        move |_| {
            file_button.add_css_class("focused");
        }
    ));

    focus_controller.connect_leave(glib::clone!(
        #[weak]
        file_button,
        move |_| {
            file_button.remove_css_class("focused");
        }
    ));

    file_button.connect_clicked(glib::clone!(
        #[strong]
        file_data,
        #[strong]
        tomanager,
        move |_| {
            if file_data.try_open() {
                let _ = tomanager.send(search::SearchEvent::RequestClose);
            }
        }
    ));

    file_button.add_controller(focus_controller);
    file_button.set_focusable(true);

    let icon_image = gtk::Image::builder()
        .pixel_size(48)
        .css_name("fileIcon")
        .build();
    icon_image.set_from_gicon(&file_data.icon());

    let file_name = gtk::Label::builder()
        .css_name("fileName")
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .halign(gtk::Align::Start)
        .build();

    let file_details = gtk::Label::builder()
        .css_name("fileDetails")
        .halign(gtk::Align::Start)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();

    let mut details_string = String::new();
    if file_data.path.is_some() {
        let p = file_data.path.clone().unwrap();
        file_name.set_label(p.file_name().unwrap().to_str().unwrap());
        if let Ok(meta) = p.metadata() {
            let file_size = human_readable_size(meta.len());
            details_string.push_str(&file_size);

            if let Ok(modified_time) = meta.modified() {
                let datetime: DateTime<Local> = modified_time.into();
                let formatted = datetime.format(&config.general.date_format).to_string();
                details_string.push_str(" | ");
                details_string.push_str(&formatted);
            }
        }

        if file_data.mime.is_some() {
            let mime = file_data.mime.clone().unwrap();
            details_string.push_str(" | ");
            details_string.push_str(&mime);
        }
    } else {
        file_name.set_label(&file_data.uri);
    }

    file_details.set_label(&details_string);

    let labels = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .hexpand(true)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Start)
        .css_name("fileLabels")
        .build();

    labels.append(&file_name);
    labels.append(&file_details);

    let content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .hexpand(true)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Start)
        .build();

    content_box.append(&icon_image);
    content_box.append(&labels);

    file_button.set_child(Some(&content_box));

    file_button
}

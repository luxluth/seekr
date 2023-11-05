use crate::config::APP_ID;
use copypasta::{ClipboardContext, ClipboardProvider};
use dbus::arg::{PropMap, Variant};
use dbus::blocking::Connection;
use std::time::Duration;
use time::OffsetDateTime;
use relm4::gtk;
use relm4::gtk::prelude::*;

pub fn systemtime_strftime(system_time: std::time::SystemTime) -> String {
    let offset_date_time = OffsetDateTime::from(system_time);
    let date = offset_date_time.date();
    format!("{}", date).to_string()
}

/// format file size to human readable string
/// B, KB, MB, GB, TB
pub fn format_size(f: u64) -> String {
    if f < 1024 {
        return format!("{} B", f).to_string();
    }
    if f < 1024 * 1024 {
        return format!("{:.2} KB", f / 1024).to_string();
    }
    if f < 1024 * 1024 * 1024 {
        return format!("{:.2} MB", f / 1024 / 1024).to_string();
    }
    if f < 1024 * 1024 * 1024 * 1024 {
        return format!("{:.2} GB", f / 1024 / 1024 / 1024).to_string();
    }
    format!("{:.2} TB", f / 1024 / 1024 / 1024 / 1024).to_string()
}

pub fn copy_to_clipboard(s: &str) {
    let mut ctx = ClipboardContext::new().unwrap();
    ctx.set_contents(s.to_owned()).unwrap();
}

pub fn app_is_running() -> bool {
    // chech the dbus for the APP_ID
    let c = Connection::new_session();
    match c {
        Ok(conn) => {
            let proxy = conn.with_proxy("org.freedesktop.DBus", "/", Duration::from_millis(5000));
            let (names,): (Vec<String>,) = proxy
                .method_call("org.freedesktop.DBus", "ListNames", ())
                .unwrap();
            for name in names {
                if name.contains(APP_ID) {
                    return true;
                }
            }

            return false;
        }

        Err(_) => {
            println!("Could not connect to dbus.");
            return false;
        }
    }
}

pub fn send_represent_event() {
    let c = Connection::new_session();
    match c {
        Ok(conn) => {
            // try to send a signal to the app
            let proxy = conn.with_proxy(
                APP_ID,
                "/dev/luxluth/fsearch/window/1",
                Duration::from_millis(5000),
            );
            // https://github.com/diwic/dbus-rs/blob/master/dbus/examples/argument_guide.md
            // https://wiki.gnome.org/Projects/GLib/GApplication/DBusAPI#Methods-2
            // argument (sava{sv})
            // The action is named by the first parameter (s).
            // If the action activation requires a parameter then this parameter must be given in the second parameter (av). If there is no parameter to be specified, the array must be empty.
            // The final parameter (a{sv}) is a list of "platform data".
            let _: () = proxy
                .method_call::<(), (&str, Vec<Variant<String>>, PropMap), &str, &str>(
                    "org.gtk.Actions",
                    "Activate",
                    ("represent", vec![], PropMap::new()),
                )
                .unwrap();
        }

        Err(_) => {
            println!("Could not connect to dbus.");
        }
    }
}


pub fn get_section_title(label: String) -> gtk::Label {
    gtk::Label::builder()
        .name("SectionTitle")
        .css_name("SectionTitle")
        .hexpand(true)
        .halign(gtk::Align::Start)
        .label(label)
        .build()
}

pub fn wrap_section(bx: gtk::Box) -> gtk::Box {
    let section = gtk::Box::builder()
        .name("Section")
        .css_name("Section")
        .focusable(false)
        .orientation(gtk::Orientation::Vertical)
        .build();

    section.append(&bx);
    section
}

/// replace placeholders in the command string
/// every placeholder starts with a % sign and ends with a space
pub fn replace_placeholders(cmd: String) -> String {
    let cmd = cmd.split_whitespace().collect::<Vec<&str>>();
    let mut new_cmd = Vec::new();
    for c in cmd {
        if !c.starts_with("%") {
            new_cmd.push(c);
        }
    }
    new_cmd.join(" ")
}

pub fn exec_a_separate_process(cmd: &str) -> bool {
    let mut cmd = cmd.split_whitespace();
    match std::process::Command::new(cmd.next().unwrap()).args(cmd).spawn() {
        Ok(_) => {
            return true;
        }
        Err(_) => {
            return false;
        }
    }    
}

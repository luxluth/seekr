use gdk::Display;
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Box, Entry, Label};
use std::path::PathBuf;
use std::env;

fn get_file_content(path: &PathBuf) -> String {
    let content = std::fs::read_to_string(path);
    match content {
        Ok(c) => {
            return c;
        },
        Err(_) => {
            return String::new();
        },
    }
}

const APP_ID: &str = "dev.luxluth.fsearch";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}
#[derive(Clone, Debug)]
pub struct EntryText {
    value: String,
    entry: Entry
}

impl EntryText {
    pub fn new() -> Self {
        let entry_input = Entry::builder()
            .name("EntryInput")
            .css_name("EntryInput")
            .placeholder_text("Start typing...")
            .secondary_icon_tooltip_text("Search")
            .secondary_icon_name("system-search-symbolic") // need to add icon
            .enable_emoji_completion(true)
            .activates_default(true)
            // take the full width of the box 
            .hexpand(true)
            .build();

        EntryText {
            value: String::new(),
            entry: entry_input
        }
    }

    pub fn build_entry_box(&mut self) -> Box {
        let entry_box = Box::builder()
            .name("EntryBox")
            .css_name("EntryBox")
            .orientation(gtk::Orientation::Vertical)
            // take the full width of the window 
            .hexpand(true)
            .focusable(false)
            .build();
        entry_box.append(&self.entry);
        
        self.entry.connect_changed(move |entry| {
            println!("{}", entry.text().to_string());
        });

        return entry_box;
    }

    pub fn set_value(&mut self, new_value: String) {
        self.value = new_value;
    }
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    match env::var("XDG_CONFIG_HOME") {
        Ok(v) => {
            let css_path = PathBuf::from(v).join("fsearch").join("style.css");
            let css_content = get_file_content(&css_path);
            provider.load_from_string(&css_content);


        },
        Err(_) => {
            match env::var("HOME") {
                Ok(v) => {
                    let css_path = PathBuf::from(v).join(".config").join("fsearch").join("style.css");
                    let css_content = get_file_content(&css_path);
                    provider.load_from_string(&css_content);
                },
                Err(_) => {
                    println!("Could not find config file.");
                },
            }
        },
    }

    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
    );
}

fn build_ui(app: &Application) {
    load_css();
    let mut entry = EntryText::new();
    let entry_box = entry.build_entry_box();
    let tip_label = Label::builder()
        .name("Tip")
        .css_name("Tip")
        .label("use @command to run a specific command")
        .hexpand(true)
        .halign(gtk::Align::Start)
        .build();

    let dynamic_box = Box::builder()
        .name("DynamicBox")
        .css_name("DynamicBox")
        .orientation(gtk::Orientation::Vertical)
        .hexpand(true)
        .build();

    entry_box.append(&tip_label);
    entry_box.append(&dynamic_box);

    let window = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .css_name("Application")
        .css_classes(vec!["application".to_string()])
        .resizable(false)
        .default_width(600)
        .default_height(50)
        .child(&entry_box)
        .title("fsearch")
        .build();

    window.present();
}

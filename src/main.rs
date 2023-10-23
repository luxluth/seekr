use gtk::prelude::*;
use relm4::prelude::*;
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

struct App {
    input: String,
}

#[derive(Debug)]
enum Msg {
    SetInput(String),
}

fn load_css() {
    match env::var("XDG_CONFIG_HOME") {
        Ok(v) => {
            let css_path = PathBuf::from(v).join("fsearch").join("style.css");
            let css_content = get_file_content(&css_path);
            relm4::set_global_css(css_content.as_str());

        },
        Err(_) => {
            match env::var("HOME") {
                Ok(v) => {
                    let css_path = PathBuf::from(v).join(".config").join("fsearch").join("style.css");
                    let css_content = get_file_content(&css_path);
                    relm4::set_global_css(css_content.as_str());
                },
                Err(_) => {
                    println!("Could not find config file.");
                },
            }
        },
    }

}

#[relm4::component]
impl SimpleComponent for App {
    type Init = String;
    type Input = Msg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("fsearch"),
            set_default_size: (600, 50),
            set_decorated: false,
            set_resizable: false,
            set_css_classes: &["application"],

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true, 
                set_focusable: false,
                set_widget_name: "EntryBox",

                #[name="entry"]
                gtk::Entry {
                    set_activates_default: true,
                    set_hexpand: true,
                    set_widget_name: "EntryInput",
                    set_placeholder_text: Some("Start typing..."),
                    // set_primary_icon_name: Some("loupe"),
                    set_text: &model.input,
                    connect_changed[sender] => move |entry| {
                        sender.input(Msg::SetInput(entry.text().to_string()));
                    },
                },

                gtk::Label {
                    set_widget_name: "Tip",
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                    set_label: "use @command to run a specific action."
                }
            }
        }
    }

    // Initialize the component.
    fn init(
        input: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App { input };

        // Insert the code generation of the view! macro here
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::SetInput(input) => {
                self.input = input;
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("relm4.example.simple");
    relm4_icons::initialize_icons();
    load_css();
    app.run::<App>(String::new());
}


mod config;
mod exec;
mod utils;
mod args;

use config::APP_ID;
use exec::Action;

use args::{FsearchArgs, Entity};
use clap::Parser;

use crate::glib::clone;
use crate::gtk::glib;

use gtk::prelude::*;
use relm4::gtk::gio::SimpleAction;
use relm4::{prelude::*, RelmIterChildrenExt};
use std::env;
use std::{path::PathBuf, process::exit};

fn get_file_content(path: &PathBuf) -> String {
    let content = std::fs::read_to_string(path);
    match content {
        Ok(c) => {
            return c;
        }
        Err(_) => {
            return String::new();
        }
    }
}

struct App {
    input: String,
    dynamic_box: Option<gtk::Box>,
    action: Option<Action>,
}

#[derive(Debug)]
enum Msg {
    SetInput(String),
    Enter,
}

fn load_css() {
    match env::var("XDG_CONFIG_HOME") {
        Ok(v) => {
            let css_path = PathBuf::from(v).join("fsearch").join("style.css");
            let css_content = get_file_content(&css_path);
            relm4::set_global_css(css_content.as_str());
        }
        Err(_) => match env::var("HOME") {
            Ok(v) => {
                let css_path = PathBuf::from(v)
                    .join(".config")
                    .join("fsearch")
                    .join("style.css");
                let css_content = get_file_content(&css_path);
                relm4::set_global_css(css_content.as_str());
            }
            Err(_) => {
                println!("Could not find config file.");
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
            set_default_size: (600, -1),
            set_decorated: false,
            set_resizable: false,
            set_css_classes: &["application"],
            set_hide_on_close: true,

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
                    set_enable_emoji_completion: true,
                    set_text: &model.input,
                    connect_changed[sender] => move |entry| {
                        sender.input(Msg::SetInput(entry.text().to_string()));
                    },

                    connect_activate[sender] => move |_| {
                        sender.input(Msg::Enter);
                    },
                },

                gtk::Label {
                    set_widget_name: "Tip",
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                    set_label: "use @command to run a specific action."
                },

                #[name="dynamic_box"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_focusable: false,
                    set_widget_name: "DynamicBox",
                },
            }
        }
    }

    // Initialize the component.
    fn init(
        input: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {
            input,
            dynamic_box: None,
            action: None,
        };
        let widgets = view_output!();

        let represent_action = SimpleAction::new("represent", None);
        represent_action.connect_activate(clone!(@weak root => move |_, _| {
            load_css();
            root.present();
        }));

        root.add_action(&represent_action);

        let mut component_parts = ComponentParts { model, widgets };

        let dynamic_box = component_parts.widgets.dynamic_box.clone();
        component_parts.model.dynamic_box = Some(dynamic_box.clone());

        component_parts
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::SetInput(input) => {
                self.input = input;
                let res = exec::exec(self.input.clone());
                if res.components.len() == 0 && res.action.is_none() {
                    self.dynamic_box
                        .as_ref()
                        .unwrap()
                        .iter_children()
                        .for_each(|child| {
                            self.dynamic_box.as_ref().unwrap().remove(&child);
                        });
                } else {
                    self.dynamic_box
                        .as_ref()
                        .unwrap()
                        .iter_children()
                        .for_each(|child| {
                            self.dynamic_box.as_ref().unwrap().remove(&child);
                        });
                    for component in res.components {
                        self.dynamic_box.as_ref().unwrap().append(&component);
                    }

                    match res.action {
                        Some(a) => {
                            self.action = Some(a);
                        }
                        None => {
                            self.action = None;
                        }
                    }
                }
            }

            Msg::Enter => match &self.action {
                Some(a) => match a {
                    Action::Exit => {
                        relm4::main_application().quit();
                    }
                    Action::Open(something) => {
                        println!("Open {:?}!", something);
                        match open::that(something.trim_start()) {
                            Ok(_) => {
                                relm4::main_application().quit();
                            }
                            Err(_) => {}
                        };
                    }
                    Action::Copy(something) => {
                        println!("Copy {:?}!", something);
                        utils::copy_to_clipboard(something);
                    }
                },
                None => return,
            },
        }
    }
}

fn main() {
    let args = FsearchArgs::parse();
    match args.entity {
        Some(Entity::Deamon) => {
            println!("Deamon");
            return;
        }
        Some(Entity::Status) => {
            println!("Deamon Status");
            return;
        }
        Some(Entity::Stop) => {
            println!("Deamon Stop");
            return;
        }
        Some(Entity::Config(config)) => {
            match config {
                args::ConfigArgs { config, css } => {
                    let at_least_one = config.is_some() || css.is_some();
                    if config.is_some() {
                        println!("Config {:?}", config.unwrap());
                    }
                    if css.is_some() {
                        println!("Css {:?}", css.unwrap());
                    }
                    if !at_least_one {
                        println!("No config file or css file specified.");
                        exit(1);
                    }
                }
            }
        }
        None => {}
    }
    if utils::app_is_running() {
        utils::send_represent_event();
        return;
    }
    let app = RelmApp::new(APP_ID);
    relm4_icons::initialize_icons();
    load_css();
    app.run::<App>(String::new());
}

mod cli;
mod completion;
mod config;
mod exec;
mod plugin;
mod utils;

use config::APP_ID;
use fsearch_core::PluginActionType as Action;

use clap::Parser;
use cli::{Command, FsearchArgs};

use crate::glib::clone;
use crate::gtk::glib;

use gtk::prelude::*;
use relm4::gtk::gio::SimpleAction;
use relm4::{prelude::*, RelmIterChildrenExt};

use std::process::exit;

use fsearch_core::{get_cfg, get_css, get_plugins, Config, PluginConfig};

struct App {
    input: String,
    config: Option<Config>,
    plugins: Vec<PluginConfig>,
    dynamic_box: Option<gtk::ListBox>,
    dynamic_icon: Option<gtk::Image>,
    action: Option<Action>,
}

#[derive(Debug)]
enum Msg {
    SetInput(String),
    Enter,
}

fn load_css() {
    relm4::set_global_css(get_css().as_str());
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

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_focusable: false,
                    set_widget_name: "EntryAndIconBox",

                    #[name="entry"]
                    gtk::Entry {
                        set_activates_default: true,
                        set_hexpand: true,
                        set_widget_name: "EntryInput",
                        set_placeholder_text: Some("Search"),
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

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_hexpand: false,
                        set_focusable: false,
                        set_widget_name: "EntryIconBox",

                        #[name="dynamic_icon"]
                        gtk::Image {
                            set_widget_name: "EntryIcon",
                            set_halign: gtk::Align::Center,
                        },
                    },
                },


                #[name="tip"]
                gtk::Label {
                    set_widget_name: "Tip",
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                    set_label: "use @command to run a specific action."
                },

                #[name="dynamic_box"]
                gtk::ListBox {
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
            config: get_cfg(),
            plugins: get_plugins(),
            dynamic_box: None,
            dynamic_icon: None,
            action: None,
        };

        let widgets = view_output!();

        let represent_action = SimpleAction::new("represent", None);
        represent_action.connect_activate(clone!(@weak root => move |_, _| {
            load_css();
            root.present();
        }));

        root.add_action(&represent_action);

        if let Some(cfg) = &model.config {
            println!("{:?}", cfg);
            if let Some(look) = &cfg.look {
                if let Some(disable_tip) = &look.disable_tip {
                    if *disable_tip {
                        widgets.tip.hide();
                    }
                }

                if let Some(initial_width) = &look.initial_width {
                    if *initial_width > 100 {
                        root.set_default_size((*initial_width) as i32, -1);
                    }
                }

                if let Some(input_placeholder) = &look.input_placeholder {
                    widgets.entry.set_placeholder_text(Some(input_placeholder));
                }
            }
        }

        let mut component_parts = ComponentParts { model, widgets };

        let dynamic_box = component_parts.widgets.dynamic_box.clone();
        component_parts.model.dynamic_box = Some(dynamic_box.clone());

        let dynamic_icon = component_parts.widgets.dynamic_icon.clone();
        component_parts.model.dynamic_icon = Some(dynamic_icon.clone());

        component_parts
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::SetInput(input) => {
                self.input = input;
                if self.input.len() == 0 {
                    self.dynamic_icon
                        .as_ref()
                        .unwrap()
                        .set_from_icon_name(None);
                }
                let res = exec::exec(self.input.clone(), &self.plugins);
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

                    match res.icon {
                        Some(icon_path) => {
                            self.dynamic_icon
                                .as_ref()
                                .unwrap()
                                .set_from_file(Some(icon_path.as_str()));
                        },
                        None => {
                            self.dynamic_icon
                                .as_ref()
                                .unwrap()
                                .set_from_icon_name(None);
                        },
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
                    Action::Launch(_something) => todo!(),
                    Action::RunCmd(_cmd) => todo!(),
                    Action::RunScript(_script) => todo!(),
                },
                None => return,
            },
        }
    }
}

fn main() {
    let matches = FsearchArgs::parse();
    match matches.command {
        Some(Command::Config(config)) => match config {
            cli::ConfigArgs { config, css } => {
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
        },
        Some(Command::Completion(arg)) => {
            let shell = arg.shell;
            completion::generate_completion(shell);
            return;
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

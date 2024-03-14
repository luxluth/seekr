use crate::plugin::execute_plugin;
use crate::utils::{get_section_title, wrap_section};
use exmex::ExError;
use fsearch_core::PluginConfig;
use relm4::gtk;
use relm4::gtk::prelude::*;

const INNER_COMMANDS: [&str; 4] = [
    "exit", // exit the program
    "open", // open a file @open <file>
    "exp",  // evaluate a mathematical expression @exp <expression>
    "help", // show help
];

const HELP: &str = r#"@exit: exit the program
@open <file>: open a file
@exp <expression>: evaluate a mathematical expression
@help: show help
"#;

type Action = fsearch_core::PluginActionType;

#[derive(Debug)]
pub struct Result {
    pub action: Option<Action>,
    pub data: String,
    pub components: Vec<gtk::Box>,
    pub icon: Option<String>, // icon path
}

type Plug = Vec<PluginConfig>;

pub fn exec(input: String, plugins: &Plug) -> Result {
    if input.is_empty() || input.len() > 1000 {
        return Result {
            action: None,
            data: input,
            components: Vec::new(),
            icon: None,
        };
    }

    let input_type = detect_input_type(&input);
    match input_type {
        InputType::Search => search(input, plugins),
        InputType::Mathematical => mathematical(input),
        InputType::Url => url(input),
        InputType::Command(cmd) => command(cmd.as_str(), input, plugins),
    }
}

fn search(input: String, plugins: &Plug) -> Result {
    let mut components: Vec<gtk::Box> = Vec::new();
    let search_prefix = gtk::Label::builder()
        .name("SearchPrefix")
        .css_name("SearchPrefix")
        .focusable(false)
        .halign(gtk::Align::Start)
        .label("Searching for ".to_string())
        .build();

    let search_query = gtk::Label::builder()
        .name("SearchQuery")
        .css_name("SearchQuery")
        .wrap(true)
        .focusable(false)
        .hexpand(true)
        .halign(gtk::Align::Start)
        .label(format!("«{}»", input))
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();

    let box_content = gtk::Box::builder()
        .name("Search")
        .focusable(false)
        .sensitive(false)
        .css_name("Search")
        .orientation(gtk::Orientation::Vertical)
        .build();
    box_content.append(&search_prefix);
    box_content.append(&search_query);
    components.push(box_content);

    let mut icon = None;
    let mut action = None;

    for plugin in plugins {
        if let Some(roaq) = plugin.run_on_any_query {
            if roaq {
                let (comp, plug_action, set_icon) = execute_plugin(plugin, input.clone());
                if let Some(comp) = comp {
                    components.push(comp);
                }
                if plug_action.is_some() {
                    action = plug_action;
                }
                if set_icon.is_some() {
                    icon = set_icon;
                }
            }
        }
    }

    Result {
        action,
        data: input,
        components,
        icon,
    }
}

fn mathematical(input: String) -> Result {
    if input.is_empty() {
        return Result {
            action: None,
            data: input,
            components: Vec::new(),
            icon: None,
        };
    }
    let result = exmex::eval_str::<f64>(&input);
    match result {
        Ok(result) => mathematical_result(input, result),
        Err(e) => mathematical_error(input, e),
    }
}

fn mathematical_result(input: String, result: f64) -> Result {
    let mut components: Vec<gtk::Box> = Vec::new();
    let title = get_section_title(format!(
        "({}) evaluation",
        input.clone().trim_start().trim_end()
    ));
    let content = gtk::Label::builder()
        .name("Content")
        .css_name("Content")
        .wrap(true)
        .css_classes(vec!["mathematical"])
        .hexpand(true)
        .halign(gtk::Align::Start)
        .label(result.to_string())
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .build();

    let box_content = gtk::Box::builder()
        .name("BoxContent")
        .css_name("BoxContent")
        .orientation(gtk::Orientation::Vertical)
        .build();
    box_content.append(&title);
    box_content.append(&content);
    components.push(wrap_section(box_content));

    Result {
        action: Some(Action::Copy(result.to_string())),
        data: input,
        components,
        icon: None,
    }
}

fn mathematical_error(input: String, err: ExError) -> Result {
    let mut components: Vec<gtk::Box> = Vec::new();
    let title = get_section_title(format!(
        "({}) evaluation",
        input.clone().trim_start().trim_end()
    ));
    let content = gtk::Label::builder()
        .name("Content")
        .css_name("Content")
        .wrap(true)
        .css_classes(vec!["error"])
        .hexpand(true)
        .wrap(true)
        .halign(gtk::Align::Start)
        .label(format!("{}", err))
        .build();

    let box_content = gtk::Box::builder()
        .name("BoxContent")
        .css_name("BoxContent")
        .orientation(gtk::Orientation::Vertical)
        .build();
    box_content.append(&title);
    box_content.append(&content);
    components.push(wrap_section(box_content));

    Result {
        action: None,
        data: input,
        components,
        icon: None,
    }
}

fn url(input: String) -> Result {
    let mut components: Vec<gtk::Box> = Vec::new();
    let title = get_section_title("Url".to_string());
    let content = gtk::Label::builder()
        .name("Content")
        .css_name("Content")
        .wrap(true)
        .css_classes(vec!["url"])
        .hexpand(true)
        .halign(gtk::Align::Start)
        .label(format!("open {}", input))
        .build();

    let box_content = gtk::Box::builder()
        .name("BoxContent")
        .css_name("BoxContent")
        .orientation(gtk::Orientation::Vertical)
        .build();

    box_content.append(&title);
    box_content.append(&content);

    components.push(wrap_section(box_content));

    Result {
        action: Some(Action::Open(input.clone())),
        data: input,
        components,
        icon: None,
    }
}

fn command(cmd: &str, input: String, plugins: &Plug) -> Result {
    match cmd {
        "exit" => Result {
            action: Some(Action::Exit),
            data: input,
            components: Vec::new(),
            icon: None,
        },

        "open" => {
            let file = input[5..].to_string();
            let mut components: Vec<gtk::Box> = Vec::new();
            let title = get_section_title("Open".to_string());
            let content = gtk::Label::builder()
                .name("Content")
                .css_name("Content")
                .wrap(true)
                .css_classes(vec!["open"])
                .hexpand(true)
                .halign(gtk::Align::Start)
                .label(format!("open {}", file))
                .build();

            let box_content = gtk::Box::builder()
                .name("BoxContent")
                .css_name("BoxContent")
                .orientation(gtk::Orientation::Vertical)
                .build();

            box_content.append(&title);
            box_content.append(&content);

            components.push(wrap_section(box_content));

            Result {
                action: Some(Action::Open(file.clone())),
                data: input,
                components,
                icon: None,
            }
        }

        "exp" => mathematical(input[4..].to_string()),

        "help" => {
            let mut components: Vec<gtk::Box> = Vec::new();
            let title = get_section_title("Help".to_string());
            let content = gtk::Label::builder()
                .name("Content")
                .css_name("Content")
                .wrap(true)
                .css_classes(vec!["help"])
                .hexpand(true)
                .halign(gtk::Align::Start)
                .label(HELP.to_string())
                .build();

            let plugin_title = get_section_title("Plugins".to_string());
            let mut plugin_content = String::new();
            for plugin in plugins {
                plugin_content.push_str(
                    format!("@{} <query>: {}\n", plugin.name, plugin.description).as_str(),
                );
            }

            let plugin_content = gtk::Label::builder()
                .name("Content")
                .css_name("Content")
                .wrap(true)
                .css_classes(vec!["help"])
                .hexpand(true)
                .halign(gtk::Align::Start)
                .label(plugin_content)
                .build();

            let help_box_content = gtk::Box::builder()
                .name("BoxContent")
                .css_name("BoxContent")
                .orientation(gtk::Orientation::Vertical)
                .build();

            let plugin_box_content = gtk::Box::builder()
                .name("BoxContent")
                .css_name("BoxContent")
                .orientation(gtk::Orientation::Vertical)
                .build();

            help_box_content.append(&title);
            help_box_content.append(&content);

            plugin_box_content.append(&plugin_title);
            plugin_box_content.append(&plugin_content);

            components.push(wrap_section(help_box_content));
            components.push(wrap_section(plugin_box_content));

            Result {
                action: None,
                data: input,
                components,
                icon: None,
            }
        }

        _ => Result {
            action: None,
            data: input,
            components: Vec::new(),
            icon: None,
        },
    }
}

#[derive(Debug)]
enum InputType {
    Search,
    Mathematical,
    Url,
    Command(String),
}

fn detect_input_type(input: &str) -> InputType {
    if input.starts_with("http://") || input.starts_with("https://") {
        return InputType::Url;
    }
    if input.starts_with('@') {
        let (_, command) = input.split_once('@').unwrap();
        let mut split = command.split(' ');
        let cmd = split.next().unwrap();
        if INNER_COMMANDS.contains(&cmd) {
            return InputType::Command(cmd.to_string());
        }
        return InputType::Search;
    }
    match exmex::eval_str::<f64>(input) {
        Ok(_) => InputType::Mathematical,
        Err(_) => InputType::Search,
    }
}

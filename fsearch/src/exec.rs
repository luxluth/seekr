use crate::plugin::execute_plugin;
use crate::utils;
use crate::utils::{get_section_title, wrap_section};
use exmex::ExError;
use relm4::gtk;
use relm4::gtk::prelude::*;

// path things
use std::fs;
use std::path::PathBuf;

use fsearch_core::PluginConfig;

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
        InputType::File(file) => f(file),
    }
}

fn search(input: String, plugins: &Plug) -> Result {
    let mut components: Vec<gtk::Box> = Vec::new();
    let search_prefix = gtk::Label::builder()
        .name("SearchPrefix")
        .css_name("SearchPrefix")
        .focusable(false)
        .halign(gtk::Align::Start)
        .label(format!("Searching for "))
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
        match plugin.run_on_any_query {
            Some(run_on_any_query) => {
                if run_on_any_query {
                    let (comp, plug_action, set_icon) = execute_plugin(plugin, input.clone());
                    if comp.is_some() {
                        components.push(comp.unwrap());
                    }
                    if plug_action.is_some() {
                        action = plug_action;
                    }
                    if set_icon.is_some() {
                        icon = set_icon;
                    }
                }
            }
            None => {}
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
        Ok(result) => {
            return mathematical_result(input, result);
        }
        Err(e) => {
            return mathematical_error(input, e);
        }
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
                .label(format!("{}", HELP))
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

fn expand_tilde(path: String) -> String {
    if path.starts_with("~") {
        match std::env::var("HOME") {
            Ok(home) => {
                return path.replace("~", home.as_str());
            }
            Err(_) => {
                return path;
            }
        }
    }
    path
}

fn f(file: String) -> Result {
    // find in the file system
    let mut components: Vec<gtk::Box> = Vec::new();
    let title = get_section_title("File".to_string());
    let file = expand_tilde(file);
    // check if the file exists
    let path = PathBuf::from(file.clone());
    if !path.exists() {
        let content = gtk::Label::builder()
            .name("Content")
            .css_name("Content")
            .wrap(true)
            .css_classes(vec!["file"])
            .hexpand(true)
            .halign(gtk::Align::Start)
            .label(format!("{} does not exist.", file))
            .css_classes(vec!["error"])
            .build();

        let box_content = gtk::Box::builder()
            .name("BoxContent")
            .css_name("BoxContent")
            .orientation(gtk::Orientation::Vertical)
            .build();

        box_content.append(&title);
        box_content.append(&content);

        components.push(wrap_section(box_content));

        return Result {
            action: None,
            data: file,
            components,
            icon: None,
        };
    }

    // check if the file is a directory
    let metadata = fs::metadata(path.clone());
    match metadata {
        Ok(meta) => {
            // is directory -> directory_path size last_modified
            if meta.is_dir() {
                let directory_name = gtk::Label::builder()
                    .name("DirectoryName")
                    .css_name("DirectoryName")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(format!("{}", path.display().to_string()))
                    .build();

                let directory_size = gtk::Label::builder()
                    .name("DirectorySize")
                    .css_name("DirectorySize")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(utils::format_size(meta.len()))
                    .build();

                let directory_last_modified = gtk::Label::builder()
                    .name("DirectoryLastModified")
                    .css_name("DirectoryLastModified")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(utils::systemtime_strftime(meta.modified().unwrap()))
                    .build();

                let box_dir = gtk::Box::builder()
                    .name("BoxContentElement")
                    .css_name("BoxContentElement")
                    .css_classes(vec!["fileContainer"])
                    .orientation(gtk::Orientation::Horizontal)
                    .build();

                box_dir.append(&directory_name);
                box_dir.append(&directory_size);
                box_dir.append(&directory_last_modified);

                let box_content = gtk::Box::builder()
                    .name("BoxContent")
                    .css_name("BoxContent")
                    .orientation(gtk::Orientation::Vertical)
                    .build();

                box_content.append(&title);
                box_content.append(&box_dir);

                components.push(wrap_section(box_content));

                return Result {
                    action: None,
                    data: file,
                    components,
                    icon: None,
                };
            }
            // is file -> file_name size file_type last_modified
            else if meta.is_file() {
                let file_name = gtk::Label::builder()
                    .name("FileName")
                    .css_name("FileName")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(format!("{}", path.file_name().unwrap().to_str().unwrap()))
                    .build();

                let file_size = gtk::Label::builder()
                    .name("FileSize")
                    .css_name("FileSize")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(utils::format_size(meta.len()))
                    .build();

                let file_type = gtk::Label::builder()
                    .name("FileType")
                    .css_name("FileType")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(format!("{:?}", meta.file_type()))
                    .build();

                let last_modified = gtk::Label::builder()
                    .name("LastModified")
                    .css_name("LastModified")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(utils::systemtime_strftime(meta.modified().unwrap()))
                    .build();

                let box_file = gtk::Box::builder()
                    .name("BoxContentElement")
                    .css_name("BoxContentElement")
                    .css_classes(vec!["fileContainer"])
                    .orientation(gtk::Orientation::Horizontal)
                    .build();

                box_file.append(&file_name);
                box_file.append(&file_size);
                box_file.append(&file_type);
                box_file.append(&last_modified);

                let box_content = gtk::Box::builder()
                    .name("BoxContent")
                    .css_name("BoxContent")
                    .orientation(gtk::Orientation::Vertical)
                    .build();

                box_content.append(&title);
                box_content.append(&box_file);

                components.push(wrap_section(box_content));

                return Result {
                    action: None,
                    data: file,
                    components,
                    icon: None,
                };
            }
            // is symlink -> symlink_name size file_type last_modified
            else if meta.is_symlink() {
                let symlink_name = gtk::Label::builder()
                    .name("SymlinkName")
                    .css_name("SymlinkName")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(format!("{}", path.file_name().unwrap().to_str().unwrap()))
                    .build();

                let symlink_size = gtk::Label::builder()
                    .name("SymlinkSize")
                    .css_name("SymlinkSize")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(utils::format_size(meta.len()))
                    .build();

                let symlink_type = gtk::Label::builder()
                    .name("SymlinkType")
                    .css_name("SymlinkType")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(format!("{:?}", meta.file_type()))
                    .build();

                let last_modified = gtk::Label::builder()
                    .name("LastModified")
                    .css_name("LastModified")
                    .css_classes(vec!["fileElement"])
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .label(utils::systemtime_strftime(meta.modified().unwrap()))
                    .build();

                let box_symlink = gtk::Box::builder()
                    .name("BoxContentElement")
                    .css_name("BoxContentElement")
                    .css_classes(vec!["fileContainer"])
                    .orientation(gtk::Orientation::Horizontal)
                    .build();

                box_symlink.append(&symlink_name);
                box_symlink.append(&symlink_size);
                box_symlink.append(&symlink_type);
                box_symlink.append(&last_modified);

                let box_content = gtk::Box::builder()
                    .name("BoxContent")
                    .css_name("BoxContent")
                    .orientation(gtk::Orientation::Vertical)
                    .build();

                box_content.append(&title);
                box_content.append(&box_symlink);

                components.push(wrap_section(box_content));

                return Result {
                    action: None,
                    data: file,
                    components,
                    icon: None,
                };
            } else {
                return Result {
                    action: None,
                    data: file,
                    components,
                    icon: None,
                };
            }
        }
        Err(_) => {
            let content = gtk::Label::builder()
                .name("Content")
                .css_name("Content")
                .wrap(true)
                .css_classes(vec!["file"])
                .hexpand(true)
                .halign(gtk::Align::Start)
                .label(format!("Cannot get metadata of {}", file))
                .build();
            let box_content = gtk::Box::builder()
                .name("BoxContent")
                .css_name("BoxContent")
                .orientation(gtk::Orientation::Vertical)
                .build();

            box_content.append(&title);
            box_content.append(&content);

            components.push(wrap_section(box_content));

            return Result {
                action: None,
                data: file,
                components,
                icon: None,
            };
        }
    }
}

#[derive(Debug)]
enum InputType {
    Search,
    Mathematical,
    Url,
    Command(String),
    File(String),
}

fn detect_input_type(input: &str) -> InputType {
    if input.starts_with("file://") {
        return InputType::File(input[7..].to_string());
    }
    if input.starts_with("http://") || input.starts_with("https://") {
        return InputType::Url;
    }
    if input.starts_with("@") {
        let command = input[1..].to_string();
        let mut split = command.split(" ");
        let cmd = split.next().unwrap();
        if INNER_COMMANDS.contains(&cmd) {
            return InputType::Command(cmd.to_string());
        }
        return InputType::Search;
    }
    match exmex::eval_str::<f64>(input) {
        Ok(_) => {
            return InputType::Mathematical;
        }
        Err(_) => {
            return InputType::Search;
        }
    }
}

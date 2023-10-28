use serde::{Deserialize, Serialize};
use serde_json;
use toml;
use std::env;
use std::path::PathBuf;

/// Get a text file content as a string
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

/// Get the CSS file content
pub fn get_css() -> String {
    match env::var("XDG_CONFIG_HOME") {
        Ok(v) => {
            let css_path = PathBuf::from(v).join("fsearch").join("style.css");
            get_file_content(&css_path)
        }
        Err(_) => match env::var("HOME") {
            Ok(v) => {
                let css_path = PathBuf::from(v)
                    .join(".config")
                    .join("fsearch")
                    .join("style.css");
                get_file_content(&css_path)
            }
            Err(_) => {
                println!("Could not find config file.");
                String::new()
            }
        },
    }
}

/// Get the config .toml file content
pub fn get_cfg() -> Option<Config> {
    match env::var("XDG_CONFIG_HOME") {
        Ok(v) => {
            let cfg_path = PathBuf::from(v).join("fsearch").join("config.toml");
            let cfg_content = get_file_content(&cfg_path);
            let cfg: Config = toml::from_str(&cfg_content).unwrap();
            Some(cfg)
        }
        Err(_) => match env::var("HOME") {
            Ok(v) => {
                let cfg_path = PathBuf::from(v)
                    .join(".config")
                    .join("fsearch")
                    .join("config.toml");
                let cfg_content = get_file_content(&cfg_path);
                let cfg: Config = toml::from_str(&cfg_content).unwrap();
                Some(cfg)
            }
            Err(_) => {
                println!("Could not find config file.");
                None
            }
        },
    }
}

/// Check if the plugins directory exists and if it has plugins in it
/// Returns a tuple with a bool indicating if the plugins directory exists
fn has_plugins() -> (bool, Option<PathBuf>) {
    match env::var("XDG_CONFIG_HOME") {
        Ok(v) => {
            let plugins_path = PathBuf::from(v).join("fsearch").join("plugins");
            let plugins_exist = plugins_path.exists();
            (plugins_exist, Some(plugins_path))
        }
        Err(_) => match env::var("HOME") {
            Ok(v) => {
                let plugins_path = PathBuf::from(v)
                    .join(".config")
                    .join("fsearch")
                    .join("plugins");
                let plugins_exist = plugins_path.exists();
                (plugins_exist, Some(plugins_path))
            }
            Err(_) => {
                println!("Could not find config file.");
                (false, None)
            }
        },
    } 
}

/// Get the plugins config .toml file content
pub fn get_plugins() -> Vec<PluginConfig> {
    let mut plugins: Vec<PluginConfig> = Vec::new();
    let (plugins_exist, plugins_path) = has_plugins();

    if plugins_exist {
        let plugins_path = plugins_path.unwrap();
        let plugins_dir = std::fs::read_dir(plugins_path);
        match plugins_dir {
            Ok(dir) => {
                for entry in dir {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    if file_name.ends_with(".toml") {
                        let plugin_content = get_file_content(&path);
                        let plugin: PluginConfig = toml::from_str(&plugin_content).unwrap();
                        plugins.push(plugin);
                    }
                }
                plugins
            }
            Err(_) => {
                println!("Could not find plugins directory.");
                plugins
            }
        }
    } else {
        plugins
    }
}

/// Util function to convert a PluginResponse to a json string 
pub fn plugin_response_to_json(plugin_response: PluginResponse) -> String {
    let json = serde_json::to_string(&plugin_response).unwrap();
    json
}

/*************************
 * Config file structure *
 *************************/
/*
Example:
[look]
initial_width = 600 // the initial width of the window
disable_tip = false // disable the tip suggestion
input_placeholder = "Search" // the input placeholder

[network]
port = 8080 // the ws port
host = "localhost" // the ws host
db_port = 3306 // the db port
db_host = "localhost" // the db host

*/
/// Toml config file structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The look section
    pub look: Option<Look>,
    /// The network section
    pub network: Option<Network>,
}

/// The look section
#[derive(Debug, Serialize, Deserialize)]
pub struct Look {
    /// The initial width of the window
    pub initial_width: Option<u32>,
    /// Disable the tip suggestion
    pub disable_tip: Option<bool>,
    /// The input placeholder
    pub input_placeholder: Option<String>
}

/// The network section
#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
    /// The ws port
    pub port: Option<u32>,
    /// The ws host
    pub host: Option<String>,
    /// The db port
    pub db_port: Option<u32>,
    /// The db host
    pub db_host: Option<String>,
}

/******************************
 * Plugin cfg file structure  *
 *****************************/
/// Toml plugin config file structure
/// The config file is located in ~/.config/fsearch/plugins/<plugin_name>.toml
/// The plugin cmd should be in the $PATH or an absolute path to a program
/// that receives the query as an argument.
///
/// Pluging scripts can live in `~/.config/fsearch/scripts/<plugin_thing>` , can be write `@script:<plugin_thing>` in the cmd field for simpler usage
///
/// Example:
///  ```toml
///  name = "ls"
///  description = "List files"
///  cmd = "@script:myls" # the command is executed with the query as an argument (ls <query>)
///  run_on_any_query = false # if true, the plugin will run on any query, not just when the query starts with the plugin name
///  priority = 0 # the priority is used to sort the plugins displayed in the UI default is 0 and max is 3
///  dev = false # dev mode (not used yet)
///  ```
///  
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    /// The name of the plugin
    pub name: String,
    /// Plugin description
    pub description: String,
    /// The command to execute
    pub cmd: String,
    /// Run on any query
    pub run_on_any_query: Option<bool>,
    /// Priority
    /// The priority is used to sort the plugins displayed in the UI default is 0 and max is 3
    pub priority: Option<i32>,
    /// dev mode (not used yet)
    /// This mode will show debug info in the UI
    pub dev: Option<bool>,
}

/// Plugin action type
#[derive(Debug, Serialize, Deserialize)]
pub enum PluginActionType {
    Copy(String),
    Open(String),
    RunCmd(String),
    RunScript(String),
    Launch(String),
    Exit,
}

/// Plugin action
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginAction {
    pub action: PluginActionType,
    pub close_after_run: Option<bool>,
}

/// GtkComponent type
#[derive(Debug, Serialize, Deserialize)]
pub enum GtkComponentType {
    Box,
    Button,
    Label,
}

/// GtkComponent align
#[derive(Debug, Serialize, Deserialize)]
pub enum Align {
    Start,
    End,
    Center,
    Fill,
    Baseline,
}

/// GtkComponent Orientation
#[derive(Debug, Serialize, Deserialize)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// GtkComponent
#[derive(Debug, Serialize, Deserialize)]
pub struct GtkComponent {
    pub component_type: GtkComponentType,
    pub id: String,
    pub hexpand: Option<bool>,
    pub halign: Option<Align>,
    pub orientation: Option<Orientation>,
    pub classes: Vec<String>,
    pub text: Option<String>,
    pub children: Option<Vec<GtkComponent>>,

    /// only for buttons
    pub on_click: Option<PluginAction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginResponse {
    pub gtk: Option<Vec<GtkComponent>>,
    pub action: Option<PluginAction>,
    pub error: Option<String>,
    pub set_icon: Option<String>,
}

/// GtkComponent builder 
/// Example:
/// ```rust
/// use fsearch_core::GtkComponentBuilder;
/// use fsearch_core::GtkComponentType;
/// use fsearch_core::Align;
/// let gtk_component = GtkComponentBuilder::new(GtkComponentType::Box)
///    .add_class("my-class".to_string())
///    .hexpand(true)
///    .halign(Align::Center)
///    .text("My text".to_string())
///    .add_child(GtkComponentBuilder::new(GtkComponentType::Label)
///    .add_class("my-child-class".to_string())
///    .hexpand(true)
///    .halign(Align::Center)
///    .text("My child text".to_string())
///    .build())
///    .build();
/// ```
pub struct GtkComponentBuilder {
    component_type: GtkComponentType,
    id: String,
    classes: Vec<String>,
    hexpand: bool,
    halign: Align,
    orientation: Option<Orientation>,
    text: String,
    children: Vec<GtkComponent>,
    on_click: Option<PluginAction>,
}

impl GtkComponentBuilder {
    /// Create a new GtkComponentBuilder
    pub fn new(ctype: GtkComponentType) -> Self {
        Self {
            component_type: ctype,
            id: String::new(),
            classes: Vec::new(),
            hexpand: false,
            halign: Align::Start,
            orientation: None,
            text: String::new(),
            children: Vec::new(),
            on_click: None,
        }
    }

    /// Set the id of the component
    pub fn id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    /// Add a class to the component
    pub fn add_class(mut self, class: String) -> Self {
        self.classes.push(class);
        self
    }

    /// Set the hexpand property of the component
    pub fn hexpand(mut self, hexpand: bool) -> Self {
        self.hexpand = hexpand;
        self
    }

    /// Set the halign property of the component
    pub fn halign(mut self, halign: Align) -> Self {
        self.halign = halign;
        self
    }

    /// Set the text property of the component
    pub fn text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    /// Add a child to the component
    pub fn add_child(mut self, child: GtkComponent) -> Self {
        self.children.push(child);
        self
    }

    /// Add children to the component
    pub fn add_children(mut self, children: Vec<GtkComponent>) -> Self {
        self.children.extend(children);
        self
    }

    /// Set the on_click property of the component
    pub fn on_click(mut self, on_click: PluginAction) -> Self {
        self.on_click = Some(on_click);
        self
    }

    /// Build the GtkComponent
    pub fn build(self) -> GtkComponent {
        GtkComponent {
            component_type: self.component_type,
            id: self.id,
            classes: self.classes,
            hexpand: Some(self.hexpand),
            halign: Some(self.halign),
            orientation: self.orientation,
            text: Some(self.text),
            children: Some(self.children),
            on_click: self.on_click,
        }
    }
}
pub fn titleit(title: String) -> GtkComponent {
    new_label(title, "SectionTitle".to_string(), vec![], Some(true), Some(Align::Start))
}

pub fn wrap_section(content: GtkComponent) -> GtkComponent {
    new_box("Section".to_string(), vec![], Some(true), None, Some(Orientation::Vertical), Some(vec![content]))
}

pub fn contentify(title: String, content: Vec<GtkComponent>) -> GtkComponent {
    let title = titleit(title);
    let content = GtkComponentBuilder::new(GtkComponentType::Box)
        .id("Content".to_string())
        .hexpand(true)
        .add_children(content)
        .build();
    let box_content = GtkComponentBuilder::new(GtkComponentType::Box)
        .id("BoxContent".to_string())
        .hexpand(true)
        .add_children(vec![title, content])
        .build();

    wrap_section(box_content)
}

pub fn new_label(
    text: String, 
    id: String, 
    classes: Vec<String>,
    hexpand: Option<bool>,
    halign: Option<Align>,
) -> GtkComponent { 
    GtkComponent {
        component_type: GtkComponentType::Label,
        id,
        classes,
        hexpand,
        orientation: None,
        halign,
        text: Some(text),
        children: None,
        on_click: None,
    }
}

pub fn new_button(
    text: String, 
    id: String, 
    classes: Vec<String>,
    hexpand: Option<bool>,
    halign: Option<Align>,
    on_click: PluginAction,
    children: Option<Vec<GtkComponent>>
) -> GtkComponent { 
    GtkComponent {
        component_type: GtkComponentType::Button,
        id,
        classes,
        hexpand,
        halign,
        orientation: None,
        text: Some(text),
        children,
        on_click: Some(on_click),
    }
}

pub fn new_box(
    id: String, 
    classes: Vec<String>,
    hexpand: Option<bool>,
    halign: Option<Align>, 
    orientation: Option<Orientation>,
    children: Option<Vec<GtkComponent>>
) -> GtkComponent { 
    GtkComponent {
        component_type: GtkComponentType::Box,
        id,
        classes,
        hexpand,
        halign,
        orientation,
        text: None,
        children,
        on_click: None,
    }
}

pub fn new_plugin_response(
    gtk: Option<Vec<GtkComponent>>,
    action: Option<PluginAction>,
    error: Option<String>,
    set_icon: Option<String>,
) -> PluginResponse {
    PluginResponse {
        gtk,
        action,
        error,
        set_icon,
    }
}


pub fn new_plugin_action(
    action: PluginActionType,
    close_after_run: Option<bool>,
) -> PluginAction {
    PluginAction {
        action,
        close_after_run,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    const TEST_CFG: &str = r#"
        [look]
        initial_width = 600
        disable_tip = false
        input_placeholder = "Search"

        [network]
        port = 8080
        host = "localhost"
        db_port = 3306
        db_host = "localhost"
    "#;

    const TEST_PLUGIN_CFG: &str = r#"
        name = "ls"
        description = "List files"
        cmd = "@script:myls"
        run_on_any_query = false
        priority = 0
        dev = false
    "#;

    /// config file deserialization test 
    #[test]
    fn test_cfg_deserialization() {
        let cfg: Config = toml::from_str(TEST_CFG).unwrap();
        assert_eq!(cfg.look.as_ref().unwrap().initial_width.unwrap(), 600);
        assert_eq!(cfg.look.as_ref().unwrap().disable_tip.unwrap(), false);
        assert_eq!(cfg.look.as_ref().unwrap().input_placeholder.as_ref().unwrap(), "Search");
        assert_eq!(cfg.network.as_ref().unwrap().port.unwrap(), 8080);
        assert_eq!(cfg.network.as_ref().unwrap().host.as_ref().unwrap(), "localhost");
        assert_eq!(cfg.network.as_ref().unwrap().db_port.unwrap(), 3306);
        assert_eq!(cfg.network.as_ref().unwrap().db_host.as_ref().unwrap(), "localhost");
    }

    /// plugin config file deserialization test
    #[test]
    fn test_plugin_cfg_deserialization() {
        let plugin_cfg: PluginConfig = toml::from_str(TEST_PLUGIN_CFG).unwrap();
        assert_eq!(plugin_cfg.name, "ls");
        assert_eq!(plugin_cfg.description, "List files");
        assert_eq!(plugin_cfg.cmd, "@script:myls");
        assert_eq!(plugin_cfg.run_on_any_query.unwrap(), false);
        assert_eq!(plugin_cfg.priority.unwrap(), 0);
        assert_eq!(plugin_cfg.dev.unwrap(), false);
    }
}

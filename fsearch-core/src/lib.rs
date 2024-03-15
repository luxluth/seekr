use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// Get a text file content as a string
fn get_file_content(path: &PathBuf) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

/// Get the CSS file content
pub fn get_css() -> String {
    match env::var("XDG_CONFIG_HOME") {
        Ok(v) => {
            let css_path = PathBuf::from(v).join("fsearch").join("style.css");
            if let Ok(css) = get_file_content(&css_path) {
                css
            } else {
                include_str!("../defaults/style.css").to_string()
            }
        }
        Err(_) => match env::var("HOME") {
            Ok(v) => {
                let css_path = PathBuf::from(v)
                    .join(".config")
                    .join("fsearch")
                    .join("style.css");
                if let Ok(config) = get_file_content(&css_path) {
                    config
                } else {
                    include_str!("../defaults/style.css").to_string()
                }
            }
            Err(_) => include_str!("../defaults/style.css").to_string(),
        },
    }
}

/// Get the config .toml file content
pub fn get_cfg() -> Option<Config> {
    match env::var("XDG_CONFIG_HOME") {
        Ok(v) => {
            let cfg_path = PathBuf::from(v).join("fsearch").join("config.toml");
            if let Ok(cfg_content) = get_file_content(&cfg_path) {
                let cfg: Config = toml::from_str(&cfg_content).unwrap();
                Some(cfg)
            } else {
                Some(toml::from_str(include_str!("../defaults/config.toml")).unwrap())
            }
        }
        Err(_) => match env::var("HOME") {
            Ok(v) => {
                let cfg_path = PathBuf::from(v)
                    .join(".config")
                    .join("fsearch")
                    .join("config.toml");
                if let Ok(cfg_content) = get_file_content(&cfg_path) {
                    let cfg: Config = toml::from_str(&cfg_content).unwrap();
                    Some(cfg)
                } else {
                    Some(toml::from_str(include_str!("../defaults/config.toml")).unwrap())
                }
            }
            Err(_) => Some(toml::from_str(include_str!("../defaults/config.toml")).unwrap()),
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
                println!("Could not determine the plugin directory");
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
                        if let Ok(plugin_content) = get_file_content(&path) {
                            let plugin: PluginConfig = toml::from_str(&plugin_content).unwrap();
                            plugins.push(plugin);
                        }
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
    serde_json::to_string(&plugin_response).unwrap()
}

pub fn get_scripts_dir() -> String {
    let mut dir = String::new();

    match env::var("XDG_CONFIG_HOME") {
        Ok(v) => {
            dir = PathBuf::from(v)
                .join("fsearch")
                .join("scripts")
                .to_str()
                .unwrap()
                .to_string();
        }
        Err(_) => match env::var("HOME") {
            Ok(v) => {
                dir = PathBuf::from(v)
                    .join(".config")
                    .join("fsearch")
                    .join("scripts")
                    .to_str()
                    .unwrap()
                    .to_string();
            }
            Err(_) => {
                println!("Could not determine the script directory");
            }
        },
    }

    dir
}

/// Toml config file structure
/// Example:
/// ```toml
/// [look]
/// initial_width = 600 # the initial width of the window
/// disable_tip = false # disable the tip suggestion
/// input_placeholder = "Search" # the input placeholder
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The look section
    pub look: Option<Look>,
}

/// The look section
#[derive(Debug, Serialize, Deserialize)]
pub struct Look {
    /// The initial width of the window
    pub initial_width: Option<u32>,
    /// Disable the tip suggestion
    pub disable_tip: Option<bool>,
    /// The input placeholder
    pub input_placeholder: Option<String>,
}

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

/// Data type
#[derive(Debug, Serialize, Deserialize)]
pub enum DataType {
    Box,
    Button,
    EventBox,
    Image,
    Label,
}

/// Element align
#[derive(Debug, Serialize, Deserialize)]
pub enum Align {
    Start,
    End,
    Center,
    Fill,
    Baseline,
}

/// Element Orientation
#[derive(Debug, Serialize, Deserialize)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// Element
/// The element is used to build the UI in the frontend
/// The frontend will convert the Element to a GtkComponent
#[derive(Debug, Serialize, Deserialize)]
pub struct Element {
    /// element type (box, button, label)
    pub element_type: DataType,
    /// element id
    pub id: String,
    /// horizontal expand
    pub hexpand: Option<bool>,
    /// vertical expand
    pub vexpand: Option<bool>,
    /// Text wrap
    pub wrap: Option<bool>,
    /// ellipsize Text
    pub ellipsize: Option<bool>,
    /// element align
    pub halign: Option<Align>,
    /// box and button orientation
    pub orientation: Option<Orientation>,
    /// css classes
    pub classes: Vec<String>,
    /// labels text
    pub text: Option<String>,
    /// a list of children elements
    pub children: Option<Vec<Element>>,
    /// button click action
    pub on_click: Option<PluginAction>,
    /// image path
    pub image_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementBuilder {
    element_type: DataType,
    id: String,
    hexpand: Option<bool>,
    vexpand: Option<bool>,
    wrap: Option<bool>,
    ellipsize: Option<bool>,
    halign: Option<Align>,
    orientation: Option<Orientation>,
    classes: Vec<String>,
    text: Option<String>,
    children: Option<Vec<Element>>,
    on_click: Option<PluginAction>,
    image_path: Option<String>,
}

impl ElementBuilder {
    pub fn new(element_type: DataType) -> Self {
        Self {
            element_type,
            id: String::new(),
            hexpand: None,
            vexpand: None,
            wrap: None,
            ellipsize: None,
            halign: None,
            orientation: None,
            classes: Vec::new(),
            text: None,
            children: None,
            on_click: None,
            image_path: None,
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    pub fn hexpand(mut self, hexpand: bool) -> Self {
        self.hexpand = Some(hexpand);
        self
    }

    pub fn vexpand(mut self, vexpand: bool) -> Self {
        self.vexpand = Some(vexpand);
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = Some(wrap);
        self
    }

    pub fn ellipsize(mut self, ellipsize: bool) -> Self {
        self.ellipsize = Some(ellipsize);
        self
    }

    pub fn halign(mut self, halign: Align) -> Self {
        self.halign = Some(halign);
        self
    }

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    pub fn classes(mut self, classes: Vec<String>) -> Self {
        self.classes = classes;
        self
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }

    pub fn children(mut self, children: Vec<Element>) -> Self {
        self.children = Some(children);
        self
    }

    pub fn on_click(mut self, on_click: PluginAction) -> Self {
        self.on_click = Some(on_click);
        self
    }

    pub fn append(mut self, child: Element) -> Self {
        match self.children {
            Some(mut children) => {
                children.push(child);
                self.children = Some(children);
            }
            None => {
                self.children = Some(vec![child]);
            }
        }
        self
    }

    pub fn image_path(mut self, image_path: &str) -> Self {
        self.image_path = Some(image_path.to_string());
        self
    }

    pub fn build(self) -> Element {
        Element {
            element_type: self.element_type,
            id: self.id,
            hexpand: self.hexpand,
            vexpand: self.vexpand,
            wrap: self.wrap,
            ellipsize: self.ellipsize,
            halign: self.halign,
            orientation: self.orientation,
            classes: self.classes,
            text: self.text,
            children: self.children,
            on_click: self.on_click,
            image_path: self.image_path,
        }
    }
}

/// Plugin response
/// The plugin response is used to send data to the frontend
/// It contains the UI elements, the action to execute and the error message
/// The response is send to the frontend as a json string
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginResponse {
    pub title: Option<String>,
    pub elements: Vec<Element>,
    pub action: Option<PluginAction>,
    pub error: Option<String>,
    pub set_icon: Option<String>,
}

pub fn new_plugin_action(action: PluginActionType, close_after_run: Option<bool>) -> PluginAction {
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
        assert!(!cfg.look.as_ref().unwrap().disable_tip.unwrap());
        assert_eq!(
            cfg.look
                .as_ref()
                .unwrap()
                .input_placeholder
                .as_ref()
                .unwrap(),
            "Search"
        );
    }

    /// plugin config file deserialization test
    #[test]
    fn test_plugin_cfg_deserialization() {
        let plugin_cfg: PluginConfig = toml::from_str(TEST_PLUGIN_CFG).unwrap();
        assert_eq!(plugin_cfg.name, "ls");
        assert_eq!(plugin_cfg.description, "List files");
        assert_eq!(plugin_cfg.cmd, "@script:myls");
        assert!(!plugin_cfg.run_on_any_query.unwrap());
        assert_eq!(plugin_cfg.priority.unwrap(), 0);
        assert!(!plugin_cfg.dev.unwrap());
    }
}

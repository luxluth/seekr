use serde::{Deserialize, Serialize};
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
/// Example:
///  name = "ls"
///  description = "List files"
///  cmd = /some/path/myls # the command is executed with the query as an argument (ls <query>)
///
///  
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    /// The name of the plugin
    pub name: String,
    /// Plugin description
    pub description: String,
    /// The command to execute
    pub cmd: String,
    /// The icon to display (A path to a file)
    pub icon: Option<String>,
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// }

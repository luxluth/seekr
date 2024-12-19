use rust_i18n::t;
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
};
use tracing::{debug, warn};

pub const APP_ID: &str = "dev.luxluth.seekr";
pub const DEFAULT_CONFIG: &str = include_str!("./default.conf");

pub const DEFAULT_CSS: &str = include_str!("./style.css");

#[derive(Clone, Debug)]
pub struct MacroDef(pub MacroName, pub String);

#[derive(Clone, Debug)]
pub struct MacroName {
    pub invoke_name: String,
    pub display_name: String,
}

impl MacroName {
    pub fn parse(macro_key: &str) -> Self {
        if macro_key.contains('|') {
            let (l, r) = macro_key.split_once('|').unwrap();
            Self {
                invoke_name: l.trim().to_string(),
                display_name: r.trim().to_string(),
            }
        } else {
            Self {
                invoke_name: macro_key.trim().to_string(),
                display_name: macro_key.trim().to_string(),
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct GeneralConf {
    pub theme: String,
    pub terminal: String,
    pub args: Vec<String>,
    pub search_placeholder: String,
}

impl Default for GeneralConf {
    fn default() -> Self {
        GeneralConf {
            theme: "Adwaita".to_string(),
            terminal: "kitty".to_string(),
            args: vec!["-e".to_string()],
            search_placeholder: t!("search_placeholder").to_string(),
        }
    }
}

pub type MacroMap = HashMap<String, MacroDef>;

#[derive(Default, Clone, Debug)]
pub struct Config {
    pub general: GeneralConf,
    pub css: String,
    pub macros: MacroMap,
    pub config_dir: PathBuf,
}

impl Config {
    pub fn get_conf(conf_path: &PathBuf) -> (GeneralConf, MacroMap) {
        let mut general = GeneralConf::default();
        let mut macros: MacroMap = MacroMap::new();
        if let Ok(mut f) = std::fs::File::open(conf_path) {
            let mut data = String::new();
            let _ = f.read_to_string(&mut data);
            let mut is_in_general = false;
            let mut is_in_macros = false;

            for (line, item) in ini_roundtrip::Parser::new(&data).enumerate() {
                match item {
                    ini_roundtrip::Item::Error(e) => {
                        warn!("{}:{line}: {e}", conf_path.display());
                    }
                    ini_roundtrip::Item::Section {
                        name: "general", ..
                    } => {
                        is_in_general = true;
                    }
                    ini_roundtrip::Item::Section { name: "macros", .. } => {
                        is_in_general = false;
                        is_in_macros = true;
                    }
                    ini_roundtrip::Item::Property {
                        key: "theme", val, ..
                    } => {
                        if is_in_general && val.is_some() {
                            general.theme = val.unwrap().trim().to_string();
                        }
                    }
                    ini_roundtrip::Item::Property {
                        key: "terminal",
                        val,
                        ..
                    } => {
                        if is_in_general && val.is_some() {
                            general.terminal = val.unwrap().trim().to_string();
                        }
                    }
                    ini_roundtrip::Item::Property {
                        key: "args", val, ..
                    } => {
                        if is_in_general && val.is_some() {
                            general.args = val
                                .unwrap()
                                .trim()
                                .split(' ')
                                .map(|x| x.to_string())
                                .collect();
                        }
                    }
                    ini_roundtrip::Item::Property {
                        key: "search_placeholder",
                        val,
                        ..
                    } => {
                        if is_in_general && val.is_some() {
                            general.search_placeholder = val.unwrap().to_string();
                        }
                    }

                    ini_roundtrip::Item::Property { key, val, .. } => {
                        if is_in_macros && val.is_some() {
                            let macro_name = MacroName::parse(&key);
                            let invoke_name = macro_name.invoke_name.clone();
                            let r#macro = MacroDef(macro_name, val.unwrap().to_string());
                            macros.insert(invoke_name, r#macro);
                        }
                    }
                    _ => {}
                }
            }
        }
        return (general, macros);
    }

    pub fn parse(path: std::path::PathBuf) -> Self {
        let mut css = DEFAULT_CSS.to_string();
        let config_dir = path.parent().unwrap().to_path_buf();
        let css_path = path.parent().unwrap().join("style.css");
        if css_path.exists() {
            if let Ok(mut f) = std::fs::File::open(&css_path) {
                css = String::new();
                let _ = f.read_to_string(&mut css);
            }
        } else {
            if let Ok(mut f) = std::fs::File::create(&css_path) {
                let _ = f.write(DEFAULT_CSS.as_bytes());
            }
        }

        let (general, macros) = Self::get_conf(&path);
        debug!("Loaded macros .... {:#?}", macros);

        return Self {
            general,
            css,
            macros,
            config_dir,
        };
    }
}

pub fn init_config_dir() -> std::path::PathBuf {
    let raw_path = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or(format!("{}/.config", std::env::var("HOME").unwrap()));
    let base_dir = std::path::Path::new(&raw_path);
    let config_dir = base_dir.join("seekr/");

    if !config_dir.exists() {
        let _ = std::fs::create_dir_all(&config_dir);
    }

    let config_file = config_dir.join("default.conf");
    debug!("config_path: {}", config_file.display());

    if !config_file.exists() {
        if let Ok(mut f) = std::fs::File::create(&config_file) {
            let _ = f.write(
                DEFAULT_CONFIG
                    .replace("%PLACEHOLDER%", &t!("search_placeholder").to_string())
                    .as_bytes(),
            );
        }
    }

    return config_file;
}

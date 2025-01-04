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

#[derive(Clone, Debug)]
pub struct LayerShellConf {
    pub active: bool,
    pub top: i32,
    pub left: i32,
}

impl Default for LayerShellConf {
    fn default() -> Self {
        Self {
            active: false,
            top: 50,
            left: -1,
        }
    }
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
    #[allow(dead_code)]
    pub gtk_layer_shell_conf: LayerShellConf,
    #[allow(dead_code)]
    pub is_wayland: bool,
}

#[derive(PartialEq, Eq)]
enum ParsingState {
    General,
    Macros,
    GtkLayerShell,
    NotSet,
}

impl Config {
    pub fn get_conf(conf_path: &PathBuf) -> (GeneralConf, MacroMap, LayerShellConf) {
        let mut general = GeneralConf::default();
        let mut macros: MacroMap = MacroMap::new();
        let mut gtk_layer_shell_conf = LayerShellConf::default();
        if let Ok(mut f) = std::fs::File::open(conf_path) {
            let mut data = String::new();
            let _ = f.read_to_string(&mut data);
            let mut state: ParsingState = ParsingState::NotSet;

            for (line, item) in ini_roundtrip::Parser::new(&data).enumerate() {
                match item {
                    ini_roundtrip::Item::Error(e) => {
                        warn!("{}:{line}: {e}", conf_path.display());
                    }
                    ini_roundtrip::Item::Section {
                        name: "general", ..
                    } => {
                        state = ParsingState::General;
                    }
                    ini_roundtrip::Item::Section {
                        name: "gtk-layer-shell",
                        ..
                    } => {
                        state = ParsingState::GtkLayerShell;
                    }
                    ini_roundtrip::Item::Section { name: "macros", .. } => {
                        state = ParsingState::Macros;
                    }
                    ini_roundtrip::Item::Property {
                        key: "active", val, ..
                    } => {
                        if state == ParsingState::GtkLayerShell && val.is_some() {
                            gtk_layer_shell_conf.active = val.unwrap().trim() == "true";
                        }
                    }
                    ini_roundtrip::Item::Property {
                        key: "top", val, ..
                    } => {
                        if state == ParsingState::GtkLayerShell && val.is_some() {
                            gtk_layer_shell_conf.top = val.unwrap().parse().unwrap_or(50);
                        }
                    }
                    ini_roundtrip::Item::Property {
                        key: "left", val, ..
                    } => {
                        if state == ParsingState::GtkLayerShell && val.is_some() {
                            gtk_layer_shell_conf.left = val.unwrap().parse().unwrap_or(-1);
                        }
                    }
                    ini_roundtrip::Item::Property {
                        key: "theme", val, ..
                    } => {
                        if state == ParsingState::General && val.is_some() {
                            general.theme = val.unwrap().trim().to_string();
                        }
                    }
                    ini_roundtrip::Item::Property {
                        key: "terminal",
                        val,
                        ..
                    } => {
                        if state == ParsingState::General && val.is_some() {
                            general.terminal = val.unwrap().trim().to_string();
                        }
                    }
                    ini_roundtrip::Item::Property {
                        key: "args", val, ..
                    } => {
                        if state == ParsingState::General && val.is_some() {
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
                        if state == ParsingState::General && val.is_some() {
                            general.search_placeholder = val.unwrap().to_string();
                        }
                    }

                    ini_roundtrip::Item::Property { key, val, .. } => {
                        if state == ParsingState::Macros && val.is_some() {
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
        return (general, macros, gtk_layer_shell_conf);
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

        let (general, macros, gtk_layer_shell_conf) = Self::get_conf(&path);
        debug!("Loaded macros .... {:#?}", macros);

        return Self {
            general,
            css,
            macros,
            config_dir,
            gtk_layer_shell_conf,
            is_wayland: std::env::var("XDG_SESSION_TYPE")
                .unwrap_or("x11".to_string())
                .to_lowercase()
                == "wayland",
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

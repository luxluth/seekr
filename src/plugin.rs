use std::{collections::HashMap, io::Read};

use mlua::prelude::*;
use tracing::{debug, error, warn};

pub struct PluginLoader {
    ctxt: Lua,
    plugins: HashMap<String, Plugin>,
}

#[derive(Debug)]
pub struct Plugin {
    __api_version: i64,
    name: String,
    description: Option<String>,
    title: Option<String>,

    on_input: Option<LuaFunction>,
    on_startup: Option<LuaFunction>,
    on_exit: Option<LuaFunction>,

    table: LuaTable,
}

impl Plugin {
    pub fn from_table(table: LuaTable) -> Result<Self, LuaError> {
        if table.raw_get::<LuaString>("name").is_ok()
            && table.raw_get::<LuaInteger>("api_version").is_ok()
        {
            let mut plug = Plugin {
                __api_version: table.raw_get::<LuaInteger>("api_version")?,
                name: table.raw_get::<LuaString>("name")?.to_string_lossy(),

                description: None,
                title: None,

                on_input: None,
                on_startup: None,
                on_exit: None,

                table,
            };

            if let Ok(desc) = plug.table.raw_get::<LuaString>("description") {
                plug.description = Some(desc.to_string_lossy());
            }

            if let Ok(title) = plug.table.raw_get::<LuaString>("title") {
                plug.title = Some(title.to_string_lossy());
            }

            if let Ok(on_input) = plug.table.raw_get::<LuaFunction>("onInput") {
                plug.on_input = Some(on_input);
            }

            if let Ok(on_exit) = plug.table.raw_get::<LuaFunction>("onExit") {
                plug.on_exit = Some(on_exit);
            }

            if let Ok(on_startup) = plug.table.raw_get::<LuaFunction>("onStartup") {
                plug.on_startup = Some(on_startup);
            }

            return Ok(plug);
        } else {
            Err(LuaError::external("This is not a valid seekr plugin"))
        }
    }
}

impl PluginLoader {
    pub fn new() -> Self {
        PluginLoader {
            ctxt: Lua::new(),
            plugins: HashMap::new(),
        }
    }

    pub fn start(&mut self) {
        for (_, plugin) in self.plugins.iter() {
            if plugin.on_startup.is_some() {
                let _ = plugin.on_startup.clone().unwrap().clone().call::<()>(());
            }
        }
    }

    fn load(&mut self, filepath: std::path::PathBuf) {
        if let Some(ext) = filepath.extension() {
            if ext == "lua" {
                if let Ok(mut f) = std::fs::File::open(&filepath) {
                    let mut file_content = String::new();
                    match f.read_to_string(&mut file_content) {
                        Ok(_) => match self.ctxt.load(&file_content).eval::<LuaTable>() {
                            Ok(table) => {
                                if table.raw_get::<LuaString>("name").is_ok()
                                    && table.raw_get::<LuaInteger>("api_version").is_ok()
                                {
                                    match Plugin::from_table(table) {
                                        Ok(plug) => {
                                            let name = plug.name.clone();
                                            if self.plugins.get(&name).is_none() {
                                                self.plugins.insert(name, plug);
                                            } else {
                                                warn!("A plugin named ({}) has already been loaded from ({})", name, filepath.display());
                                            }
                                        }
                                        Err(e) => {
                                            error!("{:?}", e);
                                        }
                                    }
                                } else {
                                    error!("Uncompatible plugin format {}", filepath.display());
                                }
                            }
                            Err(e) => {
                                error!("{:?}", e);
                            }
                        },
                        Err(e) => {
                            error!("{:?}", e);
                        }
                    }
                }
            }
        }
    }

    pub fn lookup(&mut self, config_dir: &std::path::Path) {
        let start = std::time::Instant::now();
        let plugins_dir = config_dir.join("plugins");
        if plugins_dir.exists() && plugins_dir.is_dir() {
            if let Ok(files_path) = std::fs::read_dir(plugins_dir) {
                for filepath in files_path {
                    if filepath.is_ok() {
                        let p = filepath.unwrap().path();
                        if p.is_file() {
                            self.load(p);
                        }
                    }
                }
            }
        }

        let elapsed = start.elapsed();
        if elapsed.as_millis() == 0 {
            debug!(
                "{} plugin(s) loaded in {}µs",
                self.plugins.len(),
                start.elapsed().as_micros()
            );
        } else {
            debug!(
                "{} plugin(s) loaded in {}ms",
                self.plugins.len(),
                start.elapsed().as_millis()
            );
        }
    }
}

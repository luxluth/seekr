use std::{collections::HashMap, io::Read};

// mod bindings;

use mlua::HookTriggers;
use mlua::prelude::*;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, warn};

thread_local! {
    static CURRENT_SEQ: std::cell::RefCell<u64> = std::cell::RefCell::new(0);
    static CURRENT_PLUGIN_NAME: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
}

fn check_validity(latest_seq: &Arc<Mutex<HashMap<String, u64>>>) -> LuaResult<LuaVmState> {
    let seq = CURRENT_SEQ.with(|s| *s.borrow());
    let name = CURRENT_PLUGIN_NAME.with(|s| s.borrow().clone());

    if name.is_empty() {
        return Ok(LuaVmState::Continue);
    }

    if let Ok(guard) = latest_seq.lock() {
        if let Some(latest) = guard.get(&name) {
            if seq < *latest {
                return Err(LuaError::runtime("interrupted"));
            }
        }
    }
    Ok(LuaVmState::Continue)
}

#[derive(Debug, Clone)]
pub enum Trigger {
    Any,
    Command(String),
    Contains(String),
}

#[derive(Debug, Clone)]
pub struct Plugin {
    __api_version: i64,
    name: String,
    description: Option<String>,
    title: Option<String>,

    triggers: Vec<Trigger>,

    on_input: Option<LuaFunction>,
    on_activate: Option<LuaFunction>,
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

                triggers: vec![],

                on_input: None,
                on_activate: None,
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

            if let Ok(triggers) = plug.table.raw_get::<LuaTable>("triggers") {
                for pair in triggers.pairs::<LuaValue, LuaValue>() {
                    let (_, value) = pair?;
                    if let LuaValue::String(s) = value {
                        let s = s.to_string_lossy();
                        if s == "any" {
                            plug.triggers.push(Trigger::Any);
                        } else if s.starts_with("command=") {
                            let cmd = s.replace("command=", "");
                            plug.triggers.push(Trigger::Command(cmd));
                        } else if s.starts_with("contains=") {
                            let cnt = s.replace("contains=", "");
                            plug.triggers.push(Trigger::Contains(cnt));
                        }
                    }
                }
            }

            // Default trigger if none specified
            if plug.triggers.is_empty() {
                plug.triggers
                    .push(Trigger::Command(format!("/{}", plug.name)));
            }

            if let Ok(on_input) = plug.table.raw_get::<LuaFunction>("onInput") {
                plug.on_input = Some(on_input);
            }

            if let Ok(on_activate) = plug.table.raw_get::<LuaFunction>("onActivate") {
                plug.on_activate = Some(on_activate);
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

    pub fn matches(&self, term: &str) -> bool {
        for trigger in &self.triggers {
            match trigger {
                Trigger::Any => return true,
                Trigger::Command(cmd) => {
                    if term.starts_with(cmd) {
                        return true;
                    }
                }
                Trigger::Contains(cnt) => {
                    if term.contains(cnt) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

struct SeekrGlobal {
    config_dir: String,
    tx_ui: async_channel::Sender<PluginUiEvent>,
    latest_seq: Arc<Mutex<HashMap<String, u64>>>,
}

impl LuaUserData for SeekrGlobal {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("config_dir", |_, this| Ok(this.config_dir.clone()));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("env", |_lua, _this, key: String| match std::env::var(key) {
            Ok(value) => {
                return Ok(value);
            }
            Err(_) => Ok(String::new()),
        });

        methods.add_method("log", |_lua, _this, data: (String, String)| {
            debug!("{} :: {}", data.0, data.1);
            Ok(())
        });

        methods.add_method("clear_results", |_lua, this, plugin_name: String| {
            let _ = check_validity(&this.latest_seq)?;
            let seq = CURRENT_SEQ.with(|s| *s.borrow());
            let _ = this
                .tx_ui
                .send_blocking(PluginUiEvent::Clear { plugin_name, seq });
            Ok(())
        });

        methods.add_method(
            "show_image_grid",
            |_lua, this, data: (String, Vec<String>, Option<String>)| {
                let _ = check_validity(&this.latest_seq)?;
                let seq = CURRENT_SEQ.with(|s| *s.borrow());
                let _ = this.tx_ui.send_blocking(PluginUiEvent::ShowImageGrid {
                    plugin_name: data.0,
                    images: data.1,
                    subtitle: data.2,
                    seq,
                });
                Ok(())
            },
        );

        methods.add_method(
            "show_info_box",
            |_lua, this, data: (String, String, String)| {
                let _ = check_validity(&this.latest_seq)?;
                let seq = CURRENT_SEQ.with(|s| *s.borrow());
                let _ = this.tx_ui.send_blocking(PluginUiEvent::ShowInfoBox {
                    plugin_name: data.0,
                    title: data.1,
                    body: data.2,
                    seq,
                });
                Ok(())
            },
        );

        methods.add_method("show_console", |_lua, this, data: (String, String)| {
            let _ = check_validity(&this.latest_seq)?;
            let seq = CURRENT_SEQ.with(|s| *s.borrow());
            let _ = this.tx_ui.send_blocking(PluginUiEvent::ShowConsole {
                plugin_name: data.0,
                command: data.1,
                seq,
            });
            Ok(())
        });

        methods.add_method("glob", |_lua, this, pattern: String| {
            let _ = check_validity(&this.latest_seq)?;
            let result = glob::glob(&pattern);
            if result.is_ok() {
                let mut paths = vec![];
                for path in result.unwrap().into_iter() {
                    if let Ok(path) = path {
                        paths.push(format!("{}", path.display()));
                    }
                }
                Ok(paths)
            } else {
                Ok(vec![])
            }
        });

        methods.add_method("exec", |_lua, this, cmd: String| {
            let _ = check_validity(&this.latest_seq)?;
            let _ = std::process::Command::new("sh").arg("-c").arg(cmd).spawn();
            Ok(())
        });

        methods.add_method("read", |_lua, this, cmd: String| {
            let _ = check_validity(&this.latest_seq)?;
            if let Ok(output) = std::process::Command::new("sh").arg("-c").arg(cmd).output() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    return Ok(stdout);
                }
            }
            Ok(String::new())
        });

        methods.add_method("json_to_lua", |lua, this, json_str: String| {
            let _ = check_validity(&this.latest_seq)?;
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
                return Ok(json_to_lua_value(lua, &value).unwrap_or(LuaValue::Nil));
            }
            Ok(LuaValue::Nil)
        });

        methods.add_method("close", |_lua, this, (): ()| {
            let _ = this.tx_ui.send_blocking(PluginUiEvent::Close);
            Ok(())
        });
    }
}

fn json_to_lua_value<'lua>(
    lua: &'lua Lua,
    value: &serde_json::Value,
) -> Result<LuaValue, LuaError> {
    match value {
        serde_json::Value::Null => Ok(LuaValue::Nil),
        serde_json::Value::Bool(b) => Ok(LuaValue::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Ok(LuaValue::Nil)
            }
        }
        serde_json::Value::String(s) => Ok(LuaValue::String(lua.create_string(s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua_value(lua, v)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj {
                table.set(k.as_str(), json_to_lua_value(lua, v)?)?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginLoader {
    ctxt: Lua,
    plugins: HashMap<String, Plugin>,
    config_dir: String,
    rx: std::sync::Arc<std::sync::Mutex<Receiver<MessageToPlugins>>>,
    pub sx: Sender<MessageToPlugins>,
    pub tx_ui: async_channel::Sender<PluginUiEvent>,
    latest_seq: Arc<Mutex<HashMap<String, u64>>>,
    current_seq: u64,
}

#[derive(Debug, Clone)]
pub enum MessageToPlugins {
    Term(String),
    Activate {
        plugin_name: String,
        payload: String,
    },
}

#[derive(Debug, Clone)]
pub enum PluginUiEvent {
    NewSearch(u64),
    Processing {
        plugin_name: String,
        state: bool,
        seq: u64,
    },
    ShowImageGrid {
        plugin_name: String,
        images: Vec<String>,
        subtitle: Option<String>,
        seq: u64,
    },
    ShowInfoBox {
        plugin_name: String,
        title: String,
        body: String,
        seq: u64,
    },
    ShowConsole {
        plugin_name: String,
        command: String,
        seq: u64,
    },
    Clear {
        plugin_name: String,
        seq: u64,
    },
    Close,
}

impl PluginLoader {
    pub fn new(config_dir: &std::path::Path, tx_ui: async_channel::Sender<PluginUiEvent>) -> Self {
        let ctxt = Lua::new();
        let config_dir = config_dir.to_str().unwrap().to_string();
        let latest_seq = Arc::new(Mutex::new(HashMap::new()));
        let hook_seq = latest_seq.clone();

        let _ = ctxt.set_hook(HookTriggers::EVERY_LINE, move |_lua, _debug| {
            check_validity(&hook_seq)
        });

        let _ = ctxt.globals().set(
            "seekr",
            SeekrGlobal {
                config_dir: config_dir.clone(),
                tx_ui: tx_ui.clone(),
                latest_seq: latest_seq.clone(),
            },
        );

        let (sx, rx) = mpsc::channel::<MessageToPlugins>();

        PluginLoader {
            ctxt,
            plugins: HashMap::new(),
            config_dir,
            rx: std::sync::Arc::new(std::sync::Mutex::new(rx)),
            sx,
            tx_ui,
            latest_seq,
            current_seq: 0,
        }
    }

    pub fn start(mut self) {
        for (_, plugin) in self.plugins.iter() {
            if plugin.on_startup.is_some() {
                let _ = plugin.on_startup.clone().unwrap().clone().call::<()>(());
            }
        }

        while let Ok(msg) = self.rx.lock().unwrap().recv() {
            match msg {
                MessageToPlugins::Term(term) => {
                    self.current_seq += 1;
                    let _ = self
                        .tx_ui
                        .send_blocking(PluginUiEvent::NewSearch(self.current_seq));

                    for (_, plugin) in self.plugins.iter() {
                        if plugin.matches(&term) {
                            self.latest_seq
                                .lock()
                                .unwrap()
                                .insert(plugin.name.clone(), self.current_seq);
                            if let Some(on_input) = &plugin.on_input {
                                let on_input = on_input.clone();
                                let term = term.clone();
                                let seq = self.current_seq;
                                let tx_ui = self.tx_ui.clone();
                                let plugin_name = plugin.name.clone();

                                std::thread::spawn(move || {
                                    CURRENT_SEQ.with(|s| *s.borrow_mut() = seq);
                                    CURRENT_PLUGIN_NAME
                                        .with(|s| *s.borrow_mut() = plugin_name.clone());

                                    let _ = tx_ui.send_blocking(PluginUiEvent::Processing {
                                        plugin_name: plugin_name.clone(),
                                        state: true,
                                        seq,
                                    });
                                    let _ = on_input.call::<()>(term);
                                    let _ = tx_ui.send_blocking(PluginUiEvent::Processing {
                                        plugin_name,
                                        state: false,
                                        seq,
                                    });
                                });
                            }
                        }
                    }
                }
                MessageToPlugins::Activate {
                    plugin_name,
                    payload,
                } => {
                    if let Some(plugin) = self.plugins.get(&plugin_name) {
                        if let Some(on_activate) = &plugin.on_activate {
                            let _ = on_activate.call::<()>(payload);
                        }
                    }
                }
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
                                                warn!(
                                                    "A plugin named ({}) has already been loaded from ({})",
                                                    name,
                                                    filepath.display()
                                                );
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

    pub fn lookup(&mut self) {
        let config_dir = std::path::Path::new(&self.config_dir);
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
        debug!("{} plugin(s) loaded in {elapsed:?}", self.plugins.len());
    }
}

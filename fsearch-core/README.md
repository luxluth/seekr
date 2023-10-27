# fsearch-core

`fsearch` core utils for building plugins in rust.

## What You have access to ?

### Functions

```rs
get_css() -> String;
get_cfg() -> Option<Config>;
get_plugins() -> Vec<PluginConfig>;

plugin_response_to_json(plugin_response: PluginResponse) -> String;
```

### Structs & Enums

```rs
pub struct Config {
    /// The look section
    pub look: Option<Look>,
    /// The network section
    pub network: Option<Network>,
}

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

pub struct PluginResponse {
    pub gtk: Option<Vec<GtkComponent>>,
    pub actions: Option<Vec<PluginAction>>,
}
```

All these structs can be deserialize 

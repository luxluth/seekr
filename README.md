<div align="center">
    
# seekr

_System search util for linux_

![seekr-demo](./assets/seekr-demo.gif)

</div>

## Installation

```sh
cargo install seekr-util
```

### Features

- `gtk-layer-shell` allow the seekr window to be displayed as a layer of the
  desktop _[wlr-layer-shell-unstable-v1](https://wayland.app/protocols/wlr-layer-shell-unstable-v1)_
- `localsearch`, in addition to searching apps to launch, with this feature, it
  is possible to search through files using the same method as the gnome
  shell.
- `Lua plugins`, extend the functionality of seekr by adding custom search
  engines and tools using the Lua programming language.

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [gtk4](https://www.gtk.org/docs/installations/)
- [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) (for gtk-layer-shell support)
- [localsearch](https://gitlab.gnome.org/GNOME/localsearch) (for localsearch)

## Usage

Running seekr is as simple as

```sh
$ seekr
```

`seekr` can also be run silently by passing the `--silent` options.
This will run seekr without open a window

## Configuration

On the first run of the app, configurations files will be generated into
`$XDG_CONFIG_HOME/seekr` or `$HOME/.config/seekr`

## Plugins

`seekr` supports Lua plugins to extend its functionality. Plugins are loaded from
the `plugins` directory in your configuration folder (e.g., `~/.config/seekr/plugins`).

A plugin is a Lua script that returns a table containing metadata and event handlers:

```lua
return {
    name = "my_plugin",
    api_version = 1,
    description = "A simple example plugin",
    triggers = { "command=/test" },
    onInput = function(term)
        seekr:log("my_plugin", "User typed: " .. term)
    end,
}
```

Available event handlers: `onInput`, `onEnter`, `onActivate`, `onStartup`, `onExit`.

The `seekr` global object provides several methods for plugins:
- `seekr:show_info_box(plugin_name, title, body)`
- `seekr:show_image_grid(plugin_name, images, subtitle)`
- `seekr:show_console(plugin_name, command)`
- `seekr:exec(command)`
- `seekr:read(command)`

## Contributing

You can contribute to the project in several ways:

- Translating the app via the [./locales/app.yml](./locales/app.yml) file
- Adding new functionalities to improve the tool
- Suggesting new widgets for plugins to use

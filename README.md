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

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [gtk4](https://www.gtk.org/docs/installations/)
- [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) (for gtk-layer-shell support)
- [tinysparql](https://gnome.pages.gitlab.gnome.org/tinysparql/) (for localsearch)

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

## Contributing

You can contribute to the project in two ways :

- Translating the app via the [./locales/app.yml](./locales/app.yml) file
- Adding new functionalities to improve the tool

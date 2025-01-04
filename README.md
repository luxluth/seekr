<div align="center">
    
# seekr

_System search util for linux_

</div>

![seekr-demo](./assets/seekr-demo.gif)

## Installation

```sh
cargo install seekr-util
```

> [!NOTE]
> To enable gtk-layer-shell support, enable the `gtk-layer-shell` feature as follow
>
> `cargo install seekr-util --features=gtk-layer-shell`
> make sure to have [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell)
> installed on your system

## Configuration

On the first run of the app, configurations files will be generated into
`$XDG_CONFIG_HOME/seekr` or `$HOME/.config/seekr`

## Contributing

You can contribute to the project in two ways :

- Translating the app via the [./locales/app.yml](./locales/app.yml) file
- Adding new functionalities to improve the tool

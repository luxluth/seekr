use conf::MacroDef;
use gtk::glib;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
#[cfg(feature = "gtk-layer-shell")]
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use rust_i18n::t;
use search::SearchManager;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use tokio::runtime::Runtime;
use ui::entry_button::EntryButton;

mod app;
mod bus;
mod conf;
mod icons;
mod locale;
mod plugin;
mod resources;
mod search;
mod ui;

rust_i18n::i18n!("locales", fallback = "en");

static IN_MACRO_MODE: AtomicBool = AtomicBool::new(false);

#[derive(Default, Clone, Copy)]
struct StartupOptions {
    silent: bool,
}

impl StartupOptions {
    fn read_args(&mut self) {
        for arg in std::env::args() {
            match arg.as_str() {
                "--silent" => self.silent = true,
                _ => {}
            }
        }
    }
}

fn activate(
    config: conf::Config,
    app: &Application,
    opts: StartupOptions,
    to_plugins: Sender<plugin::MessageToPlugins>,
) {
    let settings = gtk::Settings::default().expect("Failed to create GTK settings.");
    settings.set_gtk_icon_theme_name(Some(&config.general.theme));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("seekr")
        .css_name("window")
        .resizable(false)
        .decorated(false)
        .hide_on_close(true)
        .build();

    #[cfg(feature = "gtk-layer-shell")]
    {
        window.add_css_class("is_layer");
        if config.is_wayland
            && config.gtk_layer_shell_conf.active
            && gtk4_layer_shell::is_supported()
        {
            let key_press_controller = gtk::EventControllerKey::new();
            key_press_controller.connect_key_pressed(glib::clone!(
                #[strong]
                window,
                move |_, key, _, _| {
                    if gtk::gdk::Key::Escape == key {
                        window.close();
                    }
                    glib::Propagation::Proceed
                }
            ));
            window.add_controller(key_press_controller);

            window.init_layer_shell();
            window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
            window.set_namespace("seekr");
            window.set_layer(Layer::Top);

            let anchors = [
                (Edge::Top, config.gtk_layer_shell_conf.top >= 0),
                (Edge::Left, config.gtk_layer_shell_conf.left >= 0),
                (Edge::Bottom, false),
                (Edge::Right, false),
            ];

            if config.gtk_layer_shell_conf.top >= 0 {
                window.set_margin(Edge::Top, config.gtk_layer_shell_conf.top);
            }

            if config.gtk_layer_shell_conf.left >= 0 {
                window.set_margin(Edge::Left, config.gtk_layer_shell_conf.left);
            }

            for (anchor, state) in anchors {
                window.set_anchor(anchor, state);
            }
        }
    }

    if let Ok(xdg_current_desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        if xdg_current_desktop.to_lowercase() == "gnome" {
            window.add_css_class("gnome");
        }
    }

    window.set_default_size(600, 0);

    let (manager, (tomanager, frommanager)) = SearchManager::new();
    manager.manage();

    let entry = gtk::Text::builder()
        .hexpand(true)
        .css_name("input")
        .activates_default(true)
        .placeholder_text(&config.general.search_placeholder)
        .build();

    let represent_action = gtk::gio::SimpleAction::new("represent", None);
    represent_action.connect_activate(glib::clone!(
        #[weak]
        window,
        #[strong]
        tomanager,
        move |_, _| {
            let _ = tomanager.send(search::SearchEvent::Represent);
            window.present();
        }
    ));
    window.add_action(&represent_action);

    let input_container = gtk::Box::builder()
        .height_request(60)
        .hexpand(true)
        .spacing(10)
        .css_name("inputBox")
        .name("inputBox")
        .build();

    let macro_hint = gtk::Label::builder().css_name("macroHint").build();
    macro_hint.set_visible(false);

    entry.connect_activate(glib::clone!(
        #[strong]
        macro_hint,
        #[strong]
        config,
        #[strong]
        tomanager,
        move |e| {
            let entry = e.text().to_string();
            if IN_MACRO_MODE.load(Ordering::Relaxed) {
                // TODO: find a better way to store current macro tag other than
                // the css class name
                if let Some(def) = config.macros.get(macro_hint.css_classes()[0].as_str()) {
                    let MacroDef(_, command) = def;
                    let shell_cmd = command
                        .replace("{ENTRY}", &entry)
                        .replace("{CONFIG_DIR}", config.config_dir.to_str().unwrap());
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(shell_cmd)
                        .spawn();

                    e.set_text("");
                    macro_hint.set_visible(false);
                    IN_MACRO_MODE.store(false, Ordering::Relaxed);

                    let _ = tomanager.send(search::SearchEvent::RequestClose);
                }
            }
        }
    ));

    entry.connect_backspace(glib::clone!(
        #[strong]
        macro_hint,
        #[strong]
        input_container,
        move |e| if e.text().is_empty() && IN_MACRO_MODE.load(Ordering::Relaxed) {
            macro_hint.set_visible(false);
            macro_hint.set_text("");
            input_container.set_css_classes(&[]);
            IN_MACRO_MODE.store(false, Ordering::Relaxed);
        }
    ));

    let mut suggestions = config
        .macros
        .keys()
        .map(|m| format!("@{}", m.clone()))
        .collect::<Vec<String>>();
    suggestions.sort();
    let suggestions = suggestions;

    entry.connect_changed(glib::clone!(
        #[strong]
        tomanager,
        #[strong]
        macro_hint,
        #[strong]
        input_container,
        #[strong]
        config,
        #[strong]
        to_plugins,
        move |e| {
            let term = e.text().to_string();
            if !term.is_empty() {
                e.set_css_classes(&["has_input"]);
                if !IN_MACRO_MODE.load(Ordering::Relaxed) {
                    if let Some((t, _)) = term.split_once(' ') {
                        if let Ok(match_idx) = suggestions.binary_search(&t.to_string()) {
                            let macro_name = suggestions[match_idx].clone().replace('@', "");
                            let r#macro = config.macros.get(&macro_name).unwrap();
                            macro_hint.set_text(&r#macro.0.display_name);
                            macro_hint.set_visible(true);
                            IN_MACRO_MODE.store(true, Ordering::Relaxed);
                            e.set_text("");
                            macro_hint.set_css_classes(&[&macro_name]);
                            input_container.set_css_classes(&["macro_mode"]);
                        } else {
                            macro_hint.set_visible(false);
                            IN_MACRO_MODE.store(false, Ordering::Relaxed);
                        }
                    } else {
                        macro_hint.set_visible(false);
                        IN_MACRO_MODE.store(false, Ordering::Relaxed);
                    }
                }
            } else {
                if !IN_MACRO_MODE.load(Ordering::Relaxed) {
                    macro_hint.set_visible(false);
                    e.set_css_classes(&[]);
                    IN_MACRO_MODE.store(false, Ordering::Relaxed);
                }
            }
            if !IN_MACRO_MODE.load(Ordering::Relaxed) {
                let _ = tomanager.send(search::SearchEvent::Term(term.clone()));
                let _ = to_plugins.send(plugin::MessageToPlugins::Term(term));
            }
        }
    ));

    input_container.append(&macro_hint);
    input_container.append(&entry);

    let shell = gtk::Box::builder()
        .hexpand(true)
        .vexpand(false)
        .name("shell")
        .css_name("shell")
        .orientation(gtk::Orientation::Vertical)
        .build();

    let scroll_container = gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .height_request(405)
        .max_content_height(405)
        .min_content_height(0)
        .css_name("resultBox")
        .build();

    let result_box = gtk::Box::builder()
        .hexpand(true)
        .orientation(gtk::Orientation::Vertical)
        .build();

    scroll_container.set_child(Some(&result_box));
    scroll_container.set_visible(false);

    shell.append(&input_container);
    shell.append(&scroll_container);
    window.set_child(Some(&shell));

    let clear_results = glib::clone!(
        #[strong]
        result_box,
        #[strong]
        scroll_container,
        #[strong]
        window,
        move || {
            while let Some(child) = result_box.first_child() {
                result_box.remove(&child);
            }
            result_box.set_css_classes(&[]);
            scroll_container.set_visible(false);
            window.queue_resize();
        }
    );

    let show_math = glib::clone!(
        #[strong]
        result_box,
        #[strong]
        scroll_container,
        #[strong]
        window,
        move |res: f64| {
            scroll_container.set_visible(true);
            let math_box = gtk::Box::builder()
                .css_name("mathResult")
                .hexpand(true)
                .height_request(395)
                .orientation(gtk::Orientation::Vertical)
                .build();
            let head_box = gtk::Box::builder()
                .css_classes(["head"])
                .hexpand(true)
                .spacing(5)
                .halign(gtk::Align::Center)
                .build();
            let title = gtk::Label::builder()
                .hexpand(true)
                .halign(gtk::Align::Start)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build();

            title.set_text(&t!("expr_eval").to_string());
            let head_icon = gtk::Image::builder()
                .pixel_size(12)
                .gicon(&icons::get_icon("plus-symbolic"))
                .build();
            head_icon.set_css_classes(&["eval_icon"]);

            head_box.append(&head_icon);
            head_box.append(&title);

            let answer_box = gtk::Box::builder()
                .hexpand(true)
                .vexpand(true)
                .css_classes(["answer_box"])
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .build();

            let answer = gtk::Label::builder()
                .css_classes(["answer"])
                .halign(gtk::Align::Center)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build();
            answer.set_text(&format!("{res}"));
            answer_box.append(&answer);

            math_box.append(&head_box);
            math_box.append(&answer_box);

            result_box.append(&math_box);
            window.queue_resize();
        }
    );

    let add_entries = glib::clone!(
        #[strong]
        result_box,
        #[strong]
        tomanager,
        #[strong]
        scroll_container,
        #[strong]
        window,
        move |entries: Vec<app::AppEntry>| {
            let entries_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(2)
                .build();
            if !entries.is_empty() {
                scroll_container.set_visible(true);
                let title = gtk::Label::builder()
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .css_name("title")
                    .build();

                title.set_label(&t!("apps").to_string());
                entries_box.append(&title);
            }

            for entry in entries {
                let button = EntryButton(&config, entry, &tomanager);
                entries_box.append(&button);
            }

            result_box.append(&entries_box);
            window.queue_resize();
        }
    );

    if !opts.silent {
        window.present();
    }

    {
        glib::spawn_future_local(glib::clone!(async move {
            while let Ok(ev) = frommanager.recv().await {
                match ev {
                    search::ManagerEvent::DisplayEntries(entries) => add_entries(entries),
                    search::ManagerEvent::Mathematic(res) => show_math(res),
                    search::ManagerEvent::Clear => clear_results(),
                    search::ManagerEvent::Close => {
                        window.close();
                    }
                }
            }
        }));
    }
}

fn load_css(css: String, previous_provider: Option<gtk::CssProvider>) {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&css);

    if let Some(previous_provider) = previous_provider {
        gtk::style_context_remove_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display."),
            &previous_provider,
        );
    }

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn main() {
    if bus::app_is_running() {
        bus::send_represent_event();
    } else {
        let rt = Runtime::new().expect("Unable to create Runtime");
        let _enter = rt.enter();
        rust_i18n::set_locale(&locale::get_locale());
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_thread_ids(true)
            .with_timer(tracing_subscriber::fmt::time::time())
            .init();

        gtk::init().expect("Unable to init gtk");

        let application = Application::new(Some(conf::APP_ID), Default::default());
        let config_file_path = conf::init_config_dir();
        let config_dir = config_file_path.parent().unwrap();
        let mut pl = plugin::PluginLoader::new(config_dir);
        pl.lookup();

        let to_plugins = pl.sx.clone();

        std::thread::spawn(move || {
            pl.start();
        });

        application.connect_activate(move |app| {
            let mut opts = StartupOptions::default();
            opts.read_args();

            let config = conf::Config::parse(config_file_path.clone());
            load_css(config.css.clone(), None);

            activate(config.clone(), app, opts, to_plugins.clone());
        });

        application.add_main_option(
            "silent",
            's'.try_into().unwrap(),
            gtk::glib::OptionFlags::NONE,
            gtk::glib::OptionArg::None,
            &t!("silent_opt").to_string(),
            None,
        );

        application.run();
    }
}

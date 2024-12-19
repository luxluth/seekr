use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use conf::MacroDef;
use gtk::glib;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use rust_i18n::t;
use search::SearchManager;
use tokio::runtime::Runtime;
use ui::entry_button::EntryButton;

mod app;
mod bus;
mod conf;
mod icons;
mod locale;
mod resources;
mod search;
mod ui;

rust_i18n::i18n!("locales", fallback = "en");

static IN_MACRO_MODE: AtomicBool = AtomicBool::new(false);

fn activate(config: conf::Config, app: &Application) {
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

    if let Ok(xdg_current_desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        if xdg_current_desktop.to_lowercase() == "gnome" {
            window.add_css_class("gnome");
        }
    }

    window.set_default_size(600, -1);

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
                if let Some(def) = config.macros.get(macro_hint.text().as_str()) {
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
                let _ = tomanager.send(search::SearchEvent::Term(term));
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
        move || {
            while let Some(child) = result_box.first_child() {
                result_box.remove(&child);
            }
            result_box.set_css_classes(&[]);
            scroll_container.set_visible(false);
        }
    );

    let show_math = glib::clone!(
        #[strong]
        result_box,
        #[strong]
        scroll_container,
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
        }
    );

    let add_entries = glib::clone!(
        #[strong]
        result_box,
        #[strong]
        tomanager,
        #[strong]
        scroll_container,
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
        }
    );

    window.present();

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

        let config = conf::Config::parse(conf::init_config_dir());

        gtk::init().expect("Unable to init gtk");
        load_css(config.css.clone(), None);

        let application = Application::new(Some(conf::APP_ID), Default::default());

        application.connect_activate(move |app| {
            activate(config.clone(), app);
        });

        application.run();
    }
}

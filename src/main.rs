// use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use search::SearchManager;

mod app;
mod bus;
mod conf;
mod icons;
mod resources;
mod search;
mod ui;

fn activate(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("seekr")
        .css_name("window")
        .resizable(false)
        .decorated(false)
        .hide_on_close(true)
        .build();

    window.set_default_size(600, -1);

    let (manager, (tomanager, frommanager)) = SearchManager::new();
    manager.manage();

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
        .spacing(5)
        .css_name("inputBox")
        .name("inputBox")
        .build();

    // let search_icon = include_str!("assets/search.svg").as_bytes();
    // let cursor = gtk::gio::MemoryInputStream::from_bytes(&glib::Bytes::from(search_icon));
    // let loader =
    //     Pixbuf::from_stream_at_scale(&cursor, 48, 48, true, None::<&gtk::gio::Cancellable>)
    //         .unwrap();
    // let icon = gtk::gio::Icon::from(loader);
    // let search_icon = gtk::Image::from_gicon(&icon);
    // search_icon.set_css_classes(&["search_icon"]);
    //
    // input_container.append(&search_icon);

    let entry = gtk::Entry::builder()
        .hexpand(true)
        .css_name("input")
        .activates_default(true)
        .placeholder_text("Search with Seekr")
        .build();

    entry.connect_changed(glib::clone!(
        #[strong]
        tomanager,
        move |e| {
            let term = e.text().to_string();
            if !term.is_empty() {
                e.set_css_classes(&["has_input"])
            } else {
                e.set_css_classes(&[])
            }
            let _ = tomanager.send(search::SearchEvent::Term(term));
        }
    ));

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
    #[allow(deprecated)]
    scroll_container.hide();

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
            #[allow(deprecated)]
            scroll_container.hide();
        }
    );

    let show_math = glib::clone!(
        #[strong]
        result_box,
        #[strong]
        scroll_container,
        move |res: f64| {
            #[allow(deprecated)]
            scroll_container.show();
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

            // TODO: localize
            title.set_text("Expression evaluation");
            let head_icon = gtk::Image::from_gicon(resources::DIVIDE_ICON.get());
            head_icon.set_css_classes(&["search_icon"]);

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
                #[allow(deprecated)]
                scroll_container.show();
                let title = gtk::Label::builder()
                    .hexpand(true)
                    .halign(gtk::Align::Start)
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .css_name("title")
                    .build();

                // TODO: localize
                title.set_label(&format!("Applications"));
                entries_box.append(&title);
            }

            for entry in entries {
                let button = ui::EntryButton(entry, &tomanager);
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

fn load_css() {
    gtk::init().expect("Unable to init gtk");
    let provider = gtk::CssProvider::new();
    let css = include_str!("./style.css");
    provider.load_from_string(css);

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .with_timer(tracing_subscriber::fmt::time::time())
        .init();
    if bus::app_is_running() {
        bus::send_represent_event();
    } else {
        load_css();

        let application = Application::new(Some(conf::APP_ID), Default::default());

        application.connect_activate(move |app| {
            activate(&app);
        });

        application.run();
    }
}

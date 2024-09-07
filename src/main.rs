use gtk::prelude::*;
use gtk::Entry;
use gtk::{glib, Box};
use gtk::{Application, ApplicationWindow};
use search::SearchManager;

mod bus;
mod conf;
mod search;

fn activate(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("seekr")
        .width_request(900)
        .height_request(60)
        .css_name("window")
        .decorated(false)
        .hide_on_close(true)
        .build();

    let (manager, (tomanager, frommanager)) = SearchManager::new();
    manager.manage();

    let represent_action = gtk::gio::SimpleAction::new("represent", None);
    represent_action.connect_activate(glib::clone!(
        #[weak]
        window,
        move |_, _| {
            window.present();
        }
    ));
    window.add_action(&represent_action);

    let input_continer = Box::builder()
        .height_request(60)
        .hexpand(true)
        .css_name("inputBox")
        .build();

    let entry = Entry::builder()
        .height_request(60)
        .hexpand(true)
        .vexpand(true)
        .css_name("input")
        .placeholder_text("Search with Seekr")
        .activates_default(true)
        .build();

    entry.connect_changed(move |e| {
        let term = e.text().to_string();
        let _ = tomanager.send(search::SearchEvent::Term(term));
    });

    input_continer.append(&entry);
    window.set_child(Some(&input_continer));

    {
        glib::spawn_future_local(glib::clone!(async move {
            while let Ok(ev) = frommanager.recv() {
                match ev {}
            }
        }));
    }

    window.present();
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

fn main() {
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

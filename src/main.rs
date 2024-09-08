use gtk::prelude::*;
use gtk::Entry;
use gtk::{glib, Box};
use gtk::{Application, ApplicationWindow};
use search::SearchManager;

mod app;
mod bus;
mod conf;
mod icons;
mod search;

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
    let to_manager_clone = tomanager.clone();
    represent_action.connect_activate(glib::clone!(
        #[weak]
        window,
        move |_, _| {
            let _ = to_manager_clone.send(search::SearchEvent::Represent);
            window.present();
        }
    ));
    window.add_action(&represent_action);

    let input_continer = Box::builder()
        .height_request(60)
        .hexpand(true)
        .css_name("inputBox")
        .name("inputBox")
        .build();

    let entry = Entry::builder()
        .hexpand(true)
        .css_name("input")
        .placeholder_text("Search with Seekr")
        .build();

    entry.connect_changed(move |e| {
        let term = e.text().to_string();
        if !term.is_empty() {
            e.set_css_classes(&["has_input"])
        } else {
            e.set_css_classes(&[])
        }
        let _ = tomanager.send(search::SearchEvent::Term(term));
    });

    input_continer.append(&entry);

    let shell = Box::builder()
        .hexpand(true)
        .vexpand(false)
        .name("shell")
        .css_name("shell")
        .orientation(gtk::Orientation::Vertical)
        .build();

    let entries_result_box = Box::builder()
        .hexpand(true)
        .name("entryBoxResults")
        .orientation(gtk::Orientation::Vertical)
        .css_name("resultBox")
        .build();

    shell.append(&input_continer);
    shell.append(&entries_result_box);
    window.set_child(Some(&shell));

    let clear_results = glib::clone!(
        #[strong]
        entries_result_box,
        move || {
            while let Some(child) = entries_result_box.first_child() {
                entries_result_box.remove(&child);
            }
            entries_result_box.set_css_classes(&[]);
        }
    );

    let update_results = glib::clone!(
        #[strong]
        entries_result_box,
        move |entries: Vec<app::AppEntry>, term: String| {
            if !entries.is_empty() {
                entries_result_box.set_css_classes(&["filled"]);
            } else {
                entries_result_box.set_css_classes(&[]);
            }

            let tip = gtk::Label::builder()
                .hexpand(true)
                .halign(gtk::Align::Start)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .css_name("tip")
                .build();

            // TODO: localize
            tip.set_label(&format!("search results for '{term}'"));
            entries_result_box.append(&tip);

            for entry in entries {
                let entry_button = gtk::Button::builder()
                    .vexpand(false)
                    .hexpand(true)
                    .can_focus(true)
                    .focus_on_click(true)
                    .sensitive(true)
                    .css_name("entry")
                    .name("Entry")
                    .build();

                let focus_controller = gtk::EventControllerFocus::new();
                focus_controller.set_name(Some("gtk-box-focus-controller"));

                let gesture = gtk::GestureClick::new();
                let key_controller = gtk::EventControllerKey::new();

                focus_controller.connect_enter(glib::clone!(
                    #[weak]
                    entry_button,
                    move |_| {
                        entry_button.add_css_class("focused");
                    }
                ));

                focus_controller.connect_leave(glib::clone!(
                    #[weak]
                    entry_button,
                    move |_| {
                        entry_button.remove_css_class("focused");
                    }
                ));

                let en = entry.clone();
                gesture.connect_released(move |gesture, _, _, _| {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    en.launch(None, None);
                });

                let en = entry.clone();
                key_controller.connect_key_pressed(move |_, key, _, _| match key {
                    gtk::gdk::Key::Return | gtk::gdk::Key::space => {
                        en.launch(None, None);
                        gtk::glib::Propagation::Proceed
                    }
                    _ => gtk::glib::Propagation::Proceed,
                });

                entry_button.add_controller(gesture);
                entry_button.add_controller(key_controller);
                entry_button.add_controller(focus_controller);
                entry_button.set_focusable(true);

                let icon_image = gtk::Image::builder().css_name("entryIcon").build();
                icon_image.set_from_file(match icons::get_icon(&entry.icon) {
                    Some(path) => Some(path),
                    None => icons::get_icon("application-x-executable"),
                });

                let name = gtk::Label::builder()
                    .css_name("entryName")
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .halign(gtk::Align::Start)
                    .build();
                name.set_label(&entry.name);

                let desc = gtk::Label::builder()
                    .css_name("entryDescription")
                    .halign(gtk::Align::Start)
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .build();
                desc.set_label(&entry.description);

                let labels = gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .hexpand(true)
                    .valign(gtk::Align::Center)
                    .halign(gtk::Align::Start)
                    .css_name("entryLabels")
                    .build();

                labels.append(&name);
                labels.append(&desc);

                let content_box = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .spacing(10)
                    .hexpand(true)
                    .valign(gtk::Align::Center)
                    .halign(gtk::Align::Start)
                    .build();
                content_box.append(&icon_image);
                content_box.append(&labels);

                entry_button.set_child(Some(&content_box));

                entries_result_box.append(&entry_button);
            }
        }
    );

    {
        glib::spawn_future_local(glib::clone!(async move {
            while let Ok(ev) = frommanager.recv().await {
                match ev {
                    search::ManagerEvent::DisplayEntries((entries, term)) => {
                        update_results(entries, term)
                    }
                    search::ManagerEvent::Clear => clear_results(),
                }
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

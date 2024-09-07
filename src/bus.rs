use crate::conf::APP_ID;
use dbus::{
    arg::{PropMap, Variant},
    blocking::Connection,
};
use std::time::Duration;

pub fn app_is_running() -> bool {
    // chech the dbus for the APP_ID
    let c = Connection::new_session();
    match c {
        Ok(conn) => {
            let proxy = conn.with_proxy("org.freedesktop.DBus", "/", Duration::from_millis(5000));
            let (names,): (Vec<String>,) = proxy
                .method_call("org.freedesktop.DBus", "ListNames", ())
                .unwrap();
            for name in names {
                if name.contains(APP_ID) {
                    return true;
                }
            }

            false
        }

        Err(_) => {
            println!("Could not connect to dbus.");
            false
        }
    }
}

pub fn send_represent_event() {
    let c = Connection::new_session();
    match c {
        Ok(conn) => {
            // try to send a signal to the app
            let proxy = conn.with_proxy(
                APP_ID,
                "/dev/luxluth/fsearch/window/1",
                Duration::from_millis(5000),
            );

            let _: () = proxy
                .method_call::<(), (&str, Vec<Variant<String>>, PropMap), &str, &str>(
                    "org.gtk.Actions",
                    "Activate",
                    ("represent", vec![], PropMap::new()),
                )
                .unwrap();
        }

        Err(_) => {
            println!("Could not connect to dbus.");
        }
    }
}

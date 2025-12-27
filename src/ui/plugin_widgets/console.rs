use gtk::glib;
use gtk::prelude::*;
use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::thread;

pub struct Console {
    pub widget: gtk::Box,
}

impl Console {
    pub fn new(command: &str) -> Self {
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(5)
            .css_name("consoleBox")
            .build();

        let buffer = gtk::TextBuffer::new(None);
        let view = gtk::TextView::builder()
            .buffer(&buffer)
            .editable(false)
            .css_classes(["console_view"])
            .monospace(true)
            .wrap_mode(gtk::WrapMode::WordChar)
            .height_request(200) // "Shown Last 10 lines" - initial height approximation
            .build();

        let scrolled = gtk::ScrolledWindow::builder()
            .child(&view)
            .height_request(200)
            .max_content_height(400)
            .propagate_natural_height(true)
            .build();

        container.append(&scrolled);

        // Execute command
        let cmd_string = command.to_string();
        let (tx, rx) = async_channel::unbounded::<String>();

        // Sender for glib main loop
        glib::spawn_future_local(glib::clone!(
            #[strong]
            buffer,
            async move {
                while let Ok(line) = rx.recv().await {
                    let mut iter = buffer.end_iter();
                    buffer.insert(&mut iter, &format!("{}\n", line));

                    // Scroll to bottom?
                    // view.scroll_to_iter(&iter, 0.0, false, 0.0, 1.0);
                }
            }
        ));

        thread::spawn(move || {
            let mut cmd = std::process::Command::new("sh");
            cmd.arg("-c");
            cmd.arg(&cmd_string);

            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped()); // Capture stderr too?

            if let Ok(mut child) = cmd.spawn() {
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(l) = line {
                            let _ = tx.send_blocking(l);
                        }
                    }
                }
            } else {
                let _ = tx.send_blocking(format!("Failed to execute: {}", cmd_string));
            }
        });

        Self { widget: container }
    }
}

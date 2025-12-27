use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;

struct SendPixbuf(gtk::gdk_pixbuf::Pixbuf);
unsafe impl Send for SendPixbuf {}

pub struct ImageGrid {
    pub widget: gtk::Box,
}

impl ImageGrid {
    pub fn new<F>(images: Vec<String>, subtitle: Option<String>, on_click: F) -> Self
    where
        F: Fn(String) + 'static + Clone,
    {
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .css_name("imageGrid")
            .spacing(10)
            .build();

        if let Some(text) = subtitle {
            let label = gtk::Label::builder()
                .label(&text)
                .halign(gtk::Align::Start)
                .css_classes(["subtitle", "title-2"])
                .build();
            container.append(&label);
        }

        let flowbox = gtk::FlowBox::builder()
            .valign(gtk::Align::Start)
            .max_children_per_line(5)
            .min_children_per_line(3)
            .selection_mode(gtk::SelectionMode::None)
            .hexpand(true)
            .vexpand(true)
            .build();

        container.append(&flowbox);

        let (tx, rx) = async_channel::unbounded();

        // Spawn a thread to load images
        std::thread::spawn(move || {
            for path in images {
                // Load at 200x200 (2x scale for 100x100 display)
                if let Ok(pixbuf) =
                    gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(&path, 200, 200, true)
                {
                    let _ = tx.send_blocking((path, SendPixbuf(pixbuf)));
                }
            }
        });

        glib::spawn_future_local(glib::clone!(
            #[strong]
            flowbox,
            async move {
                while let Ok((path, SendPixbuf(pixbuf))) = rx.recv().await {
                    #[allow(deprecated)]
                    let texture = gdk::Texture::for_pixbuf(&pixbuf);

                    let picture = gtk::Picture::for_paintable(&texture);
                    picture.set_content_fit(gtk::ContentFit::Cover);
                    picture.set_width_request(100);
                    picture.set_height_request(100);

                    let button = gtk::Button::builder()
                        .child(&picture)
                        .css_classes(["flat", "image_grid_item"])
                        .build();

                    let on_click_clone = on_click.clone();
                    let path_clone = path.clone();

                    button.connect_clicked(move |_| {
                        on_click_clone(path_clone.clone());
                    });

                    flowbox.append(&button);
                }
            }
        ));

        Self { widget: container }
    }
}

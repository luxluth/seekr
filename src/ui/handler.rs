use crate::plugin::{MessageToPlugins, PluginUiEvent};
use crate::ui::plugin_widgets;
use gtk::glib;
use gtk::prelude::*;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub fn setup(
    rx_ui: async_channel::Receiver<PluginUiEvent>,
    to_plugins: Sender<MessageToPlugins>,
    result_box: gtk::Box,
    scroll_container: gtk::ScrolledWindow,
    window: gtk::ApplicationWindow,
) {
    let plugins_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .name("pluginsBox")
        .css_name("pluginsBox")
        .build();

    glib::spawn_future_local(glib::clone!(
        #[strong]
        plugins_box,
        #[strong]
        scroll_container,
        #[strong]
        window,
        #[strong]
        to_plugins,
        #[strong]
        rx_ui,
        async move {
            let mut count = 0;
            while let Ok(event) = rx_ui.recv().await {
                match event {
                    PluginUiEvent::ShowImageGrid {
                        plugin_name,
                        images,
                        subtitle,
                    } => {
                        tracing::debug!(
                            "ShowImageGrid: {} images for {}",
                            images.len(),
                            plugin_name
                        );
                        scroll_container.set_visible(true);

                        let to_plugins_clone = to_plugins.clone();
                        let plugin_name_clone = plugin_name.clone();

                        let grid =
                            plugin_widgets::ImageGrid::new(images, subtitle, move |payload| {
                                let _ = to_plugins_clone.send(MessageToPlugins::Activate {
                                    plugin_name: plugin_name_clone.clone(),
                                    payload,
                                });
                            });

                        grid.widget.add_css_class(&plugin_name);

                        plugins_box.append(&grid.widget);
                        result_box.append(&plugins_box);
                    }
                    PluginUiEvent::ShowInfoBox {
                        plugin_name,
                        title,
                        body,
                    } => {
                        scroll_container.set_visible(true);
                        let info = plugin_widgets::InfoBox::new(&title, &body);
                        info.widget.add_css_class(&plugin_name);
                        plugins_box.append(&info.widget);
                        result_box.append(&plugins_box);
                    }
                    PluginUiEvent::ShowConsole {
                        plugin_name,
                        command,
                    } => {
                        scroll_container.set_visible(true);
                        let console = plugin_widgets::Console::new(&command);
                        console.widget.add_css_class(&plugin_name);
                        plugins_box.append(&console.widget);
                        result_box.append(&plugins_box);
                    }
                    PluginUiEvent::Clear { plugin_name } => {
                        tracing::debug!("Clearing results for plugin: {}", plugin_name);
                        while let Some(child) = plugins_box.first_child() {
                            plugins_box.remove(&child);
                        }
                    }
                }

                window.queue_resize();

                count += 1;
                // Yield to main loop every 5 items to keep UI responsive
                if count % 5 == 0 {
                    glib::timeout_future(Duration::from_millis(1)).await;
                }
            }
        }
    ));
}

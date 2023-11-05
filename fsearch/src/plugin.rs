use crate::gtk;
use crate::gtk::prelude::*;
use crate::utils::{get_section_title, wrap_section};
use fsearch_core::{
    get_scripts_dir, Align, DataType, Element, Orientation, PluginActionType as Act, PluginConfig,
    PluginResponse,
};

pub fn elem_to_gtk_widget(component: Element) -> gtk::Widget {
    let elem_type = component.element_type;
    match elem_type {
        DataType::Box => {
            let mut box_ = gtk::Box::builder();
            box_ = box_.css_name(component.id);
            box_ = box_.css_classes(component.classes);
            let orientation = component.orientation;
            match orientation {
                Some(orientation) => match orientation {
                    Orientation::Horizontal => {
                        box_ = box_.orientation(gtk::Orientation::Horizontal);
                    }
                    Orientation::Vertical => {
                        box_ = box_.orientation(gtk::Orientation::Vertical);
                    }
                },
                None => {}
            }

            let hexpand = component.hexpand;
            match hexpand {
                Some(hexpand) => {
                    box_ = box_.hexpand(hexpand);
                }
                None => {}
            }

            let vexpand = component.vexpand;
            match vexpand {
                Some(vexpand) => {
                    box_ = box_.vexpand(vexpand);
                }
                None => {}
            }

            let halign = component.halign;
            match halign {
                Some(halign) => match halign {
                    Align::Start => {
                        box_ = box_.halign(gtk::Align::Start);
                    }
                    Align::End => {
                        box_ = box_.halign(gtk::Align::End);
                    }
                    Align::Center => {
                        box_ = box_.halign(gtk::Align::Center);
                    }
                    Align::Fill => {
                        box_ = box_.halign(gtk::Align::Fill);
                    }
                    Align::Baseline => {
                        box_ = box_.halign(gtk::Align::Baseline);
                    }
                },
                None => {}
            }

            let box_ = box_.build();
            match component.children {
                Some(children) => {
                    for child in children {
                        let child_widget = elem_to_gtk_widget(child);
                        box_.append(&child_widget);
                    }
                }
                None => {}
            }

            box_.into()
        }

        DataType::EventBox => {
            let mut event_box = gtk::Box::builder();
            event_box = event_box.css_name(component.id);
            event_box = event_box.css_classes(component.classes);
            event_box = event_box.sensitive(true).focusable(true);

            let hexpand = component.hexpand;
            match hexpand {
                Some(hexpand) => {
                    event_box = event_box.hexpand(hexpand);
                }
                None => {}
            }

            let vexpand = component.vexpand;
            match vexpand {
                Some(vexpand) => {
                    event_box = event_box.vexpand(vexpand);
                }
                None => {}
            }

            let halign = component.halign;
            match halign {
                Some(halign) => match halign {
                    Align::Start => {
                        event_box = event_box.halign(gtk::Align::Start);
                    }
                    Align::End => {
                        event_box = event_box.halign(gtk::Align::End);
                    }
                    Align::Center => {
                        event_box = event_box.halign(gtk::Align::Center);
                    }
                    Align::Fill => {
                        event_box = event_box.halign(gtk::Align::Fill);
                    }
                    Align::Baseline => {
                        event_box = event_box.halign(gtk::Align::Baseline);
                    }
                },
                None => {}
            }

            let event_box = event_box.build();
            match component.children {
                Some(children) => {
                    for child in children {
                        let child_widget = elem_to_gtk_widget(child);
                        event_box.append(&child_widget);
                    }
                }
                None => {}
            }

            event_box.connect_notify(Some("click"), move |_, _| {
                println!("Clicked!");
            });

            event_box.into()
        }

        DataType::Button => {
            let mut button = gtk::Button::builder();
            button = button.css_name(component.id);
            button = button.css_classes(component.classes);
            match component.text {
                Some(text) => {
                    button = button.label(text);
                }
                None => {}
            }

            let hexpand = component.hexpand;
            match hexpand {
                Some(hexpand) => {
                    button = button.hexpand(hexpand);
                }
                None => {}
            }

            let vexpand = component.vexpand;
            match vexpand {
                Some(vexpand) => {
                    button = button.vexpand(vexpand);
                }
                None => {}
            }

            let halign = component.halign;
            match halign {
                Some(halign) => match halign {
                    Align::Start => {
                        button = button.halign(gtk::Align::Start);
                    }
                    Align::End => {
                        button = button.halign(gtk::Align::End);
                    }
                    Align::Center => {
                        button = button.halign(gtk::Align::Center);
                    }
                    Align::Fill => {
                        button = button.halign(gtk::Align::Fill);
                    }
                    Align::Baseline => {
                        button = button.halign(gtk::Align::Baseline);
                    }
                },
                None => {}
            }

            let button = button.build();
            button.into()
        }
        DataType::Label => {
            let mut label = gtk::Label::builder();
            label = label.css_name(component.id);
            label = label.css_classes(component.classes);
            match component.text {
                Some(text) => {
                    label = label.label(text);
                }
                None => {}
            }

            let hexpand = component.hexpand;
            match hexpand {
                Some(hexpand) => {
                    label = label.hexpand(hexpand);
                }
                None => {}
            }

            let vexpand = component.vexpand;
            match vexpand {
                Some(vexpand) => {
                    label = label.vexpand(vexpand);
                }
                None => {}
            }

            let halign = component.halign;
            match halign {
                Some(halign) => match halign {
                    Align::Start => {
                        label = label.halign(gtk::Align::Start);
                    }
                    Align::End => {
                        label = label.halign(gtk::Align::End);
                    }
                    Align::Center => {
                        label = label.halign(gtk::Align::Center);
                    }
                    Align::Fill => {
                        label = label.halign(gtk::Align::Fill);
                    }
                    Align::Baseline => {
                        label = label.halign(gtk::Align::Baseline);
                    }
                },
                None => {}
            }

            match component.wrap {
                Some(wrap) => {
                    label = label.wrap(wrap);
                }
                None => {}
            }

            match component.ellipsize {
                Some(ellipsize) => {
                    if ellipsize {
                        label = label.ellipsize(gtk::pango::EllipsizeMode::End);
                    }
                }
                None => {}
            }

            let label = label.build();
            label.into()
        }

        DataType::Image => {
            let mut image = gtk::Image::builder();
            image = image.css_name(component.id);
            image = image.css_classes(component.classes);

            match component.image_path {
                Some(image_path) => {
                    image = image.file(image_path);
                }
                None => {}
            }

            let hexpand = component.hexpand;
            match hexpand {
                Some(hexpand) => {
                    image = image.hexpand(hexpand);
                }
                None => {}
            }

            let vexpand = component.vexpand;
            match vexpand {
                Some(vexpand) => {
                    image = image.vexpand(vexpand);
                }
                None => {}
            }

            let halign = component.halign;
            match halign {
                Some(halign) => match halign {
                    Align::Start => {
                        image = image.halign(gtk::Align::Start);
                    }
                    Align::End => {
                        image = image.halign(gtk::Align::End);
                    }
                    Align::Center => {
                        image = image.halign(gtk::Align::Center);
                    }
                    Align::Fill => {
                        image = image.halign(gtk::Align::Fill);
                    }
                    Align::Baseline => {
                        image = image.halign(gtk::Align::Baseline);
                    }
                },
                None => {}
            }

            let image = image.build();

            image.into()
        }
    }
}

pub fn execute_plugin(
    plugin: &PluginConfig,
    input: String,
) -> (Option<gtk::Box>, Option<Act>, Option<String>) {
    let cmd_to_exec = &*plugin.cmd.as_str();
    if cmd_to_exec.is_empty() {
        return (None, None, None);
    }

    let mut cmd_path = String::from(cmd_to_exec);

    if cmd_to_exec.starts_with("@script:") {
        let script_name = cmd_to_exec.replace("@script:", "");
        let scripts_dir = get_scripts_dir();

        if scripts_dir.len() == 0 {
            return (None, None, None);
        }

        cmd_path = String::from(format!("{}/{}", scripts_dir, script_name));
    }

    let output = std::process::Command::new(cmd_path)
        .arg(input)
        .output()
        .expect("Failed to execute script.");

    let output = String::from_utf8(output.stdout).unwrap();
    let output = output.trim();
    if output.is_empty() {
        return (None, None, None);
    }
    let output = serde_json::from_str::<PluginResponse>(output);
    let mut icon = None;
    match output {
        Ok(output) => {
            let mut elements = Vec::new();
            if output.elements.len() == 0 {
                return (None, None, None);
            }
            for element in output.elements {
                let element = elem_to_gtk_widget(element);
                elements.push(element);
            }

            if let Some(set_icon) = output.set_icon {
                icon = Some(set_icon);
            }

            let content = gtk::Box::builder()
                .name("Content")
                .css_name("Content")
                .orientation(gtk::Orientation::Vertical)
                .hexpand(true)
                .build();

            for element in elements {
                content.append(&element);
            }

            let title = get_section_title(output.title.unwrap_or(plugin.name.clone()));
            let box_content = gtk::Box::builder()
                .name("BoxContent")
                .css_name("BoxContent")
                .orientation(gtk::Orientation::Vertical)
                .build();
            box_content.append(&title);
            box_content.append(&content);

            let section = wrap_section(box_content);

            let action = output.action;
            match action {
                Some(action) => match action.action {
                    Act::Exit => {
                        return (Some(section), Some(Act::Exit), None);
                    }
                    Act::Open(s) => {
                        return (Some(section), Some(Act::Open(s)), icon);
                    }
                    Act::Copy(s) => {
                        return (Some(section), Some(Act::Copy(s)), icon);
                    }
                    Act::Launch(s) => {
                        return (Some(section), Some(Act::Launch(s)), icon);
                    }
                    Act::RunCmd(s) => {
                        return (Some(section), Some(Act::RunCmd(s)), icon);
                    }
                    Act::RunScript(s) => {
                        return (Some(section), Some(Act::RunScript(s)), icon);
                    }
                },
                None => {
                    return (Some(section), None, icon);
                }
            };
        }
        Err(_) => {
            return (None, None, None);
        }
    }
}

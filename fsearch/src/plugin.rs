use fsearch_core::{Element, DataType, Orientation, Align};
use crate::gtk;
use crate::gtk::prelude::*;


pub fn elem_to_gtk_widget(component: Element) -> gtk::Widget {
    let elem_type = component.element_type;
    match elem_type {
        DataType::Box => {
            let mut box_ = gtk::Box::builder();
            box_ = box_.css_name(component.id);
            box_ = box_.css_classes(component.classes);
            let orientation = component.orientation;
            match orientation {
                Some(orientation) => {
                    match orientation {
                        Orientation::Horizontal => {
                            box_ = box_.orientation(gtk::Orientation::Horizontal);
                        },
                        Orientation::Vertical => {
                            box_ = box_.orientation(gtk::Orientation::Vertical);
                        },
                    }
                },
                None => {},
            }

            let hexpand = component.hexpand;
            match hexpand {
                Some(hexpand) => {
                    box_ = box_.hexpand(hexpand);
                },
                None => {},
            }

            let vexpand = component.vexpand;
            match vexpand {
                Some(vexpand) => {
                    box_ = box_.vexpand(vexpand);
                },
                None => {},
            }

            let halign = component.halign;
            match halign {
                Some(halign) => {
                    match halign {
                        Align::Start => {
                            box_ = box_.halign(gtk::Align::Start);
                        },
                        Align::End => {
                            box_ = box_.halign(gtk::Align::End);
                        },
                        Align::Center => {
                            box_ = box_.halign(gtk::Align::Center);
                        },
                        Align::Fill => {
                            box_ = box_.halign(gtk::Align::Fill);
                        },
                        Align::Baseline => {
                            box_ = box_.halign(gtk::Align::Baseline);
                        },
                    }  
                },
                None => {},
            }

            
            let box_ = box_.build();
            match component.children {
                Some(children) => {
                    for child in children {
                        let child_widget = elem_to_gtk_widget(child);
                        box_.append(&child_widget);
                    }
                },
                None => {},
            }           

            box_.into()
        },

        DataType::Button => {
            let mut button = gtk::Button::builder();
            button = button.css_name(component.id);
            button = button.css_classes(component.classes);
            match component.text {
                Some(text) => {
                    button = button.label(text);
                },
                None => {},
            }

            let hexpand = component.hexpand;
            match hexpand {
                Some(hexpand) => {
                    button = button.hexpand(hexpand);
                },
                None => {},
            }

            let vexpand = component.vexpand;
            match vexpand {
                Some(vexpand) => {
                    button = button.vexpand(vexpand);
                },
                None => {},
            }

            let halign = component.halign;
            match halign {
                Some(halign) => {
                    match halign {
                        Align::Start => {
                            button = button.halign(gtk::Align::Start);
                        },
                        Align::End => {
                            button = button.halign(gtk::Align::End);
                        },
                        Align::Center => {
                            button = button.halign(gtk::Align::Center);
                        },
                        Align::Fill => {
                            button = button.halign(gtk::Align::Fill);
                        },
                        Align::Baseline => {
                            button = button.halign(gtk::Align::Baseline);
                        },
                    }  
                },
                None => {},
            }

            let button = button.build();
            button.into()
        },
        DataType::Label => todo!(),
    }
}

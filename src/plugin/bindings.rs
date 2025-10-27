use gtk::{glib, prelude::*};
use mlua::prelude::*;
use roxmltree::Document;

pub struct GtkBinding;

impl LuaUserData for GtkBinding {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {}
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {}
}

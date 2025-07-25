/*
 * Copyright (c) 2003-2003-2023. Trevor Campbell and others.
 *
 * This file is part of Kelpie Flight Planner.
 *
 * Kelpie Flight Planner is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * Kelpie Flight Planner is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Kelpie Flight Planner; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 * Contributors:
 *      Trevor Campbell
 *
 */

use std::ffi::CStr;
use adw::AlertDialog;
use gl::types;
use gtk::{AboutDialog, glib, Label, ListItem, Root, SignalListItemFactory, TreeExpander, TreeListRow, TreeListModel, Widget};
use gtk::gdk::Texture;
use gtk::glib::Object;
use adw::prelude::{AdwDialogExt, AlertDialogExt, Cast, CastNone, EditableExt, EditableExtManual, GtkWindowExt, IsA, ListItemExt, WidgetExt};
use adw::subclass::prelude::ObjectSubclassIsExt;
use gettextrs::gettext;
use crate::util;
use crate::window::airport_map_view::AirportMapView;
use crate::window::airport_view::AirportView;
use crate::window::fix_view::FixView;
use crate::window::navaid_view::NavaidView;
use crate::window::plan_view::PlanView;
use crate::window::Window;
use crate::window::world_map_view::WorldMapView;

// The following allows us to test an entry field for numeric only characters
// To use it
// entry_disallow(&entry, is_numeric);
#[allow(dead_code)]
pub fn is_non_numeric(c: char) -> bool {
    !c.is_numeric()
}

#[allow(dead_code)]
pub fn entry_disallow(entry: &gtk::Entry, pattern: fn(char) -> bool) {
    entry.connect_insert_text(move |entry, text, position| {
        if text.contains(pattern) {
            glib::signal::signal_stop_emission_by_name(entry, "insert-text");
            entry.insert_text(&text.replace(pattern, ""), position);
        }
    });
}

pub fn show_error_dialog(root: &Option<Root>, message: &str) {
    // Create a new message dialog
    if let Ok(w) = root
        .as_ref()
        .expect("Can't get the root window")
        .clone()
        .downcast::<gtk::Window>()
    {
        let message = gettext(message);
        let dialog = AlertDialog::new(None, Some(&*message));
        dialog.add_response("OK", "Ok");
        dialog.present(Some(&w));
    };
}

pub(crate) fn build_column_factory<F: Fn(Label, &T) + 'static, T: IsA<Object>>(f: F) -> SignalListItemFactory {
    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
        let label = Label::new(None);
        list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .set_child(Some(&label));
    });

    factory.connect_bind(move |_, list_item| {
        // Get `StringObject` from `ListItem`
        let obj = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .item()
            .and_downcast::<T>()
            .expect("The item has to be an <T>.");

        // Get `Label` from `ListItem`
        let label = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .child()
            .and_downcast::<Label>()
            .expect("The child has to be a `Label`.");

        // Set "label" to "number"
        f(label, &obj);
    });
    factory
}

pub(crate) fn build_tree_column_factory(f: fn(Label, &TreeListRow)) -> SignalListItemFactory {
    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
        let label = Label::new(None);
        let expander = TreeExpander::new();
        expander.set_child(Some(&label));
        expander.set_indent_for_icon(true);
        expander.set_indent_for_depth(true);
        let list_item = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem");
        list_item.set_child(Some(&expander));
        list_item.set_focusable(false);
    });

    factory.connect_bind(move |_factory, list_item| {
        // Get `StringObject` from `ListItem`
        let obj = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .item()
            .and_downcast::<TreeListRow>()
            .expect("The item has to be an <T>.");

        // Get `Label` from `ListItem`
        if let Some(widget) = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .child() {
            let expander = widget.downcast::<TreeExpander>()
                .expect("The child has to be a `Expander`.");

            let widget = expander.child()
                .expect("The child has to be a `Widget`.");
            let label = widget.downcast::<Label>()
                .expect("The child has to be a `Label`.");
            // Set "label" to "value"
            f(label, &obj);

            expander.set_list_row(Some(&obj));
        }
    });
    factory
}


pub(crate) fn show_help_about(window: &Window) {
    let icon = Texture::from_resource(
        "/com/shartrec/kelpie_planner/images/kelpiedog_120x120_transparent.png");

    let mut builder = Object::builder::<AboutDialog>();

    builder = builder.property("program-name", util::info::PROGRAM_NAME);
    builder = builder.property("version", util::info::VERSION);
    builder = builder.property("website", util::info::WEBSITE);
    builder = builder.property("license-type", util::info::LICENSE_TYPE);
    builder = builder.property("title", util::info::ABOUT_TITLE);
    builder = builder.property("authors", [util::info::AUTHOR].as_ref());
    builder = builder.property("system-information", get_gl_info());
    builder = builder.property("logo", &icon);

    let about_dialog = builder.build();
    about_dialog.set_transient_for(Some(window));
    about_dialog.set_modal(true);
    about_dialog.set_destroy_with_parent(true);

    about_dialog.set_visible(true);
}

fn get_gl_info() -> String {
    let mut gl_info = String::new();
    if let Some(s) = get_gl_string(gl::VERSION) {
        gl_info = gl_info + "Open GL Version : " + s.as_str() + "\n";
    }
    if let Some(s) = get_gl_string(gl::RENDERER) {
        gl_info = gl_info + "Open GL Renderer : " + s.as_str() + "\n";
    }
    gl_info
}


#[allow(unsafe_code)]
pub(crate) fn get_gl_string(name: types::GLenum) -> Option<String> {
    unsafe {
        let string_ptr = gl::GetString(name);
        match *string_ptr {
            0 => None,
            _ => {
                let c_str: &CStr = CStr::from_ptr(string_ptr as *const i8);
                Some(c_str.to_str().unwrap().to_string())
            }
        }
    }
}

pub(crate) fn get_plan_view<W: IsA<Widget>>(widget: &W) -> Option<PlanView> {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            if let Some(page) = our_window.imp().plan_tab_view.selected_page() {
                page.child().downcast::<PlanView>().ok()
            } else {
                None
            }
        }
        None => None,
    }
}

pub(crate) fn get_world_map_view<W: IsA<Widget>>(widget: &W) -> Option<WorldMapView> {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            our_window.imp().world_map_view.try_get()
        }
        None => None,
    }
}

pub(crate) fn show_world_map_view<W: IsA<Widget>>(widget: &W) {
    if let Some(r) = widget.root() {
        let our_window = r.downcast::<Window>().unwrap();
        if let Some(notebook) = our_window.imp().map_notebook.try_get() {
            if let Some(view) = our_window.imp().world_map_view.try_get() {
                let page_num = notebook.page_num(&view);
                notebook.set_current_page(page_num);
            }
        }
    }
}

pub(crate) fn get_airport_map_view<W: IsA<Widget>>(widget: &W) -> Option<AirportMapView> {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            our_window.imp().airport_map_view.try_get()
        }
        None => None,
    }
}

pub(crate) fn show_airport_map_view<W: IsA<Widget>>(widget: &W) {
    if let Some(r) = widget.root() {
        let our_window = r.downcast::<Window>().unwrap();
        if let Some(notebook) = our_window.imp().map_notebook.try_get() {
            if let Some(view) = our_window.imp().airport_map_view.try_get() {
                let page_num = notebook.page_num(&view);
                notebook.set_current_page(page_num);
            }
        }
    }
}

pub(crate) fn get_airport_view<W: IsA<Widget>>(widget: &W) -> Option<AirportView> {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            our_window.imp().airport_view.try_get()
        }
        None => None,
    }
}

pub(crate) fn show_airport_view<W: IsA<Widget>>(widget: &W) {
    if let Some(r) = widget.root() {
        let our_window = r.downcast::<Window>().unwrap();
        if let Some(notebook) = our_window.imp().search_notebook.try_get() {
            if let Some(view) = our_window.imp().airport_view.try_get() {
                let page_num = notebook.page_num(&view);
                notebook.set_current_page(page_num);
            }
        }
    }
}

pub(crate) fn get_navaid_view<W: IsA<Widget>>(widget: &W) -> Option<NavaidView> {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            our_window.imp().navaid_view.try_get()
        }
        None => None,
    }
}

pub(crate) fn show_navaid_view<W: IsA<Widget>>(widget: &W) {
    if let Some(r) = widget.root() {
        let our_window = r.downcast::<Window>().unwrap();
        if let Some(notebook) = our_window.imp().search_notebook.try_get() {
            if let Some(view) = our_window.imp().navaid_view.try_get() {
                let page_num = notebook.page_num(&view);
                notebook.set_current_page(page_num);
            }
        }
    }
}

pub(crate) fn get_fix_view<W: IsA<Widget>>(widget: &W) -> Option<FixView> {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            our_window.imp().fix_view.try_get()
        }
        None => None,
    }
}

pub(crate) fn show_fix_view<W: IsA<Widget>>(widget: &W) {
    if let Some(r) = widget.root() {
        let our_window = r.downcast::<Window>().unwrap();
        if let Some(notebook) = our_window.imp().search_notebook.try_get() {
            if let Some(view) = our_window.imp().fix_view.try_get() {
                let page_num = notebook.page_num(&view);
                notebook.set_current_page(page_num);
            }
        }
    }
}

// Get a tree path vector from a TreeListModel
pub(crate) fn get_tree_path(row: u32, trm: &TreeListModel) -> Vec<u32> {
    let mut path: Vec<u32> = Vec::new();
    // Walk the tree filling in the path vector according to the depth we are at
    for i in 0..=row {
        if let Some(r) = trm.row(i) {
            let path_index = r.depth();
            if path_index as usize >= path.len() {
                path.push(0);
            } else {
                let old_val = path.remove(path_index as usize);
                path.insert(path_index as usize, old_val + 1);
            }
            // clear lower level paths
            path.truncate((path_index + 1) as usize);
        }
    }
    path
}

// Expand the tree represented by TreeListModel to specified depth.
// Root depth is 0
pub(crate) fn expand_tree(trm: &TreeListModel, depth: u32) {
    for i in 0.. {
        if let Some(r) = trm.row(i) {
            if r.depth() <= depth {
                r.set_expanded(true);
            }
        } else {
            break;
        }
    }
}



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

use gtk::{ButtonsType, gdk_pixbuf, glib, MessageDialog, MessageType, Root, ScrolledWindow, show_about_dialog};
use gtk::gdk::Texture;
use gtk::glib::Cast;
use gtk::prelude::{
    CastNone, DialogExtManual, EditableExt, EditableExtManual, GtkWindowExt, WidgetExt,
};
use gtk::subclass::prelude::ObjectSubclassIsExt;

use crate::util;
use crate::window::airport_map_view::AirportMapView;
use crate::window::airport_view::AirportView;
use crate::window::fix_view::FixView;
use crate::window::navaid_view::NavaidView;
use crate::window::plan_view::PlanView;
use crate::window::Window;
use crate::window::world_map_view::WorldMapView;

// The following allows us to test an entry field for numeric only chgaracters
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
        let dialog = MessageDialog::new(
            Some(&w),
            gtk::DialogFlags::MODAL,
            MessageType::Error,
            ButtonsType::Ok,
            message,
        );
        dialog.run_async(|obj, _answer| {
            obj.close();
        });
    };
}



pub fn show_help_about(root: &Option<Root>) {
    // Create a new message dialog
    if let Ok(w) = root
        .as_ref()
        .expect("Can't get the root window")
        .clone()
        .downcast::<gtk::Window>()
    {
        let icon = Texture::from_resource(
            "/com/shartrec/kelpie_planner/images/kelpiedog_120x120_transparent.png");
        show_about_dialog(Some(&w), &[
            ("program-name", &util::info::PROGRAM_NAME),
            ("version", &util::info::VERSION),
            ("website", &util::info::WEBSITE),
            ("license-type", &util::info::LICENSE_TYPE),
            ("title", &util::info::ABOUT_TITLE),
            ("authors", &[util::info::AUTHOR].as_ref()),
            ("logo", &icon)
            ]);
    }
}

pub(crate) fn get_plan_view(widget: &ScrolledWindow) -> Option<PlanView> {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            our_window
                .imp()
                .plan_stack
                .visible_child()
                .and_downcast::<PlanView>()
        }
        None => None,
    }
}
pub(crate) fn get_world_map_view(widget: &ScrolledWindow) -> Option<WorldMapView> {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            our_window.imp().world_map_view.try_get()
        }
        None => None,
    }
}
pub(crate) fn show_world_map_view(widget: &ScrolledWindow) {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            if let Some(notebook) = our_window.imp().map_notebook.try_get() {
                if let Some(view) = our_window.imp().world_map_view.try_get() {
                    let page_num = notebook.page_num(&view);
                    notebook.set_current_page(page_num);
                }
            }
            ()
        }
        None => (),
    }
}pub(crate) fn get_airport_map_view(widget: &ScrolledWindow) -> Option<AirportMapView> {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            our_window.imp().airport_map_view.try_get()
        }
        None => None,
    }
}
pub(crate) fn show_airport_map_view(widget: &ScrolledWindow) {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            if let Some(notebook) = our_window.imp().map_notebook.try_get() {
                if let Some(view) = our_window.imp().airport_map_view.try_get() {
                    let page_num = notebook.page_num(&view);
                    notebook.set_current_page(page_num);
                }
            }
            ()
        }
        None => (),
    }
}

pub(crate) fn get_airport_view(widget: &ScrolledWindow) -> Option<AirportView> {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            our_window.imp().airport_view.try_get()
        }
        None => None,
    }
}
pub(crate) fn show_airport_view(widget: &ScrolledWindow) {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            if let Some(notebook) = our_window.imp().search_notebook.try_get() {
                if let Some(view) = our_window.imp().airport_view.try_get() {
                    let page_num = notebook.page_num(&view);
                    notebook.set_current_page(page_num);
                }
            }
            ()
        }
        None => (),
    }
}
pub(crate) fn get_navaid_view(widget: &ScrolledWindow) -> Option<NavaidView> {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            our_window.imp().navaid_view.try_get()
        }
        None => None,
    }
}
pub(crate) fn show_navaid_view(widget: &ScrolledWindow) {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            if let Some(notebook) = our_window.imp().search_notebook.try_get() {
                if let Some(view) = our_window.imp().navaid_view.try_get() {
                    let page_num = notebook.page_num(&view);
                    notebook.set_current_page(page_num);
                }
            }
            ()
        }
        None => (),
    }
}
pub(crate) fn get_fix_view(widget: &ScrolledWindow) -> Option<FixView> {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            our_window.imp().fix_view.try_get()
        }
        None => None,
    }
}
pub(crate) fn show_fix_view(widget: &ScrolledWindow) {
    match widget.root() {
        Some(r) => {
            let our_window = r.downcast::<Window>().unwrap();
            if let Some(notebook) = our_window.imp().search_notebook.try_get() {
                if let Some(view) = our_window.imp().fix_view.try_get() {
                    let page_num = notebook.page_num(&view);
                    notebook.set_current_page(page_num);
                }
            }
            ()
        }
        None => (),
    }
}

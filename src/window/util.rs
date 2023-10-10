/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */


use gtk::{ButtonsType, glib, MessageDialog, MessageType, Root, ScrolledWindow};
use gtk::glib::Cast;
use gtk::prelude::{CastNone, DialogExtManual, EditableExt, EditableExtManual, GtkWindowExt, WidgetExt};
use gtk::subclass::prelude::ObjectSubclassIsExt;

use crate::window::airport_map_view::AirportMapView;
use crate::window::airport_view::AirportView;
use crate::window::fix_view::FixView;
use crate::window::navaid_view::NavaidView;
use crate::window::plan_view::PlanView;
use crate::window::Window;

// The following allows us to test an entry field for numeric only chgaracters
// To use it
// entry_disallow(&entry, is_numeric);
#[allow(dead_code)]
fn is_non_numeric(c: char) -> bool {
    !c.is_numeric()
}
#[allow(dead_code)]
fn entry_disallow(entry: &gtk::Entry, pattern: fn(char) -> bool) {
    entry.connect_insert_text(move |entry, text, position| {
        if text.contains(pattern) {
            glib::signal::signal_stop_emission_by_name(entry, "insert-text");
            entry.insert_text(&text.replace(pattern, ""), position);
        }
    });
}

pub fn show_error_dialog(root: &Option<Root>, message: &str) {
    // Create a new message dialog
    if let Ok(w) = root.as_ref().expect("Can't get the root window").clone().downcast::<gtk::Window>() {
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

pub(crate) fn get_plan_view(widget: &ScrolledWindow) -> Option<PlanView> {
    match &widget.root() {
        Some(r) => {
            let our_window = r.clone().downcast::<Window>().unwrap();
            our_window.imp().plan_stack.visible_child().and_downcast::<PlanView>()
        }
        None => {
            None
        }
    }
}
pub(crate) fn get_airport_map_view(widget: &ScrolledWindow) -> Option<AirportMapView> {
    match &widget.root() {
        Some(r) => {
            let our_window = r.clone().downcast::<Window>().unwrap();
            our_window.imp().airport_map_view.try_get()
        }
        None => {
            None
        }
    }
}
pub(crate) fn get_airport_view(widget: &ScrolledWindow) -> Option<AirportView> {
    match &widget.root() {
        Some(r) => {
            let our_window = r.clone().downcast::<Window>().unwrap();
            our_window.imp().airport_view.try_get()
        }
        None => {
            None
        }
    }
}
pub(crate) fn show_airport_view(widget: &ScrolledWindow) {
    match &widget.root() {
        Some(r) => {
            let our_window = r.clone().downcast::<Window>().unwrap();
            if let Some(notebook) = our_window.imp().search_notebook.try_get() {
                if let Some(view) = our_window.imp().airport_view.try_get() {
                    let page_num = notebook.page_num(&view);
                    notebook.set_current_page(page_num);
                }
            }
            ()
        }
        None => ()
    }
}
pub(crate) fn get_navaid_view(widget: &ScrolledWindow) -> Option<NavaidView> {
    match &widget.root() {
        Some(r) => {
            let our_window = r.clone().downcast::<Window>().unwrap();
            our_window.imp().navaid_view.try_get()
        }
        None => {
            None
        }
    }
}
pub(crate) fn show_navaid_view(widget: &ScrolledWindow) {
    match &widget.root() {
        Some(r) => {
            let our_window = r.clone().downcast::<Window>().unwrap();
            if let Some(notebook) = our_window.imp().search_notebook.try_get() {
                if let Some(view) = our_window.imp().navaid_view.try_get() {
                    let page_num = notebook.page_num(&view);
                    notebook.set_current_page(page_num);
                }
            }
            ()
        }
        None => ()
    }
}
pub(crate) fn get_fix_view(widget: &ScrolledWindow) -> Option<FixView> {
    match &widget.root() {
        Some(r) => {
            let our_window = r.clone().downcast::<Window>().unwrap();
            our_window.imp().fix_view.try_get()
        }
        None => {
            None
        }
    }
}
pub(crate) fn show_fix_view(widget: &ScrolledWindow) {
    match &widget.root() {
        Some(r) => {
            let our_window = r.clone().downcast::<Window>().unwrap();
            if let Some(notebook) = our_window.imp().search_notebook.try_get() {
                if let Some(view) = our_window.imp().fix_view.try_get() {
                    let page_num = notebook.page_num(&view);
                    notebook.set_current_page(page_num);
                }
            }
            ()
        }
        None => ()
    }
}

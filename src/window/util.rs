/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */


use gtk::{ButtonsType, glib, MessageDialog, MessageType, Root};
use gtk::prelude::{DialogExtManual, EditableExt, EditableExtManual, GtkWindowExt};
use gtk::glib::Cast;

// The following allows us to test an enttry field for numeric only chgaracters
// To use it
// entry_disallow(&entry, is_numeric);
fn is_non_numeric(c: char) -> bool {
    !c.is_numeric()
}
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
        dialog.run_async(|obj, answer| {
            obj.close();
        });
    };
}


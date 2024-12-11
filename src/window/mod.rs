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

// This module uses OpenGL and all of which functions are unsafe by their very nature.
#![allow(unsafe_code)]

use adw::Application;
use glib::Object;
use gtk::{gio, glib};
use gtk::prelude::GtkWindowExt;

mod airport_map_view;
mod airport_view;
mod fix_view;
pub(crate) mod imp;
mod navaid_view;
mod plan_view;
pub(crate) mod util;
mod world_map_view;
pub(crate) mod map_utils;
pub(crate) mod render_gl;
mod sphere_renderer;
mod shoreline_renderer;
mod airport_renderer;
mod navaid_renderer;
mod plan_renderer;
pub mod preferences;
mod aircraft_renderer;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn setup_actions(&self) {}
    pub fn save_window_size(&self) -> Result<(), glib::BoolError> {
        // Get the size of the window
        let size = self.default_size();

        // Set the window state in `settings`
        let pref = crate::preference::manager();
        pref.put("window-width", size.0);
        pref.put("window-height", size.1);
        pref.put("window-is-maximized", self.is_maximized());

        Ok(())
    }

    fn load_window_size(&self) {
        // Get the window state from `settings`
        let pref = crate::preference::manager();

        // Set the size of the window
        if let Some(w) = pref.get::<i32>("window-width") {
            if let Some(h) = pref.get::<i32>("window-height") {
                self.set_default_size(w, h);
            }
        }

        // If the window was maximized when it was closed, maximize it again
        if let Some(is_maximised) = pref.get::<bool>("is-maximized") {
            if is_maximised {
                self.maximize();
            }
        }
    }
}

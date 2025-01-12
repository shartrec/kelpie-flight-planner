/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
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
use std::sync::Arc;

use gtk::glib;
use gtk::prelude::Cast;
use gtk::subclass::prelude::ObjectSubclassIsExt;

use crate::model::waypoint::Waypoint;

// To use the Waypoint in a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct WaypointObject(ObjectSubclass<imp::WaypointObject>);
}

impl WaypointObject {
    pub fn new(waypoint: &Waypoint) -> WaypointObject {
        let obj: WaypointObject = glib::Object::new();
        obj.downcast_ref::<WaypointObject>()
            .expect("The item has to be an <WaypointObject>.")
            .imp().set_waypoint(waypoint.clone());
        obj
    }
}

mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;

    use gtk::{glib, Label};
    use gtk::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass};

    use crate::model::waypoint::Waypoint;

    #[derive(Default)]
    pub struct WaypointObject {
        waypoint: RefCell<Option<Waypoint>>,
        ui: RefCell<Option<Label>>
    }

    impl WaypointObject {
        pub fn set_waypoint(&self, waypoint: Waypoint) {
            self.waypoint.replace(Some(waypoint));
        }

        pub fn waypoint(&self) -> Waypoint {
            self.waypoint.borrow().as_ref().unwrap().clone()
        }

        pub fn set_ui(&self, label: Option<Label>) {
            self.ui.replace(label);
        }

        pub fn ui(&self) -> Option<Label> {
            self.ui.borrow().clone()
        }
    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for WaypointObject {
        const NAME: &'static str = "WaypointObject";
        type Type = super::WaypointObject;
    }

    impl ObjectImpl for WaypointObject {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
}


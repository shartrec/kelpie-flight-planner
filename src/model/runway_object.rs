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
use adw::prelude::Cast;
use adw::subclass::prelude::ObjectSubclassIsExt;

use crate::model::airport::{Airport, Runway};

// To use the Runway in a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct RunwayObject(ObjectSubclass<imp::RunwayObject>);
}

impl RunwayObject {
    pub fn new(runway: &Runway, airport: Arc<Airport>) -> RunwayObject {
        let obj: RunwayObject = glib::Object::new();
        let runway_impl = obj.downcast_ref::<RunwayObject>()
            .expect("The item has to be an <RunwayObject>.")
            .imp();
        runway_impl.set_runway(runway.clone());
        runway_impl.set_airport(airport);
        obj
    }
}

mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;
    use gtk::glib;
    use adw::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass};

    use crate::model::airport::{Airport, Runway};

    #[derive(Default)]
    pub struct RunwayObject {
        runway: RefCell<Option<Runway>>,
        airport: RefCell<Option<Arc<Airport>>>,
    }

    impl RunwayObject {
        pub fn set_runway(&self, runway: Runway) {
            self.runway.replace(Some(runway));
        }
        pub fn runway(&self) -> RefCell<Option<Runway>> {
            self.runway.clone()
        }
        pub fn set_airport(&self, airport: Arc<Airport>) {
            self.airport.replace(Some(airport));
        }
        pub fn airport(&self) -> Arc<Airport> {
            self.airport.borrow().as_ref().unwrap().clone()
        }
    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for RunwayObject {
        const NAME: &'static str = "RunwayObject";
        type Type = super::RunwayObject;
    }

    impl ObjectImpl for RunwayObject {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
}


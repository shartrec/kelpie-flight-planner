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
// This is a list model that wraps the earth::airports static list making

use gtk::gio::ListModel;
use gtk::glib;

// To use the Airports as a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct Airports(ObjectSubclass<imp::Airports>) @implements ListModel;
}

impl Airports {
    pub fn new() -> Airports {
        glib::Object::new()
    }
}

impl Default for Airports {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use gtk::{gio, glib};
    use gtk::glib::Object;
    use gtk::prelude::StaticType;
    use gtk::subclass::prelude::{ListModelImpl, ObjectImpl, ObjectImplExt, ObjectSubclass};

    use crate::earth::get_earth_model;
    use crate::model::airport_object::AirportObject;

    #[derive(Default)]
    pub struct Airports {
        cache: RefCell<HashMap<u32, AirportObject>>,
    }

    impl Airports {}

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Airports {
        const NAME: &'static str = "Airports";
        type Type = super::Airports;
        type Interfaces = (gio::ListModel, );
    }

    impl ObjectImpl for Airports {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl ListModelImpl for Airports {
        fn item_type(&self) -> glib::Type {
            AirportObject::static_type()
        }

        fn n_items(&self) -> u32 {
            get_earth_model().airports.read().expect("Can't get Airport lock").len() as u32
        }

        fn item(&self, position: u32) -> Option<Object> {
            let mut ref_mut = self.cache.borrow_mut();
            ref_mut.get(&position).map(|ao| Object::from(ao.clone())).or_else(|| {
                let binding = get_earth_model().airports
                    .read()
                    .expect("Unable to get a lock on the airports");
                let airport = binding.iter().nth(position as usize);

                airport.map(|airport| {
                    let ao = AirportObject::new(airport);
                    ref_mut.insert(position, ao.clone());
                    Object::from(ao)
                })
            })
        }
    }
}


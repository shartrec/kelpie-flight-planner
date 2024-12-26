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
// This is a list model that wraps the earth::navaids static list making

use gtk::gio::ListModel;
use gtk::glib;

// To use the Navaids as a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct Navaids(ObjectSubclass<imp::Navaids>) @implements ListModel;
}

impl Navaids {
    pub fn new() -> Navaids {
        glib::Object::new()
    }
}

impl Default for Navaids {
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
    use crate::model::navaid_object::NavaidObject;

    #[derive(Default)]
    pub struct Navaids {
        cache: RefCell<HashMap<u32, NavaidObject>>,
    }

    impl Navaids {}

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Navaids {
        const NAME: &'static str = "Navaids";
        type Type = super::Navaids;
        type Interfaces = (gio::ListModel, );
    }

    impl ObjectImpl for Navaids {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl ListModelImpl for Navaids {
        fn item_type(&self) -> glib::Type {
            NavaidObject::static_type()
        }

        fn n_items(&self) -> u32 {
            get_earth_model().navaids.read().expect("Can't get Navaid lock").len() as u32
        }

        fn item(&self, position: u32) -> Option<Object> {
            let mut ref_mut = self.cache.borrow_mut();
            ref_mut.get(&position).map(|ao| Object::from(ao.clone())).or_else(|| {
                let binding = get_earth_model().navaids
                    .read()
                    .expect("Unable to get a lock on the navaids");
                let navaid = binding.iter().nth(position as usize);

                navaid.map(|navaid| {
                    let ao = NavaidObject::new(navaid);
                    ref_mut.insert(position, ao.clone());
                    Object::from(ao)
                })
            })
        }
    }
}


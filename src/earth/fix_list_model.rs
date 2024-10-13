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
// This is a list model that wraps the earth::fixes static list making

use gtk::gio::ListModel;
use gtk::glib;

// To use the Fixes as a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct Fixes(ObjectSubclass<imp::Fixes>) @implements ListModel;
}

impl Fixes {
    pub fn new() -> Fixes {
        glib::Object::new()
    }
}

impl Default for Fixes {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use gtk::{gio, glib};
    use gtk::glib::Object;
    use gtk::prelude::StaticType;
    use gtk::subclass::prelude::{ListModelImpl, ObjectImpl, ObjectImplExt, ObjectSubclass};

    use crate::earth::get_earth_model;
    use crate::model::fix_object::FixObject;

    #[derive(Default)]
    pub struct Fixes {
    }

    impl Fixes {
    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Fixes {
        const NAME: &'static str = "Fixes";
        type Type = super::Fixes;
        type Interfaces = (gio::ListModel, );
    }

    impl ObjectImpl for Fixes {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl ListModelImpl for Fixes {
        fn item_type(&self) -> glib::Type {
            FixObject::static_type()
        }

        fn n_items(&self) -> u32 {
            get_earth_model().fixes.read().expect("Can't get Fix lock").len() as u32
        }

        fn item(&self, position: u32) -> Option<Object> {
            match get_earth_model().fixes
                .read()
                .expect("Unable to get a lock on the fixes")
                .iter().nth(position as usize) {
                Some(fix) => {
                    let ao = FixObject::new(fix);
                    Some(Object::from(ao))
                }

                None => None
            }
        }
    }

}


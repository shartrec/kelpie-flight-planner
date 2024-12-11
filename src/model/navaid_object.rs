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

use crate::model::navaid::Navaid;

// To use the Navaid in a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct NavaidObject(ObjectSubclass<imp::NavaidObject>);
}

impl NavaidObject {
    pub fn new(navaid: &Arc<Navaid>) -> NavaidObject {
        let obj: NavaidObject = glib::Object::new();
        obj.downcast_ref::<NavaidObject>()
            .expect("The item has to be an <NavaidObject>.")
            .imp().set_navaid(navaid.clone());
        obj
    }
}

mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;

    use gtk::glib;
    use gtk::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass};

    use crate::model::navaid::Navaid;

    #[derive(Default)]
    pub struct NavaidObject {
        navaid: RefCell<Option<Arc<Navaid>>>,
    }

    impl NavaidObject {
        pub fn set_navaid(&self, navaid: Arc<Navaid>) {
            self.navaid.replace(Some(navaid));
        }

        pub fn navaid(&self) -> Arc<Navaid> {
            self.navaid.borrow().as_ref().unwrap().clone()
        }
    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for NavaidObject {
        const NAME: &'static str = "NavaidObject";
        type Type = super::NavaidObject;
    }

    impl ObjectImpl for NavaidObject {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
}


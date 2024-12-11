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

use crate::model::fix::Fix;

// To use the Fix in a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct FixObject(ObjectSubclass<imp::FixObject>);
}

impl FixObject {
    pub fn new(fix: &Arc<Fix>) -> FixObject {
        let obj: FixObject = glib::Object::new();
        obj.downcast_ref::<FixObject>()
            .expect("The item has to be an <FixObject>.")
            .imp().set_fix(fix.clone());
        obj
    }
}

mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;

    use gtk::glib;
    use gtk::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass};

    use crate::model::fix::Fix;

    #[derive(Default)]
    pub struct FixObject {
        fix: RefCell<Option<Arc<Fix>>>,
    }

    impl FixObject {
        pub fn set_fix(&self, fix: Arc<Fix>) {
            self.fix.replace(Some(fix));
        }

        pub fn fix(&self) -> Arc<Fix> {
            self.fix.borrow().as_ref().unwrap().clone()
        }
    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for FixObject {
        const NAME: &'static str = "FixObject";
        type Type = super::FixObject;
    }

    impl ObjectImpl for FixObject {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
}


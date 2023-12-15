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
use std::cell::RefCell;

use gtk::glib;
use gtk::glib::Cast;
use gtk::subclass::prelude::ObjectSubclassIsExt;

use crate::model::sector::Sector;

// To use the Sector in a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct SectorObject(ObjectSubclass<imp::SectorObject>);
}

impl SectorObject {
    pub fn new(sector: &RefCell<Sector>) -> SectorObject {
        let obj: SectorObject = glib::Object::new();
        obj.downcast_ref::<SectorObject>()
            .expect("The item has to be an <SectorObject>.")
            .imp().set_sector(sector.clone());
        obj
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gtk::glib;
    use gtk::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass};

    use crate::model::sector::Sector;

    #[derive(Default)]
    pub struct SectorObject {
        sector: RefCell<Option<RefCell<Sector>>>,
    }

    impl SectorObject {
        pub fn set_sector(&self, sector: RefCell<Sector>) {
            self.sector.replace(Some(sector));
        }

        pub fn sector(&self) -> RefCell<Sector> {
            self.sector.borrow().as_ref().unwrap().clone()
        }
    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for SectorObject {
        const NAME: &'static str = "SectorObject";
        type Type = super::SectorObject;
    }

    impl ObjectImpl for SectorObject {
        fn constructed(&self) {
            self.parent_constructed();
        }

    }

}


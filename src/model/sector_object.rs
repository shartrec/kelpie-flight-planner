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
use gtk::gio::ListModel;
use gtk::glib;
use gtk::prelude::Cast;
use gtk::subclass::prelude::ObjectSubclassIsExt;

use crate::model::sector::Sector;

// To use the Sector in a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct SectorObject(ObjectSubclass<imp::SectorObject>) @implements ListModel;
}

impl SectorObject {
    pub fn new(sector: &Sector) -> SectorObject {
        let obj: SectorObject = glib::Object::new();
        obj.downcast_ref::<SectorObject>()
            .expect("The item has to be an <SectorObject>.")
            .imp().set_sector(sector.clone());
        obj
    }
}

mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;
    use adw::gio;
    use adw::glib::Object;
    use adw::subclass::prelude::ListModelImpl;
    use gtk::{glib, Label};
    use gtk::prelude::StaticType;
    use gtk::subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass};

    use crate::model::sector::Sector;
    use crate::model::waypoint_object::WaypointObject;

    #[derive(Default)]
    pub struct SectorObject {
        sector: RefCell<Option<Sector>>,
        ui: RefCell<Option<Label>>
    }

    impl SectorObject {
        pub fn set_sector(&self, sector: Sector) {
            self.sector.replace(Some(sector));
        }

        pub fn sector(&self) -> Sector {
            self.sector.borrow().as_ref().unwrap().clone()
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
    impl ObjectSubclass for SectorObject {
        const NAME: &'static str = "SectorObject";
        type Type = super::SectorObject;
        type Interfaces = (gio::ListModel, );
    }

    impl ObjectImpl for SectorObject {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl ListModelImpl for SectorObject {
        fn item_type(&self) -> glib::Type {
            WaypointObject::static_type()
        }

        fn n_items(&self) -> u32 {
            (self.sector().get_waypoint_count() + 2) as u32
        }

        fn item(&self, position: u32) -> Option<Object> {
            if position == 0 {
                self.sector().get_start().map(|waypoint| {
                    Object::from(WaypointObject::new(&waypoint))
                })
            } else if position == self.n_items() + 1 {
                self.sector().get_end().map(|waypoint| {
                    Object::from(WaypointObject::new(&waypoint))
                })
            } else {
                self.sector().get_waypoint(position as usize).as_ref().map(|waypoint| {
                    Object::from(WaypointObject::new(&waypoint))
                })
            }
        }
    }


}


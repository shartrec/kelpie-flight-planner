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
use std::rc::Rc;
use gtk::gio::ListModel;
use gtk::glib;
use adw::subclass::prelude::ObjectSubclassIsExt;

use crate::model::sector::Sector;

// To use the Sector in a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct SectorObject(ObjectSubclass<imp::SectorObject>) @implements ListModel;
}

impl SectorObject {
    pub fn new(sector: Rc<RefCell<Sector>>) -> SectorObject {
        let obj: SectorObject = glib::Object::new();
        obj.imp().set_sector(sector.clone());
        obj
    }
}

mod imp {
    use std::cell::RefCell;
    use std::rc::Rc;
    use adw::gio;
    use adw::glib::Object;
    use gtk::{glib, Label};
    use adw::prelude::StaticType;
    use adw::subclass::prelude::{ListModelImpl, ObjectImpl, ObjectImplExt, ObjectSubclass};

    use crate::model::sector::Sector;
    use crate::model::waypoint_object::WaypointObject;

    #[derive(Default)]
    pub struct SectorObject {
        sector: RefCell<Option<Rc<RefCell<Sector>>>>,
        ui: RefCell<Option<Label>>
    }

    impl SectorObject {
        pub fn set_sector(&self, sector: Rc<RefCell<Sector>>) {
            self.sector.replace(Some(sector));
        }

        pub fn sector(&self) -> Rc<RefCell<Sector>> {
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
            let binding = self.sector();
            let sector = binding.borrow();
            let mut x = sector.get_waypoint_count();
            if sector.get_start().is_some() {
                x += 1;
            }
            if sector.get_end().is_some() {
                x += 1;
            }
            x as u32
        }

        fn item(&self, position: u32) -> Option<Object> {
            let binding = self.sector();
            let sector = binding.borrow();

            let mut pos = position as usize;
            if let Some(wp) = sector.get_start() {
                if pos == 0 {
                    return Some(Object::from(WaypointObject::new(&wp)));
                }
                pos -= 1;
            }
            if pos < sector.get_waypoint_count() {
                if let Some(wp) =  sector.get_waypoint(pos) {
                    return  Some(Object::from(WaypointObject::new(&wp)));
                }
            }
            if pos == sector.get_waypoint_count() {
                if let Some(wp) = sector.get_end() {
                    return Some(Object::from(WaypointObject::new(&wp)))
                }
            }
            None
        }
    }


}


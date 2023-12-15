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

// To use the SectorListModel as a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct SectorListModel(ObjectSubclass<imp::SectorListModel>) @implements ListModel;
}

impl SectorListModel {
    pub fn new() -> SectorListModel {
        glib::Object::new()
    }
}

impl Default for SectorListModel {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;
    use gtk::{gio, glib};
    use gtk::glib::{Object, StaticType};
    use gtk::subclass::prelude::{ListModelImpl, ObjectImpl, ObjectImplExt, ObjectSubclass};
    use crate::model::plan::Plan;

    use crate::model::waypoint_object::WaypointObject;

    #[derive(Default)]
    pub struct SectorListModel {
        plan: RefCell<Option<Rc<RefCell<Plan>>>>,
        sector_index: RefCell<Option<usize>>,
    }

    impl SectorListModel {
        pub fn set_plan(&self, plan: Rc<RefCell<Plan>>) {
            self.plan.replace(Some(plan));
        }
        pub fn set_sector_index(&self, sector_index: usize) {
            self.sector_index.replace(Some(sector_index));
        }

        pub fn plan(&self) -> Rc<RefCell<Plan>> {
            self.plan.borrow().as_ref().unwrap().clone()
        }

        pub fn sector_index(&self) -> usize {
            self.sector_index.borrow().as_ref().unwrap().clone()
        }
    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for SectorListModel {
        const NAME: &'static str = "SectorListModel";
        type Type = super::SectorListModel;
        type Interfaces = (gio::ListModel, );
    }

    impl ObjectImpl for SectorListModel {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl ListModelImpl for SectorListModel {
        fn item_type(&self) -> glib::Type {
            WaypointObject::static_type()
        }

        fn n_items(&self) -> u32 {
            let rc_plan = self.plan();
            let sector_binding = rc_plan.deref().borrow();
            let sectors = sector_binding.get_sectors();
            let x = self.sector_index.borrow().expect("Nothing here");
            let s = &sectors[x];
            let x = s.borrow().get_waypoint_count() as u32 + 2;
            x
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            let rc_plan = self.plan();
            let sector_binding = rc_plan.deref().borrow();
            let sectors = sector_binding.get_sectors();
            let x = self.sector_index.borrow().expect("Nothing here");
            let s = &sectors[x];

            let s_borrowed = s.borrow();
            let wp =
                if position == 0 {
                    s_borrowed.get_start()
                } else if position == s.borrow().get_waypoint_count() as u32 + 1 {
                    s_borrowed.get_end()
                } else {
                    let wp = s_borrowed.get_waypoints().read().expect("Can't get waypoint lock")[position as usize - 1].clone();
                    Some(wp)
                };

            match wp {
                Some(wp) => {
                    let wpo = WaypointObject::new(wp.clone());
                    Some(Object::from(wpo))
                }
                None => None
            }
        }
    }
}


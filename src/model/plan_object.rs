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
use gtk::gio::ListModel;

use crate::model::plan::Plan;

// To use the Plan in a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct PlanObject(ObjectSubclass<imp::PlanObject>) @implements ListModel;
}

impl PlanObject {
    pub fn new(plan: Plan) -> PlanObject {
        let obj: PlanObject = glib::Object::new();
        obj.downcast_ref::<PlanObject>()
            .expect("The item has to be an <PlanObject>.")
            .imp().set_plan(plan);
        obj
    }
}

mod imp {
    use std::cell::RefCell;
    use adw::gio;
    use adw::glib::Object;
    use gtk::{glib, Label};
    use gtk::prelude::StaticType;
    use gtk::subclass::prelude::{ListModelImpl, ObjectImpl, ObjectImplExt, ObjectSubclass};
    use crate::model::plan::Plan;
    use crate::model::sector_object::SectorObject;

    #[derive(Default)]
    pub struct PlanObject {
        plan: RefCell<Option<Plan>>,
        ui: RefCell<Option<Label>>
    }

    impl PlanObject {
        pub fn set_plan(&self, plan: Plan) {
            self.plan.replace(Some(plan));
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
    impl ObjectSubclass for PlanObject {
        const NAME: &'static str = "PlanObject";
        type Type = super::PlanObject;
        type Interfaces = (gio::ListModel, );
    }

    impl ObjectImpl for PlanObject {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl ListModelImpl for PlanObject {
        fn item_type(&self) -> glib::Type {
            SectorObject::static_type()
        }


        fn n_items(&self) -> u32 {
            let x = &self.plan.borrow();
            if let Some(p) = x.as_ref() {
                p.get_sectors().len() as u32
            } else {
                0
            }
        }

        fn item(&self, position: u32) -> Option<Object> {
            let x = &self.plan.borrow();
            if let Some(p) = x.as_ref() {
                p.get_sectors().get(position as usize).map(|sector| {
                let so = SectorObject::new(sector);
                    Object::from(so)
                })
            } else {
                None
            }
        }
    }

}


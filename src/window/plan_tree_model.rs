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

use std::cell::RefCell;
use std::rc::Rc;
use gtk::gio::ListModel;
use gtk::{glib, ListItem, TreeListModel};
use gtk::glib::Cast;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use crate::model::plan::Plan;
use crate::model::sector_object::SectorObject;

// To use the Plan as a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct PlanTreeModel(ObjectSubclass<imp::PlanTreeModel>) @implements ListModel;
}

impl PlanTreeModel {
    pub fn new() -> PlanTreeModel {
        glib::Object::new()
    }
}

impl Default for PlanTreeModel {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;
    use std::sync::Arc;
    use gtk::{gio, glib};
    use gtk::glib::{Object, StaticType};
    use gtk::subclass::prelude::{ListModelImpl, ObjectImpl, ObjectImplExt, ObjectSubclass};
    use crate::model::plan::Plan;
    use crate::model::sector_object::SectorObject;


    #[derive(Default)]
    pub struct PlanTreeModel {
        plan: RefCell<Option<Rc<RefCell<Plan>>>>,
    }

    impl PlanTreeModel {
        pub fn set_plan(&self, plan: Rc<RefCell<Plan>>) {
            self.plan.replace(Some(plan));
        }

        pub fn plan(&self) -> Rc<RefCell<Plan>> {
            self.plan.borrow().as_ref().unwrap().clone()
        }

    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for PlanTreeModel {
        const NAME: &'static str = "PlanTreeModel";
        type Type = super::PlanTreeModel;
        type Interfaces = (gio::ListModel, );
    }

    impl ObjectImpl for PlanTreeModel {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl ListModelImpl for PlanTreeModel {
        fn item_type(&self) -> glib::Type {
            SectorObject::static_type()
        }

        fn n_items(&self) -> u32 {
            self.plan().deref().borrow().get_sectors().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            match self.plan().deref().borrow().get_sectors().iter().nth(position as usize) {
                Some(sector) => {
                    let so = SectorObject::new(&sector);
                    Some(Object::from(so))
                }

                None => None
            }
        }
    }

}

pub fn create_plan_model(plan: Rc<RefCell<Plan>>) -> TreeListModel {

    let list_model = PlanTreeModel::new();
    list_model.imp().set_plan(plan.clone());

    TreeListModel::new(list_model, true, true, | parent | {
        match parent.downcast_ref::<SectorObject>() {
            Some(sector_object) => {
                None
            }
            None => None
        }


    })

}
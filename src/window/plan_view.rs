/*
 * Copyright (c) 2003-2003-2023. Trevor Campbell and others.
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
#![forbid(unsafe_code)]

use gtk::gio;
use gtk::{self, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use adw::gio::ListModel;
    use adw::TabPage;
    use glib::subclass::InitializingObject;
    use gtk::gdk::{Key, ModifierType, Rectangle};
    use gtk::gio::{MenuModel, SimpleAction, SimpleActionGroup};
    use gtk::glib::{clone, MainContext, Propagation};
    use gtk::{prelude::WidgetExt, Builder, Button, CheckButton, ColumnView, ColumnViewColumn, DropDown, Entry, Label, ListScrollFlags, PopoverMenu, ScrolledWindow, SingleSelection, Stack, StringObject, TreeListModel, TreeListRow};
    use log::error;
    use std::cell::RefCell;
    use std::ops::{Deref, DerefMut};
    use std::rc::Rc;
    use std::sync::Arc;

    use crate::earth::coordinate::Coordinate;
    use crate::event::Event;
    use crate::hangar::hangar::get_hangar;
    use crate::model::airport::Airport;
    use crate::model::location::Location;
    use crate::model::plan::Plan;
    use crate::model::plan_object::PlanObject;
    use crate::model::sector::Sector;
    use crate::model::sector_object::SectorObject;
    use crate::model::runway_object::RunwayObject;
    use crate::model::waypoint::Waypoint;
    use crate::model::waypoint_object::WaypointObject;
    use crate::planner::planner::Planner;
    use crate::preference::{AUTO_PLAN, USE_MAGNETIC_HEADINGS};
    use crate::window::util::{build_column_factory, build_tree_column_factory, get_airport_map_view, get_airport_view, get_fix_view, get_navaid_view, get_world_map_view, show_airport_map_view, show_airport_view, show_fix_view, show_navaid_view, show_world_map_view, get_tree_path, expand_tree};
    use crate::{earth, event};
    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/plan_view.ui")]
    pub struct PlanView {
        #[template_child]
        pub aircraft_combo: TemplateChild<DropDown>,
        #[template_child]
        pub plan_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub btn_make_plan: TemplateChild<Button>,
        #[template_child]
        pub btn_new_sector: TemplateChild<Button>,
        #[template_child]
        pub btn_move_up: TemplateChild<Button>,
        #[template_child]
        pub btn_move_down: TemplateChild<Button>,
        #[template_child]
        pub btn_max_alt: TemplateChild<CheckButton>,
        #[template_child]
        pub max_alt: TemplateChild<Entry>,

        #[template_child]
        pub plan_tree: TemplateChild<ColumnView>,
        #[template_child]
        pub col_name: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_alt: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_lat: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_long: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_freq: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_hdg: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_dist: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_time: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_speed: TemplateChild<ColumnViewColumn>,

        pub plan: Rc<RefCell<Plan>>,

        popover: RefCell<Option<PopoverMenu>>,
        page: RefCell<Option<TabPage>>,
    }

    impl PlanView {
        pub(crate) fn set_parent_page(&self, page: TabPage) {
            self.page.replace(Some(page));
        }

        pub(crate) fn new_plan(&self) {
            {   // Block limits scope of mutable borrow of the plan
                let mut plan = self.plan.borrow_mut();
                plan.add_sector(Sector::new());
                plan.set_aircraft(&get_hangar().imp().get_default_aircraft());
                plan.set_dirty(false);
            }
            self.refresh(None);
        }

        pub(crate) fn set_plan(&self, plan: Plan) {
            self.plan.replace(plan);
            self.refresh(None);
        }

        pub(crate) fn get_plan(&self) -> Rc<RefCell<Plan>> {
            self.plan.clone()
        }

        pub fn plan_updated(&self) {
            let pref = crate::preference::manager();
            if pref.get::<bool>(AUTO_PLAN).unwrap_or(true) {
                self.make_plan();
            }
            self.refresh(None);
        }

        fn refresh(&self, selection: Option<u32>) {
            if let Some(page) = &self.page.borrow().deref() {
                page.set_title(&self.plan.borrow().get_name());
            }
            // update the heading if required for Mag vs True Hdg
            let pref = crate::preference::manager();
            let col_hdg = if pref.get::<bool>(USE_MAGNETIC_HEADINGS).unwrap_or(false) {
                "Hdg(mag)"
            } else {
                "Hdg(true)"
            };

            self.col_hdg.set_title(Some(col_hdg));

            let plan = self.get_plan();
            let plan_object = PlanObject::new(&plan.clone());

            let model = TreeListModel::new(plan_object, false, false, |object| {
                if object.is::<SectorObject>() {
                    let s = object.downcast_ref::<SectorObject>().expect("Sector Object");
                    let so = s.clone();
                    Some(so.upcast::<ListModel>())
                } else if object.is::<WaypointObject>() {
                    let wp = object.downcast_ref::<WaypointObject>().expect("Waypoint Object");
                    match wp.imp().waypoint().borrow().as_ref() {
                        Some(Waypoint::Airport { .. }) => {
                            let wpo = wp.clone();
                            Some(wpo.upcast::<ListModel>())
                        }
                        _ => None
                    }
                } else {
                    None
                }
            });

            let selection_model = SingleSelection::new(Some(model));

            selection_model.connect_selection_changed(clone!(#[weak(rename_to = view)] self, move |_tree_view, _x, _y| {
                let plan = view.plan.borrow();
                view.btn_move_up.set_sensitive(false);
                view.btn_move_down.set_sensitive(false);
                let sel_pos;

                let selection = view.plan_tree.model().unwrap().selection();
                if !selection.is_empty() {
                    sel_pos = selection.nth(0);
                    let smodel = view.plan_tree.model().unwrap();
                    let ssmodel = smodel.downcast_ref::<SingleSelection>().unwrap();
                    let trm = ssmodel.model().and_downcast::<TreeListModel>().unwrap();
                    if let Some(row) = trm.row(sel_pos) {

                    let tree_path = get_tree_path(sel_pos, &trm);

                        let item = row.item().unwrap();
                        if item.is::<SectorObject>() {
                            let sectors = plan.get_sectors();
                            let sector_index = tree_path[0];
                            if sector_index > 0 && sector_index < sectors.len() as u32 {
                                view.btn_move_up.set_sensitive(true);
                            }
                            if sector_index < (sectors.len() - 1) as u32 {
                                view.btn_move_down.set_sensitive(true);
                            }
                        } else if item.is::<WaypointObject>() {
                            let parent = row.parent().unwrap();
                            let parent_item = parent.item().unwrap();
                            let sector = parent_item.downcast_ref::<SectorObject>().unwrap();
                            let cell = sector.imp().sector();
                            let sector = cell.borrow_mut();

                            // Only if a waypoint.  index > 0 and < waypoint count
                            let mut wp_index = tree_path[1] as usize;
                            if sector.get_start().is_some() {
                                if wp_index == 0 {
                                    return;
                                }
                                wp_index -= 1;
                            }
                            if wp_index > 0 && wp_index < sector.get_waypoint_count() {
                                view.btn_move_up.set_sensitive(true);
                            }
                            if wp_index < sector.get_waypoint_count() - 1 {
                                view.btn_move_down.set_sensitive(true);
                            }
                        }
                    }
                }
            }));

            self.plan_tree.set_model(Some(&selection_model));

            if let Some(stack) = self.obj().parent().and_downcast_ref::<Stack>() {
                stack.page(self.obj().as_ref()).set_title(&plan.borrow().get_name());
                stack.page(self.obj().as_ref()).set_name(&plan.borrow().get_name());
            };

            expand_tree(&selection_model.model().and_downcast::<TreeListModel>().unwrap(), 0);

            if let Some(sel) = selection {
                self.plan_tree.scroll_to(sel, None, ListScrollFlags::SELECT, None);
            }

        }

        fn make_plan(&self) {
            let planner = Planner::new();
            let mut plan = self.plan.borrow_mut();
            let mut loc = None;
            for sector in plan.get_sectors_mut().iter_mut() {
                let waypoints = planner.make_plan(sector.borrow().deref());
                sector.borrow_mut().remove_all_waypoints();
                sector.borrow_mut().add_all_waypoint(waypoints);
                loc = sector.borrow().get_start();
            }
            planner.recalc_plan_elevations(plan.deref_mut());
            drop(plan);
            if let Some(map_view) = get_world_map_view(&self.plan_window) {
                if let Some(wp) = loc {
                    map_view.imp().set_plan(self.plan.clone());
                    map_view.imp().center_map(wp.get_loc().clone());
                }
            }
            event::manager().notify_listeners(Event::PlanChanged);
        }

        pub fn initialise(&self) {
            if let Some(rx) = event::manager().register_listener() {
                MainContext::default().spawn_local(clone!(#[weak(rename_to = view)] self, async move {
                    while let Ok(ev) = rx.recv().await {
                        if let Event::PreferencesChanged = ev {
                            view.refresh(None);
                        }
                    }
                }));
            }
        }

        pub fn add_airport_to_plan(&self, loc: Arc<Airport>) {
            let mut added = false;
            let mut plan = self.plan.borrow_mut();
            // See if a sector is selected
            let selection = self.plan_tree.model().unwrap().selection();

            if !selection.is_empty() {
                let sel_pos = selection.nth(0);
                let s_model = self.plan_tree.model().unwrap();
                let ss_model = s_model.downcast_ref::<SingleSelection>().unwrap();
                let trm = ss_model.model().and_downcast::<TreeListModel>().unwrap();
                if let Some(row) = trm.row(sel_pos) {

                    let mut sector = None;
                    let item = row.item().unwrap();
                    if item.is::<SectorObject>() {
                        let sector_o = item.downcast_ref::<SectorObject>().unwrap();
                        sector = Some(sector_o.imp().sector());
                    } else if item.is::<WaypointObject>() {
                        let parent = row.parent().unwrap();
                        let parent_item = parent.item().unwrap();
                        let sector_o = parent_item.downcast_ref::<SectorObject>().unwrap();
                        sector = Some(sector_o.imp().sector());
                    }
                    if let Some(sector) = sector {
                        if sector.borrow().get_start().is_none() {
                            sector.borrow_mut().set_start(Some(loc.clone()));
                            added = true;
                        } else if sector.borrow().get_end().is_none() {
                            sector.borrow_mut().set_end(Some(loc.clone()));
                            added = true;
                        }
                    }
                }
                if !added {
                    plan.add_airport(loc);
                }
                drop(plan);
                self.plan_updated();
                event::manager().notify_listeners(Event::PlanChanged);
            }
        }

        pub fn add_waypoint_to_plan(&self, waypoint: Waypoint) {
            let mut plan = self.plan.borrow_mut();
            // See if a sector or waypoint is selected
            let selection = self.plan_tree.model().unwrap().selection();
            if !selection.is_empty() {
                let sel_pos = selection.nth(0);
                let smodel = self.plan_tree.model().unwrap();
                let ssmodel = smodel.downcast_ref::<SingleSelection>().unwrap();
                let trm = ssmodel.model().and_downcast::<TreeListModel>().unwrap();
                if let Some(row) = trm.row(sel_pos) {

                    let tree_path = get_tree_path(sel_pos, &trm);
                    let item = row.item().unwrap();
                    if item.is::<SectorObject>() {
                        let sector = item.downcast_ref::<SectorObject>().unwrap();
                        let cell = sector.imp().sector();
                        cell.borrow_mut().add_waypoint_optimised(waypoint);
                    } else if item.is::<WaypointObject>() {
                        let parent = row.parent().unwrap();
                        let parent_item = parent.item().unwrap();
                        let sector = parent_item.downcast_ref::<SectorObject>().unwrap();
                        let cell = sector.imp().sector();
                        let mut wp_pos = tree_path[1] as usize;
                        // decrement position if we have a start
                        if wp_pos > 0 && cell.borrow().get_start().is_some() {
                            wp_pos -= 1;
                        }

                        cell.borrow_mut().insert_waypoint(wp_pos as usize, waypoint);
                    }
                }

            } else {
                let i = plan.get_sectors().len();
                if i > 0 {
                    let sector = &mut plan.get_sectors_mut()[i - 1];
                    sector.borrow_mut().add_waypoint_optimised(waypoint);
                }
            }
            let planner = Planner::new();
            planner.recalc_plan_elevations(plan.deref_mut());
            drop(plan);
            self.refresh(None);
            event::manager().notify_listeners(Event::PlanChanged);
        }

        fn new_sector(&self) {
            let mut prev_airport_id = "".to_string();
            let mut prev = false;
            let mut plan = self.plan.borrow_mut();

            if let Some(prev_sector) = plan.get_sectors().last() {
                if let Some(Waypoint::Airport { airport, .. }) = prev_sector.borrow().get_end() {
                    prev_airport_id = airport.get_id().to_string().clone();
                    prev = true;
                }
            }
            plan.add_sector(Sector::new());

            if prev {
                if let Some(airport) =
                    earth::get_earth_model().get_airport_by_id(prev_airport_id.as_str())
                {
                    plan.add_airport(airport);
                }
            }
            drop(plan);
            self.refresh(None);
            event::manager().notify_listeners(Event::PlanChanged);
        }

        fn move_selected_up(&self) {
            let mut plan = self.plan.borrow_mut();

            let mut sel_pos = 0;
            let mut new_selection = Some(sel_pos);

            let selection = self.plan_tree.model().unwrap().selection();
            if !selection.is_empty() {
                sel_pos = selection.nth(0);
                let s_model = self.plan_tree.model().unwrap();
                let ss_model = s_model.downcast_ref::<SingleSelection>().unwrap();
                let trm = ss_model.model().and_downcast::<TreeListModel>().unwrap();
                if let Some(row) = trm.row(sel_pos) {

                    let tree_path = get_tree_path(sel_pos, &trm);
                    let item = row.item().unwrap();
                    if item.is::<SectorObject>() {
                        let sector_index = tree_path[0];
                        if sector_index > 0 {
                            plan.move_sector_up(sector_index as usize);
                            new_selection = None;
                        }
                    } else if item.is::<WaypointObject>() {
                        let parent = row.parent().unwrap();
                        let parent_item = parent.item().unwrap();
                        let sector = parent_item.downcast_ref::<SectorObject>().unwrap();
                        let cell = sector.imp().sector();
                        let mut sector = cell.borrow_mut();

                        // Only if a waypoint.  index > 0 and < waypoint count
                        let mut wp_index = tree_path[1] as usize;
                        if sector.get_start().is_some() {
                            if wp_index == 0 {
                                return;
                            }
                            wp_index -= 1;
                        }
                        if wp_index > 0
                            && wp_index < sector.get_waypoint_count()
                        {
                            sector.move_waypoint_up(wp_index);
                            new_selection = Some(sel_pos - 1);
                        } else {
                            return;
                        }
                    }
                }
            }

            drop(plan);
            self.refresh(new_selection);
            event::manager().notify_listeners(Event::PlanChanged);
        }

        fn move_selected_down(&self) {
            let mut plan = self.plan.borrow_mut();

            let mut sel_pos = 0;
            let mut new_selection = Some(sel_pos);

            let selection = self.plan_tree.model().unwrap().selection();
            if !selection.is_empty() {
                sel_pos = selection.nth(0);
                let s_model = self.plan_tree.model().unwrap();
                let ss_model = s_model.downcast_ref::<SingleSelection>().unwrap();
                let trm = ss_model.model().and_downcast::<TreeListModel>().unwrap();
                if let Some(row) = trm.row(sel_pos) {

                    let tree_path = get_tree_path(sel_pos, &trm);
                    let item = row.item().unwrap();
                    if item.is::<SectorObject>() {
                        let sector_index = tree_path[0];
                        if sector_index < (plan.get_sectors().len() - 1) as u32 {
                            plan.move_sector_down(sector_index as usize);
                            new_selection = None;
                        }
                    } else if item.is::<WaypointObject>() {
                        let parent = row.parent().unwrap();
                        let parent_item = parent.item().unwrap();
                        let sector = parent_item.downcast_ref::<SectorObject>().unwrap();
                        let cell = sector.imp().sector();
                        let mut sector = cell.borrow_mut();

                        // Only if a waypoint.  index > 0 and < waypoint count
                        let mut wp_index = tree_path[1] as usize;
                        if sector.get_start().is_some() {
                            if wp_index == 0 {
                                return;
                            }
                            wp_index -= 1;
                        }
                        if wp_index < sector.get_waypoint_count() - 1
                        {
                            sector.move_waypoint_down(wp_index);
                            new_selection = Some(sel_pos + 1);
                        } else {
                            return;
                        }
                    }
                }
            }

            drop(plan);
            self.refresh(new_selection);
            event::manager().notify_listeners(Event::PlanChanged);
        }

        fn remove_selected(&self) {
            let mut plan = self.plan.borrow_mut();

            let mut sel_pos = 0;
            let mut new_selection = Some(sel_pos);

            let selection = self.plan_tree.model().unwrap().selection();
            if !selection.is_empty() {
                sel_pos = selection.nth(0);
                let s_model = self.plan_tree.model().unwrap();
                let ss_model = s_model.downcast_ref::<SingleSelection>().unwrap();
                let trm = ss_model.model().and_downcast::<TreeListModel>().unwrap();
                if let Some(row) = trm.row(sel_pos) {

                    let tree_path = get_tree_path(sel_pos, &trm);
                    let item = row.item().unwrap();
                    if item.is::<SectorObject>() {
                        let sector_index = tree_path[0];
                        if sector_index < plan.get_sectors().len() as u32 {
                            plan.remove_sector_at(sector_index as usize);
                            new_selection = None;
                        }
                    } else if item.is::<WaypointObject>() {
                        let parent = row.parent().unwrap();
                        let parent_item = parent.item().unwrap();
                        let sector = parent_item.downcast_ref::<SectorObject>().unwrap();
                        let cell = sector.imp().sector();
                        let mut sector = cell.borrow_mut();

                        // Only if a waypoint.  index > 0 and < waypoint count
                        let mut wp_index = tree_path[1] as usize;
                        if sector.get_start().is_some() {
                            if wp_index == 0 {
                                sector.set_start(None);
                                new_selection = None;
                            } else {
                                wp_index -= 1;
                            }
                        }
                        if wp_index < sector.get_waypoint_count()
                        {
                            sector.remove_waypoint(wp_index);
                            new_selection = Some(sel_pos);
                        }
                        if wp_index == sector.get_waypoint_count() {
                            sector.set_end(None);
                            new_selection = None;
                        }
                    }
                }
            }

            drop(plan);
            self.refresh(new_selection);
            event::manager().notify_listeners(Event::PlanChanged);
        }

        fn get_selected_location(&self) -> Option<Coordinate> {
            let selection = self.plan_tree.model().unwrap().selection();
            if !selection.is_empty() {
                let sel_pos = selection.nth(0);
                let s_model = self.plan_tree.model().unwrap();
                let ss_model = s_model.downcast_ref::<SingleSelection>().unwrap();
                let trm = ss_model.model().and_downcast::<TreeListModel>().unwrap();
                if let Some(row) = trm.row(sel_pos) {

                    let item = row.item().unwrap();
                    if item.is::<SectorObject>() {
                        let sector = item.downcast_ref::<SectorObject>().unwrap();
                        let cell = sector.imp().sector();
                        let selection = if let Some(wp) = cell.borrow().get_start() {
                            Some(wp.get_loc().clone())
                        } else { cell.borrow().get_end().map(|wp| wp.get_loc().clone()) };
                        selection
                    } else if item.is::<WaypointObject>() {
                        let waypoint = item.downcast_ref::<WaypointObject>().unwrap();
                        let cell = waypoint.imp().waypoint();
                        // Only if a waypoint.  index > 0 and < waypoint count
                        let selection =Some(cell.borrow().as_ref()?.get_loc().clone());
                        selection
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }

        fn get_selected_airport(&self) -> Option<Arc<Airport>> {
            let selection = self.plan_tree.model().unwrap().selection();
            if !selection.is_empty() {
                let sel_pos = selection.nth(0);
                let s_model = self.plan_tree.model().unwrap();
                let ss_model = s_model.downcast_ref::<SingleSelection>().unwrap();
                let trm = ss_model.model().and_downcast::<TreeListModel>().unwrap();
                if let Some(row) = trm.row(sel_pos) {

                    let item = row.item().unwrap();
                    if item.is::<SectorObject>() {
                        let sector = item.downcast_ref::<SectorObject>().unwrap();
                        let cell = sector.imp().sector();
                        let selection = if let Some(wp) = cell.borrow().get_start() {
                            match wp {
                                Waypoint::Airport { airport, .. } => {
                                    Some(airport.clone())
                                }
                                _ => None
                            }
                        } else if let Some(wp) = cell.borrow().get_end() {
                            match wp {
                                Waypoint::Airport { airport, .. } => {
                                    Some(airport.clone())
                                }
                                _ => None
                            }
                        } else {
                            None
                        };
                        selection
                    } else if item.is::<WaypointObject>() {
                        let waypoint = item.downcast_ref::<WaypointObject>().unwrap();
                        let wp = waypoint.imp().waypoint();
                        let selection = match wp.borrow().as_ref() {
                            Some(Waypoint::Airport { airport, .. }) => {
                                Some(airport.clone())
                            }
                            _ => None
                        };
                        selection
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }

        }

        //noinspection RsExternalLinter
        fn setup_aircraft_combo(&self) {
            self.aircraft_combo.set_factory(Some(&build_column_factory(|label: Label, string_object: &StringObject| {
                label.set_label(string_object.string().as_ref());
                label.set_xalign(0.0);
            })));

            let selection_model = SingleSelection::new(Some(get_hangar().clone()));
            self.aircraft_combo.set_model(Some(&selection_model));

            self.aircraft_combo.connect_selected_notify(clone!(#[weak(rename_to = view)] self, move | combo | {
                // Get the selection
                let index = combo.selected();
                if let Some(aircraft) = get_hangar().imp().aircraft_at(index) {
                    let mut plan1 = view.plan.borrow_mut();
                    plan1.set_aircraft(&Some(aircraft));
                }
            }));

            // set the selection initially to the default
            let hangar = get_hangar().imp();
            for (i, (_k, a)) in hangar.get_all().read().expect("could not get hangar lock").iter().enumerate() {
                if *a.is_default() {
                    self.aircraft_combo.set_selected(i as u32);
                }
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlanView {
        const NAME: &'static str = "PlanView";
        type Type = super::PlanView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlanView {
        //noinspection DuplicatedCode
        fn constructed(&self) {
            self.parent_constructed();

            self.col_name.set_factory(Some(&build_tree_column_factory(|label: Label, row: &TreeListRow| {
                // get the item from the tree list row
                let item = row.item().unwrap();
                if item.is::<SectorObject>() {
                    let sector = item.downcast_ref::<SectorObject>().unwrap();
                    let cell = sector.imp().sector();
                    label.set_label(cell.borrow().get_name().as_str());
                    sector.imp().set_ui(Some(label.clone()));
                } else if item.is::<WaypointObject>() {
                    let waypoint = item.downcast_ref::<WaypointObject>().unwrap();
                    let cell = waypoint.imp().waypoint();
                    label.set_label(cell.borrow().as_ref().unwrap().get_name());
                    waypoint.imp().set_ui(Some(label.clone()));
                } else if item.is::<RunwayObject>() {
                    let runway = item.downcast_ref::<RunwayObject>().unwrap();
                    let cell = runway.imp().runway();
                    label.set_label(cell.borrow().as_ref().unwrap().number_pair().as_str());
                }
                label.set_xalign(0.0);
            })));

            self.col_alt.set_factory(Some(&build_column_factory(|label: Label, row: &TreeListRow| {
                // get the item from the tree list row
                let item = row.item().unwrap();
                if item.is::<SectorObject>() {
                    label.set_label("");
                } else if item.is::<WaypointObject>() {
                    let waypoint = item.downcast_ref::<WaypointObject>().unwrap();
                    let cell = waypoint.imp().waypoint();
                    label.set_label(cell.borrow().as_ref().unwrap().get_elevation().to_string().as_str());
                }
                label.set_xalign(1.0);
            })));

            self.col_lat.set_factory(Some(&build_column_factory(|label: Label, row: &TreeListRow| {
                // get the item from the tree list row
                let item = row.item().unwrap();
                if item.is::<SectorObject>() {
                    label.set_label("");
                } else if item.is::<WaypointObject>() {
                    let waypoint = item.downcast_ref::<WaypointObject>().unwrap();
                    let cell = waypoint.imp().waypoint();
                    label.set_label(cell.borrow().as_ref().unwrap().get_lat_as_string().as_str());
                }
                label.set_xalign(0.0);
            })));

            self.col_long.set_factory(Some(&build_column_factory(|label: Label, row: &TreeListRow| {
                // get the item from the tree list row
                let item = row.item().unwrap();
                if item.is::<SectorObject>() {
                    label.set_label("");
                } else if item.is::<WaypointObject>() {
                    let waypoint = item.downcast_ref::<WaypointObject>().unwrap();
                    let cell = waypoint.imp().waypoint();
                    label.set_label(cell.borrow().as_ref().unwrap().get_long_as_string().as_str());
                }
                label.set_xalign(0.0);
            })));

            self.col_freq.set_factory(Some(&build_column_factory(|label: Label, row: &TreeListRow| {
                // get the item from the tree list row
                let item = row.item().unwrap();
                if item.is::<SectorObject>() {
                    label.set_label("");
                } else if item.is::<WaypointObject>() {
                    let waypoint = item.downcast_ref::<WaypointObject>().unwrap();
                    let cell = waypoint.imp().waypoint();
                    label.set_label(cell.borrow().as_ref().unwrap().get_freq().map_or("".to_string(), |f| {
                        f.to_string()
                    }).as_str());
                } else if item.is::<RunwayObject>() {
                    let runway = item.downcast_ref::<RunwayObject>().unwrap();
                    let cell = runway.imp().runway();
                    let rw_ref = cell.borrow();
                    let id = rw_ref.as_ref().unwrap().number();
                    let opp_id = rw_ref.as_ref().unwrap().opposite_number();
                    let airport = runway.imp().airport();
                    let ils = airport.get_ils(id);
                    let ils_opp = opp_id.and_then(|op| airport.get_ils(&op));

                    let text = if ils.is_some() || ils_opp.is_some() {
                        format!(
                            "{:0.3} / {:0.3}",
                            ils.unwrap_or(0.),
                            ils_opp.unwrap_or(0.),
                        )
                    } else {
                        "".to_string()
                    };
                    label.set_label(text.as_str());
                }
                label.set_xalign(0.0);
            })));

            // Need to do self weak clone stuff for remaining columns
            self.col_hdg.set_factory(Some(&build_column_factory(clone!(#[weak(rename_to = view)] self, move|label: Label, row: &TreeListRow| {
                // get the item from the tree list row
                let item = row.item().unwrap();
                if item.is::<SectorObject>() {
                    label.set_label("");
                } else if item.is::<WaypointObject>() {
                    let waypoint = item.downcast_ref::<WaypointObject>().unwrap();
                    let cell = waypoint.imp().waypoint();
                    let value = &(format!("{:6.0}", view.plan.borrow().get_leg_heading_to(cell.borrow().as_ref().unwrap())));
                    label.set_label(value.as_str());
                }
                label.set_xalign(0.0);
            }))));
            self.col_dist.set_factory(Some(&build_column_factory(clone!(#[weak(rename_to = view)] self, move|label: Label, row: &TreeListRow| {
                // get the item from the tree list row
                let item = row.item().unwrap();
                if item.is::<SectorObject>() {
                    label.set_label("");
                } else if item.is::<WaypointObject>() {
                    let waypoint = item.downcast_ref::<WaypointObject>().unwrap();
                    let cell = waypoint.imp().waypoint();
                    let value = view.plan.borrow().get_leg_distance_to_as_string(cell.borrow().as_ref().unwrap());
                    label.set_label(value.as_str());
                }
                label.set_xalign(0.0);
            }))));
            self.col_time.set_factory(Some(&build_column_factory(clone!(#[weak(rename_to = view)] self, move|label: Label, row: &TreeListRow| {
                // get the item from the tree list row
                let item = row.item().unwrap();
                if item.is::<SectorObject>() {
                    label.set_label("");
                } else if item.is::<WaypointObject>() {
                    let waypoint = item.downcast_ref::<WaypointObject>().unwrap();
                    let cell = waypoint.imp().waypoint();
                    let value = view.plan.borrow().get_time_to_as_string(cell.borrow().as_ref().unwrap());
                    label.set_label(value.as_str());
                }
                label.set_xalign(0.0);
            }))));
            self.col_speed.set_factory(Some(&build_column_factory(clone!(#[weak(rename_to = view)] self, move|label: Label, row: &TreeListRow| {
                // get the item from the tree list row
                let item = row.item().unwrap();
                if item.is::<SectorObject>() {
                    label.set_label("");
                } else if item.is::<WaypointObject>() {
                    let waypoint = item.downcast_ref::<WaypointObject>().unwrap();
                    let cell = waypoint.imp().waypoint();
                    match cell.borrow().as_ref().unwrap() {
                        Waypoint::Airport{..} => {
                            label.set_label("");
                        }
                        _ => {
                            let value = view.plan.borrow().get_speed_to_as_string(cell.borrow().as_ref().unwrap());
                            label.set_label(value.as_str());
                        }
                    };
                }
                label.set_xalign(0.0);
            }))));

            self.btn_make_plan
                .connect_clicked(clone!(#[weak(rename_to = view)] self, move |_search| {
                    view.make_plan();
                    view.refresh(None);
                }));

            let builder = Builder::from_resource("/com/shartrec/kelpie_planner/plan_popover.ui");
            let menu = builder.object::<MenuModel>("plan-menu");
            match menu {
                Some(popover) => {
                    let popover = PopoverMenu::builder()
                        .menu_model(&popover)
                        .has_arrow(false)
                        .build();
                    popover.set_parent(&self.plan_tree.get());
                    let _ = self.popover.replace(Some(popover));
                }
                None => error!(" Not a popover"),
            }


            // Enable context menu key
            let ev_key = gtk::EventControllerKey::new();
            ev_key.connect_key_pressed(clone!(#[weak(rename_to = view)] self, #[upgrade_or] Propagation::Proceed,
                    move | _event, key_val, _key_code, modifier | {
                if key_val == Key::Menu && modifier == ModifierType::empty() {
                    if view.get_selected_location().is_some() {
                        // Need to get selected row x, y but that's not supported for ColumnViews
                        let rect = Rectangle::new(50, 1, 1, 1);
                        if let Some(popover) = view.popover.borrow().as_ref() {
                            popover.set_pointing_to(Some(&rect));
                            popover.popup();
                        };
                    }
                    Propagation::Stop
                } else {
                    Propagation::Proceed
                }

            }));
            self.plan_window.add_controller(ev_key);

            let gesture = gtk::GestureClick::new();
            gesture.set_button(3);
            gesture.connect_released(clone!(#[weak(rename_to = view)] self, move |gesture, _n, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                if let Some(popover) = view.popover.borrow().as_ref() {
                        popover.set_pointing_to(Some(&Rectangle::new(x as i32, y as i32, 1, 1)));
                        popover.popup();
                };
            }));
            self.plan_window.add_controller(gesture);

            self.btn_new_sector
                .connect_clicked(clone!(#[weak(rename_to = view)] self, move |_| {
                    view.new_sector();
                }));

            self.btn_move_up
                .connect_clicked(clone!(#[weak(rename_to = view)] self, move |_| {
                    view.move_selected_up();
                }));

            self.btn_move_down
                .connect_clicked(clone!(#[weak(rename_to = view)] self, move |_| {
                    view.move_selected_down();
                }));

            self.btn_max_alt
                .connect_toggled(clone!(#[weak(rename_to = view)] self, move | button | {
                let mut plan = view.plan.borrow_mut();
                    if button.is_active() {
                        match view.max_alt.text().parse::<i32>() {
                            Ok(n) => {
                                plan.set_max_altitude(Some(n));
                            }
                            _ => {
                                plan.set_max_altitude(None);
                            }
                        }
                        view.max_alt.set_sensitive(true);
                    } else {
                        plan.set_max_altitude(None);
                        view.max_alt.set_sensitive(false);
                    }
                }));

            self.max_alt.connect_changed(clone!(#[weak(rename_to = view)] self, move| editable | {
                if view.btn_max_alt.is_active() {
                    let mut plan = view.plan.borrow_mut();
                    match editable.text().parse::<i32>() {
                        Ok(n) => {
                            plan.set_max_altitude(Some(n));
                        }
                        _ => {
                            plan.set_max_altitude(None);
                        }
                    }
                }
            }));

            // Set up the popup menu
            let actions = SimpleActionGroup::new();
            self.plan_window
                .get()
                .insert_action_group("plan", Some(&actions));

            let action = SimpleAction::new("remove", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
               view.remove_selected();
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("view", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
               if let Some(airport) = view.get_selected_airport() {
                    if let Some(airport_map_view) = get_airport_map_view(&view.plan_window.get()) {
                        show_airport_map_view(&view.plan_window.get());
                        airport_map_view.imp().set_airport(airport);
                    }
                }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("show_on_map", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
               if let Some(map_view) = get_world_map_view(&view.plan_window) {
                    show_world_map_view(&view.plan_window);
                    if let Some(loc) = view.get_selected_location() {
                        map_view.imp().set_plan(view.plan.clone());
                        map_view.imp().center_map(loc.clone());
                    }
                }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_airports_near", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                if let Some(loc) = view.get_selected_location() {
                    if let Some(airport_view) = get_airport_view(&view.plan_window.get()) {
                        show_airport_view(&view.plan_window.get());
                        airport_view.imp().search_near(&loc);
                    }
               }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_navaids_near", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                if let Some(loc) = view.get_selected_location() {
                    if let Some(navaid_view) = get_navaid_view(&view.plan_window.get()) {
                        show_navaid_view(&view.plan_window.get());
                        navaid_view.imp().search_near(&loc);
                    }
                    }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_fixes_near", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                if let Some(loc) = view.get_selected_location() {
                    if let Some(fix_view) = get_fix_view(&view.plan_window.get()) {
                        show_fix_view(&view.plan_window.get());
                        fix_view.imp().search_near(&loc);
                    }
                }
            }));
            actions.add_action(&action);

            self.setup_aircraft_combo();

            self.initialise();
        }

        fn dispose(&self) {
            if let Some(popover) = self.popover.borrow().as_ref() {
                popover.unparent();
            };
        }
    }

    impl BoxImpl for PlanView {}

    impl WidgetImpl for PlanView {}
}

glib::wrapper! {
    pub struct PlanView(ObjectSubclass<imp::PlanView>) @extends gtk::Widget, gtk::Box,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PlanView {
    pub fn new() -> Self {
        glib::Object::new::<PlanView>()
    }
}

impl Default for PlanView {
    fn default() -> Self {
        Self::new()
    }
}

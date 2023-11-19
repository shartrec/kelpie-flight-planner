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
use gtk::{self, CompositeTemplate, glib, prelude::*, subclass::prelude::*};
use gtk::gio;

mod imp {
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::rc::Rc;
    use std::sync::Arc;

    use glib::subclass::InitializingObject;
    use gtk::{Builder, Button, CheckButton, DropDown, Entry, Label, ListItem, PopoverMenu, ScrolledWindow, SignalListItemFactory, SingleSelection, Stack, StringObject, TreePath, TreeStore, TreeView};
    use gtk::gdk::Rectangle;
    use gtk::gio::{MenuModel, SimpleAction, SimpleActionGroup};
    use gtk::glib::{clone, MainContext, PRIORITY_DEFAULT};
    use log::error;

    use crate::{earth, event};
    use crate::earth::coordinate::Coordinate;
    use crate::event::Event;
    use crate::hangar::hangar::get_hangar;
    use crate::model::airport::Airport;
    use crate::model::location::Location;
    use crate::model::plan::Plan;
    use crate::model::sector::Sector;
    use crate::model::waypoint::Waypoint;
    use crate::planner::planner::Planner;
    use crate::preference::{AUTO_PLAN, USE_MAGNETIC_HEADINGS};
    use crate::window::util::{get_airport_map_view, get_airport_view, get_fix_view, get_navaid_view, get_world_map_view, show_airport_map_view, show_airport_view, show_fix_view, show_navaid_view};

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/plan_view.ui")]
    pub struct PlanView {
        #[template_child]
        pub aircraft_combo: TemplateChild<DropDown>,
        #[template_child]
        pub plan_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub plan_tree: TemplateChild<TreeView>,
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

        pub plan: Rc<RefCell<Plan>>,

        popover: RefCell<Option<PopoverMenu>>,
        my_listener_id: RefCell<usize>,
    }

    enum Col {
        Name = 0,
        Elev = 1,
        Lat = 2,
        Long = 3,
        Freq = 4,
        Hdg = 5,
        Dist = 6,
        Time = 7,
        Speed = 8,
    }

    impl PlanView {
        pub(crate) fn new_plan(&self) {
            {
                let mut plan = self.plan.borrow_mut();
                plan.add_sector(Sector::new());
                plan.set_aircraft(&get_hangar().imp().get_default_aircraft());
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
            if pref.get::<bool>(AUTO_PLAN).unwrap_or(false) {
                self.make_plan();
            }
            self.refresh(None);
        }

        fn refresh(&self, selection: Option<TreePath>) {
            // update the heading if required for Mag vs True Hdg
            let pref = crate::preference::manager();
            let col_hdg = if pref.get::<bool>(USE_MAGNETIC_HEADINGS).unwrap_or(false) {
                "Hdg(mag)"
            } else {
                "Hdg(true)"
            };
            if let Some(col) = self.plan_tree.column(5) {
                col.set_title(col_hdg);
            }

            let tree_store = TreeStore::new(&[
                String::static_type(),
                i32::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
            ]);
            // Iterate over the plan loading the TreeModel
            let plan = self.plan.borrow();
            for s_ref in plan.get_sectors().deref() {
                let binding = s_ref.borrow();
                let s = binding.deref();
                if !s.is_empty() {
                    let row = tree_store.append(None);
                    tree_store.set(&row,
                                   &[
                                       (Col::Name as u32, &s.get_name()),
                                       (Col::Dist as u32, &s.get_distance_as_string(&plan)),
                                       (Col::Time as u32, &s.get_duration_as_string(&plan)),
                                   ]);
                    if let Some(airport) = s.get_start() {
                        let wp_row = tree_store.append(Some(&row));
                        tree_store.set(
                            &wp_row,
                            &[
                                (Col::Name as u32, &airport.get_name()),
                                (Col::Elev as u32, &(airport.get_elevation())),
                                (Col::Lat as u32, &airport.get_lat_as_string()),
                                (Col::Long as u32, &airport.get_long_as_string()),
                            ],
                        );
                    }
                    for wp in s
                        .get_waypoints()
                        .read()
                        .expect("Can't get read lock on sectors")
                        .deref()
                    {
                        let wp_row = tree_store.append(Some(&row));
                        tree_store.set(
                            &wp_row,
                            &[
                                (Col::Name as u32, &wp.get_name()),
                                (Col::Elev as u32, &(wp.get_elevation())),
                                (Col::Lat as u32, &wp.get_lat_as_string()),
                                (Col::Long as u32, &wp.get_long_as_string()),
                                (
                                    Col::Freq as u32,
                                    &(match wp.get_freq() {
                                        Some(f) => format!("{:>6.2}", f),
                                        None => "".to_string(),
                                    }),
                                ),
                                (
                                    Col::Hdg as u32,
                                    &(format!("{:6.0}", plan.get_leg_heading_to(wp))),
                                ),
                                (
                                    Col::Dist as u32,
                                    &plan.get_leg_distance_to_as_string(wp),
                                ),
                                (Col::Time as u32, &plan.get_time_to_as_string(wp)),
                                (Col::Speed as u32, &plan.get_speed_to_as_string(wp)),
                            ],
                        );
                    }
                    if let Some(airport) = s.get_end() {
                        let wp_row = tree_store.append(Some(&row));
                        tree_store.set(
                            &wp_row,
                            &[
                                (Col::Name as u32, &airport.get_name()),
                                (Col::Elev as u32, &(airport.get_elevation())),
                                (Col::Lat as u32, &airport.get_lat_as_string()),
                                (Col::Long as u32, &airport.get_long_as_string()),
                                (Col::Dist as u32, &plan.get_leg_distance_to_as_string(&airport)),
                            ],
                        );
                    }
                }
            }

            self.plan_tree.set_model(Some(&tree_store));
            self.plan_tree.expand_all();
            if let Some(path) = selection {
                self.plan_tree.selection().select_path(&path);
                self.plan_tree
                    .emit_by_name_with_values("cursor-changed", &[]);
            };

            if let Some(stack) = self.obj().parent().and_downcast_ref::<Stack>() {
                stack.page(self.obj().as_ref()).set_title(&plan.get_name());
            };
        }

        fn make_plan(&self) {
            let planner = Planner::new();
            let plan = self.plan.borrow_mut();
            let mut loc = None;
            for s in plan.get_sectors().iter() {
                let waypoints = planner.make_plan(s.borrow().get_start(), s.borrow().get_end());
                let mut s_clone = s.borrow().deref().clone();
                s_clone.add_all_waypoint(waypoints);
                s.replace(s_clone);
                planner.recalc_plan_elevations(&plan);
                loc = s.borrow().get_start();
            }
            drop(plan);
            if let Some(map_view) = get_world_map_view(&self.plan_window) {
                if let Some(wp) = loc {
                    map_view.imp().set_plan(self.plan.clone());
                    map_view.imp().center_map(wp.get_loc().clone());
                }
            }
        }

        pub fn initialise(&self) -> () {
            let (tx, rx) = MainContext::channel(PRIORITY_DEFAULT);
            let index = event::manager().register_listener(tx);
            rx.attach(None, clone!(@weak self as view => @default-return glib::source::Continue(true), move |ev: Event| {
                match ev {
                    Event::PreferencesChanged => {
                        view.refresh(None);
                    },
                    _ => (),
                }
                glib::source::Continue(true)
            }));
            // self.my_listener.replace(Some(rx));
            self.my_listener_id.replace(index);
        }

        pub fn add_airport_to_plan(&self, loc: Arc<Airport>) {
            let mut added = false;
            let mut plan = self.plan.borrow_mut();
            // See if a sector is selected
            if let Some((model, iter)) = self.plan_tree.selection().selected() {
                let path = model.path(&iter).indices();
                // Sectors are at the top level
                if path.len() == 1 {
                    let sector_index = path[0] as usize;
                    let sector = &plan.get_sectors()[sector_index];
                    if sector.borrow().get_start() == None {
                        sector.borrow_mut().set_start(Some(loc.clone()));
                        added = true;
                    } else if sector.borrow().get_end() == None {
                        sector.borrow_mut().set_end(Some(loc.clone()));
                        added = true;
                    }
                }
            }

            if !added {
                plan.add_airport(loc);
            }
            drop(plan);
            let _ = &self.refresh(None);
        }

        pub fn add_waypoint_to_plan(&self, waypoint: Waypoint) {
            let plan = self.plan.borrow();
            // See if a sector or waypoint is selected
            if let Some((model, iter)) = self.plan_tree.selection().selected() {
                let path = model.path(&iter).indices();
                // Sectors are at the top level
                match path.len() {
                    1 => {
                        // a Sector only is selected
                        let sector_index = path[0] as usize;
                        let sector = &plan.get_sectors()[sector_index];
                        sector.borrow_mut().add_waypoint(waypoint);
                    }
                    2 => {
                        let sector_index = path[0] as usize;
                        let mut wp_index = path[1] as usize;
                        // The airport is the first subitem of the plan
                        if wp_index != 0 {
                            wp_index -= 1;
                        }
                        let sector = &plan.get_sectors()[sector_index];
                        sector
                            .borrow_mut()
                            .insert_waypoint(wp_index, waypoint);
                    }
                    _ => {
                        // Add to end of last sector
                        if let Some(s) = plan.get_sectors().last() {
                            s.borrow_mut().add_waypoint(waypoint);
                        }
                    }
                }
            } else {
                if let Some(s) = plan.get_sectors().last() {
                    s.borrow_mut().add_waypoint(waypoint);
                }
            }
            drop(plan);
            let _ = &self.refresh(None);
        }

        fn new_sector(&self) {
            let mut prev_airport_id = "".to_string();
            let mut prev = false;
            let mut plan = self.plan.borrow_mut();

            if let Some(prev_sector) = plan.get_sectors().last() {
                if let Some(wp) = prev_sector.borrow().get_end() {
                    match wp {
                        Waypoint::Airport { airport, .. } => {
                            prev_airport_id = airport.get_id().to_string().clone();
                            prev = true;
                        }
                        _ => (),
                    }
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
            let _ = &self.refresh(None);
        }

        fn move_selected_up(&self) {
            let plan = self.plan.borrow_mut();
            let mut tree_path: Option<TreePath> = None;

            // See if a sector or waypoint is selected
            if let Some((model, iter)) = self.plan_tree.selection().selected() {
                let path = model.path(&iter).indices();
                // Sectors are at the top level
                match path.len() {
                    1 => (),
                    2 => {
                        let sector_index = path[0] as usize;
                        let wp_index = path[1] as usize;
                        let sectors = plan.get_sectors();
                        // Only if a waypoint.  index > 1 and < sectors.len() - 1
                        let sector = &sectors[sector_index];
                        let mut s_clone = sector.borrow().deref().clone();
                        if wp_index > 1
                            && wp_index < sectors[sector_index].borrow().get_waypoint_count() + 1
                        {
                            let i = wp_index - 1;
                            if let Some(wp) = s_clone.remove_waypoint(i) {
                                let _ = s_clone.insert_waypoint(i - 1, wp.clone());
                                tree_path =
                                    Some(TreePath::from_indices(&[sector_index as i32, i as i32]));
                            }
                            sector.replace(s_clone);
                        }
                    }
                    _ => {}
                }
            }
            drop(plan);
            let _ = &self.refresh(tree_path);
        }

        fn move_selected_down(&self) {
            let plan = self.plan.borrow_mut();
            let mut tree_path: Option<TreePath> = None;
            // See if a sector or waypoint is selected
            if let Some((model, iter)) = self.plan_tree.selection().selected() {
                let path = model.path(&iter).indices();
                // Sectors are at the top level
                match path.len() {
                    1 => (),
                    2 => {
                        let sector_index = path[0] as usize;
                        let wp_index = path[1] as usize;
                        let sectors = plan.get_sectors();
                        // Only if a waypoint.  index > 1 and < sectors.len() - 1
                        let sector = &sectors[sector_index];
                        let mut s_clone = sector.borrow().deref().clone();
                        if wp_index > 0
                            && wp_index < sectors[sector_index].borrow().get_waypoint_count()
                        {
                            let i = wp_index - 1;
                            if let Some(wp) = s_clone.remove_waypoint(i) {
                                let _ = s_clone.insert_waypoint(i + 1, wp.clone());
                                tree_path = Some(TreePath::from_indices(&[
                                    sector_index as i32,
                                    (i + 2) as i32,
                                ]));
                            }
                            sector.replace(s_clone);
                        }
                    }
                    _ => {}
                }
            }
            drop(plan);
            let _ = &self.refresh(tree_path);
        }

        fn remove_selected(&self) {
            let mut plan = self.plan.borrow_mut();
            let tree_path: Option<TreePath> = None;
            // See if a sector or waypoint is selected
            if let Some((model, iter)) = self.plan_tree.selection().selected() {
                let path = model.path(&iter).indices();
                // Sectors are at the top level
                match path.len() {
                    1 => {
                        let sector_index = path[0] as usize;
                        plan.remove_sector_at(sector_index);
                    }
                    2 => {
                        let sector_index = path[0] as usize;
                        let wp_index = path[1] as usize;
                        let sectors = plan.get_sectors();
                        // Only if a waypoint.  index > 1 and < sectors.len() - 1
                        let sector = &sectors[sector_index];
                        let mut s_clone = sector.borrow().deref().clone();
                        if wp_index == 0 {
                            let _ = s_clone.set_start(None);
                        } else if wp_index == sector.borrow().get_waypoint_count() + 1 {
                            let _ = s_clone.set_end(None);
                        } else {
                            let i = wp_index - 1;
                            let _ = s_clone.remove_waypoint(i);
                        }
                        sector.replace(s_clone);
                    }
                    _ => {}
                }
            }
            drop(plan);
            let _ = &self.refresh(tree_path);
        }

        fn get_selected_location(&self) -> Option<Coordinate> {
            let plan = self.plan.borrow();
            // See if a sector or waypoint is selected
            if let Some((model, iter)) = self.plan_tree.selection().selected() {
                let path = model.path(&iter).indices();
                // Sectors are at the top level
                match path.len() {
                    1 => {
                        None
                    }
                    2 => {
                        let sector_index = path[0] as usize;
                        let wp_index = path[1] as usize;
                        let sectors = plan.get_sectors();
                        // Only if a waypoint.  index > 1 and < sectors.len() - 1
                        let sector = &sectors[sector_index];
                        if wp_index == 0 {
                            if let Some(wp) = &sector.borrow().get_start() {
                                Some(wp.get_loc().clone())
                            } else {
                                None
                            }
                        } else if wp_index == sector.borrow().get_waypoint_count() + 1 {
                            if let Some(wp) = &sector.borrow().get_end() {
                                Some(wp.get_loc().clone())
                            } else {
                                None
                            }
                        } else {
                            let i = wp_index - 1;
                            if let Some(wp) = &sector.borrow().get_waypoint(i) {
                                Some(wp.get_loc().clone())
                            } else {
                                None
                            }
                        }
                    }
                    _ => None
                }
            } else {
                None
            }
        }

        fn get_selected_airport(&self) -> Option<Arc<Airport>> {
            let plan = self.plan.borrow();
            // See if a sector or waypoint is selected
            if let Some((model, iter)) = self.plan_tree.selection().selected() {
                let path = model.path(&iter).indices();
                // Sectors are at the top level
                match path.len() {
                    1 => { None }
                    2 => {
                        let sector_index = path[0] as usize;
                        let wp_index = path[1] as usize;
                        let sectors = plan.get_sectors();
                        let sector = &sectors[sector_index];
                        if wp_index == 0 {
                            if let Some(wp) = &sector.borrow().get_start() {
                                match wp {
                                    Waypoint::Airport { airport, .. } => {
                                        Some(airport.clone())
                                    }
                                    _ => None
                                }
                            } else {
                                None
                            }
                        } else if wp_index == sector.borrow().get_waypoint_count() + 1 {
                            if let Some(wp) = &sector.borrow().get_end() {
                                match wp {
                                    Waypoint::Airport { airport, .. } => {
                                        Some(airport.clone())
                                    }
                                    _ => None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            } else {
                None
            }
        }

        fn setup_aircraft_combo(&self) {
            let factory = SignalListItemFactory::new();
            factory.connect_setup(move |_, list_item| {
                let label = Label::new(None);
                list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .set_child(Some(&label));
            });

            let selection_model = SingleSelection::new(Some(get_hangar().clone()));
            self.aircraft_combo.set_factory(Some(&factory));
            self.aircraft_combo.set_model(Some(&selection_model));

            self.aircraft_combo.connect_selected_notify(clone!(@weak self.plan as plan => move | combo | {
                // Get the selectiion
                let index = combo.selected();
                if let Some(aircraft) = get_hangar().imp().aircraft_at(index) {
                    let mut plan = plan.borrow_mut();
                    plan.set_aircraft(&Some(aircraft));
                }
            }));

            factory.connect_bind(move |_, list_item| {
                // Get `IntegerObject` from `ListItem`
                let string_object = list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .item()
                    .and_downcast::<StringObject>()
                    .expect("The item has to be an `IntegerObject`.");

                // Get `Label` from `ListItem`
                let label = list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .child()
                    .and_downcast::<Label>()
                    .expect("The child has to be a `Label`.");

                // Set "label" to "number"
                label.set_label(&string_object.string().to_string());
                label.set_xalign(0.0);
            });

            // set the selection initially to the default
            let hangar = get_hangar().imp();
            let mut i = 0;
            for (_k, a) in hangar.get_all().read().expect("ould not get hangar lock").iter() {
                if *a.is_default() {
                    self.aircraft_combo.set_selected(i);
                }
                i += 1;
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
        fn constructed(&self) {
            self.parent_constructed();

            self.btn_make_plan
                .connect_clicked(clone!(@weak self as view => move |_search| {
                    view.make_plan();
                    view.refresh(None);
                }));

            let builder = Builder::from_resource("/com/shartrec/kelpie_planner/plan_popover.ui");
            let menu = builder.object::<MenuModel>("plan-menu");
            match menu {
                Some(popover) => {
                    let popover = PopoverMenu::builder()
                        .menu_model(&popover)
                        .build();
                    popover.set_parent(&self.plan_window.get());
                    let _ = self.popover.replace(Some(popover));
                }
                None => error!(" Not a popover"),
            }


            let gesture = gtk::GestureClick::new();
            gesture.set_button(3);
            gesture.connect_released(clone!(@weak self as view => move |gesture, _n, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                if let Some(popover) = view.popover.borrow().as_ref() {
                        popover.set_pointing_to(Some(&Rectangle::new(x as i32, y as i32, 1, 1)));
                        popover.popup();
                };
            }));
            self.plan_window.add_controller(gesture);

            self.btn_new_sector
                .connect_clicked(clone!(@weak self as view => move |_| {
                    view.new_sector();
                }));

            self.btn_move_up
                .connect_clicked(clone!(@weak self as view => move |_| {
                    view.move_selected_up();
                }));

            self.btn_move_down
                .connect_clicked(clone!(@weak self as view => move |_| {
                    view.move_selected_down();
                }));

            self.btn_max_alt
                .connect_toggled(clone!(@weak self as view => move | button | {
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

            self.max_alt.connect_changed(clone!(@weak self as view => move| editable | {
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


            self.plan_tree.connect_cursor_changed(clone!(@weak self as view => move |tree_view| {
                let plan = view.plan.borrow();
                view.btn_move_up.set_sensitive(false);
                view.btn_move_down.set_sensitive(false);
                if let Some((model, iter)) = tree_view.selection().selected() {
                    let path = model.path(&iter).indices();
                    if path.len() > 1 {
                        let sector_index = path[0] as usize;
                        let wp_index = path[1] as usize;
                        let sectors = plan.get_sectors();
                        // Only if a waypoint.  index > 0 and < sectors.len() - 1
                        if wp_index > 1 && wp_index < sectors[sector_index].borrow().get_waypoint_count() + 1 {
                            view.btn_move_up.set_sensitive(true);
                        }
                        if wp_index > 0 && wp_index < sectors[sector_index].borrow().get_waypoint_count() {
                            view.btn_move_down.set_sensitive(true);
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
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
               view.remove_selected();
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("view", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
               if let Some(airport) = view.get_selected_airport() {
                    match get_airport_map_view(&view.plan_window.get()) {
                        Some(airport_map_view) => {
                            show_airport_map_view(&view.plan_window.get());
                            airport_map_view.imp().set_airport(airport);
                        }
                        None => (),
                    }
                }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_airports_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                if let Some(loc) = view.get_selected_location() {
                    match get_airport_view(&view.plan_window.get()) {
                        Some(airport_view) => {
                            show_airport_view(&view.plan_window.get());
                            airport_view.imp().search_near(&loc);
                            ()
                        },
                        None => (),
                    }
               }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_navaids_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                if let Some(loc) = view.get_selected_location() {
                    match get_navaid_view(&view.plan_window.get()) {
                        Some(navaid_view) => {
                            show_navaid_view(&view.plan_window.get());
                            navaid_view.imp().search_near(&loc);
                            ()
                        },
                        None => (),
                    }
                    }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_fixes_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                if let Some(loc) = view.get_selected_location() {
                    match get_fix_view(&view.plan_window.get()) {
                        Some(fix_view) => {
                            show_fix_view(&view.plan_window.get());
                            fix_view.imp().search_near(&loc);
                            ()
                        },
                        None => (),
                    }
                }
            }));
            actions.add_action(&action);

            self.setup_aircraft_combo();

            self.initialise();
        }

        fn dispose(&self) {
            event::manager().unregister_listener(self.my_listener_id.borrow().deref());
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

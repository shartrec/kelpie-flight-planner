/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use gtk::gio;
use gtk::{self, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use std::ops::Deref;
    use std::sync::{Arc, RwLock};

    use glib::subclass::InitializingObject;
    use gtk::gdk::Rectangle;
    use gtk::gio::{ListStore, MenuModel, SimpleAction, SimpleActionGroup};
    use gtk::glib::clone;
    use gtk::{
        Builder, Button, DropDown, Label, ListItem, PopoverMenu, ScrolledWindow,
        SignalListItemFactory, SingleSelection, Stack, StringObject, TreePath, TreeStore, TreeView,
    };
    use log::error;

    use crate::earth;
    use crate::hangar::hangar::Hangar;
    use crate::model::airport::Airport;
    use crate::model::location::Location;
    use crate::model::plan::Plan;
    use crate::model::waypoint::Waypoint;
    use crate::planner::planner::Planner;
    use crate::preference::AUTO_PLAN;
    use crate::window::util::get_airport_map_view;

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

        pub plan: Arc<RwLock<Plan>>,
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
            let guard = self.plan.write().expect("Should always have a plan");
            guard.add_sector(None, None);
            guard.set_aircraft(&Hangar::get_hangar().get_default_aircraft());
        }

        pub fn get_plan(&self) -> Arc<RwLock<Plan>> {
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
            let plan = self.plan.read().expect("Can't get plan lock");
            for s_ref in plan.get_sectors().deref() {
                let row = tree_store.append(None);
                let binding = s_ref.borrow();
                let s = binding.deref();
                tree_store.set(&row, &[(0, &s.get_name())]);
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
                                &(format!("{:6.0}", plan.get_leg_heading_to(wp.deref()))),
                            ),
                            (
                                Col::Dist as u32,
                                &plan.get_leg_distance_to_as_string(wp.deref()),
                            ),
                            (Col::Time as u32, &plan.get_time_to_as_string(wp.deref())),
                            (Col::Speed as u32, &plan.get_speed_to_as_string(wp.deref())),
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
                            (
                                Col::Dist as u32,
                                &plan.get_leg_distance_to_as_string(&airport),
                            ),
                        ],
                    );
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
            let plan = self.plan.write().expect("Could not get plan lock");
            for s in plan.get_sectors().iter() {
                let waypoints = planner.make_plan(s.borrow().get_start(), s.borrow().get_end());
                s.borrow_mut().deref().add_all_waypoint(waypoints);
                planner.recalc_plan_elevations(&plan);
            }
            drop(plan);
        }

        pub fn initialise(&self) -> () {}

        pub fn add_airport_to_plan(&self, loc: Arc<Airport>) {
            let mut added = false;
            let plan = self.plan.write().expect("Can't get lock on plan");
            // See if a sector is selected
            if let Some((model, iter)) = self.plan_tree.selection().selected() {
                let path = model.path(&iter).indices();
                // Sectors are at the top level
                if path.len() == 1 {
                    let sector_index = path[0] as usize;
                    let sector = &plan.get_sectors()[sector_index];
                    if sector.borrow().get_start() == None {
                        sector.deref().borrow_mut().set_start(loc.clone());
                        added = true;
                    } else if sector.borrow().get_end() == None {
                        sector.deref().borrow_mut().set_end(loc.clone());
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
            let plan = self.plan.write().expect("Can't get lock on plan");
            // See if a sector or waypoint is selected
            if let Some((model, iter)) = self.plan_tree.selection().selected() {
                let path = model.path(&iter).indices();
                // Sectors are at the top level
                match path.len() {
                    1 => {
                        // a Sector only is selected
                        let sector_index = path[0] as usize;
                        let sector = &plan.get_sectors()[sector_index];
                        sector.deref().borrow_mut().add_waypoint(waypoint);
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
                            .deref()
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
            let plan = self.plan.write().expect("Can't get lock on plan");

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
            plan.add_sector(None, None);

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
            let plan = self.plan.write().expect("Can't get lock on plan");
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
                        if wp_index > 1
                            && wp_index < sectors[sector_index].borrow().get_waypoint_count() + 1
                        {
                            let i = wp_index - 1;
                            if let Some(wp) = &sector.borrow().remove_waypoint(i) {
                                let _ = &sector.borrow().insert_waypoint(i - 1, wp.clone());
                                tree_path =
                                    Some(TreePath::from_indices(&[sector_index as i32, i as i32]));
                            }
                        }
                    }
                    _ => {}
                }
            }
            drop(plan);
            let _ = &self.refresh(tree_path);
        }

        fn move_selected_down(&self) {
            let plan = self.plan.write().expect("Can't get lock on plan");
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
                        if wp_index > 0
                            && wp_index < sectors[sector_index].borrow().get_waypoint_count()
                        {
                            let i = wp_index - 1;
                            if let Some(wp) = &sector.borrow().remove_waypoint(i) {
                                let _ = &sector.borrow().insert_waypoint(i + 1, wp.clone());
                                tree_path = Some(TreePath::from_indices(&[
                                    sector_index as i32,
                                    (i + 2) as i32,
                                ]));
                            }
                        }
                    }
                    _ => {}
                }
            }
            drop(plan);
            let _ = &self.refresh(tree_path);
        }

        fn remove_selected(&self) {
            let plan = self.plan.write().expect("Can't get lock on plan");
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
                        if wp_index == 0 {
                            let _ = &sector.borrow().clear_start();
                        } else if wp_index == sector.borrow().get_waypoint_count() + 1 {
                            let _ = &sector.borrow().clear_end();
                        } else {
                            let i = wp_index - 1;
                            let _ = &sector.borrow().remove_waypoint(i);
                        }
                    }
                    _ => {}
                }
            }
            drop(plan);
            let _ = &self.refresh(tree_path);
        }

        fn view_airport(&self) {
            let plan = self.plan.write().expect("Can't get lock on plan");
            let tree_path: Option<TreePath> = None;
            // See if a sector or waypoint is selected
            if let Some((model, iter)) = self.plan_tree.selection().selected() {
                let path = model.path(&iter).indices();
                // Sectors are at the top level
                match path.len() {
                    1 => {}
                    2 => {
                        let sector_index = path[0] as usize;
                        let wp_index = path[1] as usize;
                        let sectors = plan.get_sectors();
                        let sector = &sectors[sector_index];
                        if wp_index == 0 {
                            if let Some(wp) = &sector.borrow().get_start() {
                                match wp {
                                    Waypoint::Airport { airport, .. } => {
                                        self.view_airport_int(airport.clone());
                                    }
                                    _ => {}
                                }
                            }
                        } else if wp_index == sector.borrow().get_waypoint_count() + 1 {
                            if let Some(wp) = &sector.borrow().get_end() {
                                match wp {
                                    Waypoint::Airport { airport, .. } => {
                                        self.view_airport_int(airport.clone());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            drop(plan);
            let _ = &self.refresh(tree_path);
        }

        fn view_airport_int(&self, airport: Arc<Airport>) {
            match get_airport_map_view(&self.plan_window.get()) {
                Some(airport_map_view) => {
                    airport_map_view.imp().set_airport(airport);
                }
                None => (),
            }
        }

        fn setup_aircraft_combo(&self) {
            let store = ListStore::new(StringObject::static_type());
            if let Ok(aircraft) = Hangar::get_hangar().get_all().read() {
                for a in aircraft.iter() {
                    store.append(&StringObject::new(&a.get_name()));
                }
            }

            let factory = SignalListItemFactory::new();
            factory.connect_setup(move |_, list_item| {
                let label = Label::new(None);
                list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .set_child(Some(&label));
            });

            let selection_model = SingleSelection::new(Some(store));
            self.aircraft_combo.set_factory(Some(&factory));
            self.aircraft_combo.set_model(Some(&selection_model));

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
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlanView {
        const NAME: &'static str = "PlanView";
        type Type = super::PlanView;
        type ParentType = gtk::Widget;

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

            let gesture = gtk::GestureClick::new();
            gesture.set_button(3);
            gesture.connect_released(clone!(@weak self as view => move |gesture, _n, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);

                let builder = Builder::from_resource("/com/shartrec/kelpie_planner/plan_popover.ui");
                let menu = builder.object::<MenuModel>("plan-menu");
                match menu {
                    Some(popover) => {
                        let popover = PopoverMenu::builder()
                            .menu_model(&popover)
                            .pointing_to(&Rectangle::new(x as i32, y as i32, 1, 1))
                            .build();
                        popover.set_parent(&view.plan_window.get());
                        popover.popup();
                    }
                    None => error!(" Not a popover"),
                }
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

            self.plan_tree.connect_cursor_changed(clone!(@weak self as view => move |tree_view| {
            let plan = view.plan.read().expect("Can't get lock on plan");

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
               view.view_airport();
            }));
            actions.add_action(&action);

            self.setup_aircraft_combo();

            self.initialise();
        }
    }

    impl WidgetImpl for PlanView {}
}

glib::wrapper! {
    pub struct PlanView(ObjectSubclass<imp::PlanView>) @extends gtk::Widget,
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

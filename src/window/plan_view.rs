/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use gtk::{TreeStore, TreeView};
use gtk::{self, CompositeTemplate, glib, prelude::*, subclass::prelude::*};
use gtk::gio;

mod imp {
    use std::ops::Deref;
    use std::sync::{Arc, RwLock};

    use glib::subclass::InitializingObject;
    use gtk::TreeStore;

    use crate::model::aircraft::Aircraft;
    use crate::model::plan::Plan;
    use crate::model::waypoint::Waypoint;
    use crate::planner::planner::Planner;

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/plan_view.ui")]
    pub struct PlanView {
        #[template_child]
        pub plan_tree: TemplateChild<TreeView>,
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
            let mut guard = self.plan.write().expect("Should always have a plan");
            guard.add_sector(None, None);
            guard.set_aircraft(&Some(Aircraft::new("Cessna".to_string(), 140, 7000, 110, 1000, 100, 500)));
        }

        pub fn get_plan(&self) -> Arc<RwLock<Plan>> {
            self.plan.clone()
        }

        pub fn plan_updated(&self) {

            let planner = Planner::new();
            let mut plan = self.plan.write().expect("Could not get plan lock");
            if plan.get_sectors().len() > 0 {
                let ref_cell = &plan.get_sectors()[0];
                let s0 = ref_cell;

                let waypoints = planner.make_plan(s0.borrow().deref().get_start(), s0.borrow().deref().get_end());
                s0.borrow_mut().deref().add_all_waypoint(waypoints);
                planner.recalc_plan_elevations(&plan);
            }
            drop(plan);


            // Load the view with a new tree model
            let tree_store = TreeStore::new(&[
                String::static_type(),
                i32::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type()
            ]);
            // Iterate over the plan loading the TreeModel
            let plan = self.plan.read().expect("Can't get plan lock");
            for s_ref in plan.get_sectors().deref() {
                let row = tree_store.append(None);
                let binding = s_ref.borrow();
                let s = binding.deref();
                tree_store.set(&row, &[
                    (0, &s.get_name())
                ]);
                if let Some(airport) = s.get_start() {
                    let wp_row = tree_store.append(Some(&row));
                    tree_store.set(&wp_row, &[
                        (Col::Name as u32, &airport.get_name()),
                        (Col::Elev as u32, &(airport.get_elevation() as i32)),
                        (Col::Lat as u32, &airport.get_lat_as_string()),
                        (Col::Long as u32, &airport.get_long_as_string()),
                    ]);
                }
                for wp in s.get_waypoints().read().expect("Can't get read lock on sectors").deref() {
                    let wp_row = tree_store.append(Some(&row));
                    let aircraft = plan.get_aircraft();
                    let speeds = match aircraft.deref() {
                        Some(a) => (a.get_climb_speed(), a.get_cruise_speed(), a.get_sink_speed()),
                        None => (&100, &140, &80),
                    };
                    tree_store.set(&wp_row, &[
                        (Col::Name as u32, &wp.get_name()),
                        (Col::Elev as u32, &(wp.get_elevation() as i32)),
                        (Col::Lat as u32, &wp.get_lat_as_string()),
                        (Col::Long as u32, &wp.get_long_as_string()),
                        (Col::Freq as u32, &(match wp.get_freq() {
                            Some(f) => format!("{:>6.2}", f),
                            None => "".to_string(),
                        })),
                        (Col::Hdg as u32, &(format!("{:6.0}", plan.get_leg_heading_to(wp.deref())))),
                        (Col::Dist as u32, &plan.get_leg_distance_to_as_string(wp.deref())),
                        (Col::Time as u32, &plan.get_time_to_as_string(wp.deref(), speeds.0, speeds.1, speeds.2)),
                        (Col::Speed as u32, &plan.get_speed_to_as_string(wp.deref(),speeds.0, speeds.1, speeds.2)),
                    ]);
                }
                if let Some(airport) = s.get_end() {
                    let wp_row = tree_store.append(Some(&row));
                    tree_store.set(&wp_row, &[
                        (Col::Name as u32, &airport.get_name()),
                        (Col::Elev as u32, &(airport.get_elevation() as i32)),
                        (Col::Lat as u32, &airport.get_lat_as_string()),
                        (Col::Long as u32, &airport.get_long_as_string()),
                        (Col::Dist as u32, &plan.get_leg_distance_to_as_string(&airport)),
                    ]);
                }

            }

            self.plan_tree.set_model(Some(&tree_store));
            self.plan_tree.expand_all();
        }

        pub fn initialise(&self) -> () {}
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

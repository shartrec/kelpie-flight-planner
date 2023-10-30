/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use gtk::{self, CompositeTemplate, glib, prelude::*, subclass::prelude::*};

mod imp {
    use std::cell::Cell;

    use glib::subclass::InitializingObject;
    use gtk::{Builder, Button, Entry, ListStore, PopoverMenu, ScrolledWindow, TreeView};
    use gtk::gdk::Rectangle;
    use gtk::gio::{MenuModel, SimpleAction, SimpleActionGroup};
    use gtk::glib::clone;
    use log::error;

    use crate::earth;
    use crate::earth::coordinate::Coordinate;
    use crate::model::location::Location;
    use crate::model::waypoint::Waypoint;
    use crate::util::lat_long_format::LatLongFormat;
    use crate::util::location_filter::{CombinedFilter, Filter, NameIdFilter, RangeFilter};
    use crate::window::util::{
        get_airport_view, get_fix_view, get_plan_view, show_airport_view, show_error_dialog,
        show_fix_view,
    };

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/navaid_view.ui")]
    pub struct NavaidView {
        #[template_child]
        pub navaid_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub navaid_list: TemplateChild<TreeView>,
        #[template_child]
        pub navaid_search_name: TemplateChild<Entry>,
        #[template_child]
        pub navaid_search_lat: TemplateChild<Entry>,
        #[template_child]
        pub navaid_search_long: TemplateChild<Entry>,
        #[template_child]
        pub navaid_search: TemplateChild<Button>,
    }

    impl NavaidView {
        pub fn initialise(&self) -> () {}

        pub fn navaids_loaded(&self) {
            self.navaid_search.set_sensitive(true);
        }

        pub fn search(&self) {
            let term = self.navaid_search_name.text();
            let lat = self.navaid_search_lat.text();
            let long = self.navaid_search_long.text();

            let mut combined_filter = CombinedFilter::new();
            if !term.is_empty() {
                if let Some(filter) = NameIdFilter::new(term.as_str()) {
                    combined_filter.add(Box::new(filter));
                }
            }
            if !lat.is_empty() || !long.is_empty() {
                if lat.is_empty() || long.is_empty() {
                    //show message popup
                    show_error_dialog(
                        &self.obj().root(),
                        "Enter both a Latitude and Longitude for search.",
                    );
                    return;
                } else {
                    let lat_as_float;
                    let lat_format = LatLongFormat::lat_format();
                    match lat_format.parse(lat.as_str()) {
                        Ok(latitude) => lat_as_float = latitude,
                        Err(_) => {
                            show_error_dialog(&self.obj().root(), "Invalid Latitude for search.");
                            return;
                        }
                    }
                    let long_as_float;
                    let long_format = LatLongFormat::long_format();
                    match long_format.parse(long.as_str()) {
                        Ok(longitude) => long_as_float = longitude,
                        Err(_) => {
                            show_error_dialog(&self.obj().root(), "Invalid Latitude for search.");
                            return;
                        }
                    }
                    if let Some(filter) = RangeFilter::new(lat_as_float, long_as_float, 100.0) {
                        combined_filter.add(Box::new(filter));
                    }
                }
            }

            let navaids = earth::get_earth_model().get_navaids().read().unwrap();
            let searh_result = navaids.iter().filter(move |a| {
                let navaid: &dyn Location = &***a;
                combined_filter.filter(navaid)
            });
            let store = ListStore::new(&[
                String::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
                i32::static_type(),
            ]);
            for navaid in searh_result {
                store.insert_with_values(
                    None,
                    &[
                        (0, &navaid.get_id()),
                        (1, &navaid.get_name()),
                        (2, &navaid.get_lat_as_string()),
                        (3, &navaid.get_long_as_string()),
                        (4, &(navaid.get_elevation())),
                    ],
                );
            }
            self.navaid_list.set_model(Some(&store));
        }

        fn add_selected_to_plan(&self) {
            if let Some((model, iter)) = self.navaid_list.selection().selected() {
                let id = TreeModelExtManual::get::<String>(&model, &iter, 0);
                let name = TreeModelExtManual::get::<String>(&model, &iter, 1);
                if let Some(navaid) =
                    earth::get_earth_model().get_navaid_by_id_and_name(id.as_str(), name.as_str())
                {
                    match get_plan_view(&self.navaid_window.get()) {
                        Some(view) => {
                            // get the plan
                            view.imp().add_waypoint_to_plan(Waypoint::Navaid {
                                navaid: navaid.clone(),
                                elevation: Cell::new(0),
                                locked: true,
                            });
                        }
                        None => (),
                    }
                }
            }
        }

        pub fn search_near(&self, coordinate: &Coordinate) {
            self.navaid_search_lat
                .set_text(&coordinate.get_latitude_as_string());
            self.navaid_search_long
                .set_text(&coordinate.get_longitude_as_string());
            self.navaid_search.activate();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NavaidView {
        const NAME: &'static str = "NavaidView";
        type Type = super::NavaidView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NavaidView {
        fn constructed(&self) {
            self.parent_constructed();
            self.initialise();

            self.navaid_list.connect_row_activated(
                clone!(@weak self as view => move |_list_view, _position, _col| {
                    view.add_selected_to_plan()
                }),
            );

            let gesture = gtk::GestureClick::new();
            gesture.set_button(3);
            gesture.connect_released(clone!(@weak self as view => move |gesture, _n, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);

                let builder = Builder::from_resource("/com/shartrec/kelpie_planner/navaid_popover.ui");
                let menu = builder.object::<MenuModel>("navaids-menu");
                match menu {
                    Some(popover) => {
                        let popover = PopoverMenu::builder()
                            .menu_model(&popover)
                            .pointing_to(&Rectangle::new(x as i32, y as i32, 1, 1))
                            .build();
                        popover.set_parent(&view.navaid_window.get());
                        popover.popup();
                    }
                    None => error!(" Not a popover"),
                }
            }));
            self.navaid_window.add_controller(gesture);

            self.navaid_search
                .connect_clicked(clone!(@weak self as window => move |_search| {
                    window.search();
                }));
            self.navaid_search_name.connect_activate(
                clone!(@weak self as window => move |_search| {
                    window.search();
                }),
            );
            self.navaid_search_lat.connect_activate(
                clone!(@weak self as window => move |_search| {
                    window.search();
                }),
            );
            self.navaid_search_long.connect_activate(
                clone!(@weak self as window => move |_search| {
                    window.search();
                }),
            );

            let actions = SimpleActionGroup::new();
            self.navaid_window
                .get()
                .insert_action_group("navaid", Some(&actions));
            let action = SimpleAction::new("add_to_plan", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
               view.navaid_list.activate();
            }));

            let action = SimpleAction::new("find_airports_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                if let Some((model, iter)) = view.navaid_list.selection().selected() {
                    let id = TreeModelExtManual::get::<String>(&model, &iter, 0);
                    let name = TreeModelExtManual::get::<String>(&model, &iter, 1);
                    if let Some(navaid) = earth::get_earth_model().get_navaid_by_id_and_name(id.as_str(), name.as_str()) {
                        match get_airport_view(&view.navaid_window.get()) {
                            Some(airport_view) => {
                                show_airport_view(&view.navaid_window.get());
                                airport_view.imp().search_near(&navaid.get_loc());
                                ()
                            },
                            None => (),
                        }
                    }
               }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_fixes_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                if let Some((model, iter)) = view.navaid_list.selection().selected() {
                    let id = TreeModelExtManual::get::<String>(&model, &iter, 0);
                    let name = TreeModelExtManual::get::<String>(&model, &iter, 1);
                    if let Some(navaid) = earth::get_earth_model().get_navaid_by_id_and_name(id.as_str(), name.as_str()) {
                        match get_fix_view(&view.navaid_window.get()) {
                            Some(fix_view) => {
                                show_fix_view(&view.navaid_window.get());
                                fix_view.imp().search_near(&navaid.get_loc());
                                ()
                            },
                            None => (),
                        }
                    }
               }
            }));

            actions.add_action(&action);
            let action = SimpleAction::new("add_to_plan", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                view.add_selected_to_plan()
            }));
            actions.add_action(&action);
        }
    }

    impl BoxImpl for NavaidView {}

    impl WidgetImpl for NavaidView {}
}

glib::wrapper! {
    pub struct NavaidView(ObjectSubclass<imp::NavaidView>) @extends gtk::Widget, gtk::Box;
}

impl NavaidView {
    pub fn new() -> Self {
        glib::Object::new::<NavaidView>()
    }
}

impl Default for NavaidView {
    fn default() -> Self {
        Self::new()
    }
}

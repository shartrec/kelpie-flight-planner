/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use gtk::{self, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use gtk::{ListStore, TreeView};

mod imp {
    use std::boxed::Box;

    use glib::subclass::InitializingObject;
    use gtk::gdk::Rectangle;
    use gtk::gio::{MenuModel, SimpleAction, SimpleActionGroup};
    use gtk::glib::clone;
    use gtk::{Builder, Button, Entry, PopoverMenu, ScrolledWindow};
    use log::error;

    use crate::earth;
    use crate::earth::coordinate::Coordinate;
    use crate::model::location::Location;
    use crate::util::lat_long_format::LatLongFormat;
    use crate::util::location_filter::{CombinedFilter, Filter, NameIdFilter, RangeFilter};
    use crate::window::util::{
        get_airport_map_view, get_fix_view, get_navaid_view, get_plan_view, show_error_dialog,
        show_fix_view, show_navaid_view,
    };

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/airport_view.ui")]
    pub struct AirportView {
        #[template_child]
        pub airport_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub airport_list: TemplateChild<TreeView>,
        #[template_child]
        pub airport_search_name: TemplateChild<Entry>,
        #[template_child]
        pub airport_search_lat: TemplateChild<Entry>,
        #[template_child]
        pub airport_search_long: TemplateChild<Entry>,
        #[template_child]
        pub airport_search: TemplateChild<Button>,
    }

    impl AirportView {
        pub fn initialise(&self) -> () {}

        pub fn airports_loaded(&self) {
            self.airport_search.set_sensitive(true);
        }

        pub fn search(&self) {
            // CHeck we have sensible search criteria
            let term = self.airport_search_name.text();
            let lat = self.airport_search_lat.text();
            let long = self.airport_search_long.text();

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
            let airports = earth::get_earth_model().get_airports().read().unwrap();
            let searh_result = airports.iter().filter(move |a| {
                let airport: &dyn Location = &***a;
                combined_filter.filter(airport)
            });
            let store = ListStore::new(&[
                String::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
                i32::static_type(),
            ]);

            for airport in searh_result {
                store.insert_with_values(
                    None,
                    &[
                        (0, &airport.get_id()),
                        (1, &airport.get_name()),
                        (2, &airport.get_lat_as_string()),
                        (3, &airport.get_long_as_string()),
                        (4, &(airport.get_elevation())),
                    ],
                );
            }
            self.airport_list.set_model(Some(&store));
        }

        fn add_selected_to_plan(&self) {
            if let Some((model, iter)) = self.airport_list.selection().selected() {
                let name = TreeModelExtManual::get::<String>(&model, &iter, 0);
                if let Some(airport) = earth::get_earth_model().get_airport_by_id(name.as_str()) {
                    match get_plan_view(&self.airport_window.get()) {
                        Some(ref mut plan_view) => {
                            // get the plan
                            plan_view.imp().add_airport_to_plan(airport);
                            ()
                        }
                        None => (),
                    }
                }
            }
        }

        pub fn search_near(&self, coordinate: &Coordinate) {
            self.airport_search_lat
                .set_text(&coordinate.get_latitude_as_string());
            self.airport_search_long
                .set_text(&coordinate.get_longitude_as_string());
            self.airport_search.activate();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AirportView {
        const NAME: &'static str = "AirportView";
        type Type = super::AirportView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AirportView {
        fn constructed(&self) {
            self.parent_constructed();
            self.initialise();

            self.airport_list.connect_row_activated(
                clone!(@weak self as view => move | _list_view, _position, _col| {
                    view.add_selected_to_plan();
                }),
            );

            let gesture = gtk::GestureClick::new();
            gesture.set_button(3);
            gesture.connect_released(clone!(@weak self as view => move |gesture, _n, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);

                let builder = Builder::from_resource("/com/shartrec/kelpie_planner/airport_popover.ui");
                let menu = builder.object::<MenuModel>("airports-menu");
                match menu {
                    Some(popover) => {
                        let popover = PopoverMenu::builder()
                            .menu_model(&popover)
                            .pointing_to(&Rectangle::new(x as i32, y as i32, 1, 1))
                            .build();
                        popover.set_parent(&view.airport_window.get());
                        popover.popup();
                    }
                    None => error!(" Not a popover"),
                }
            }));
            self.airport_window.add_controller(gesture);

            // If the user clicks search or pressses enter in any of the search fields do the search
            self.airport_search
                .connect_clicked(clone!(@weak self as window => move |_search| {
                    window.search();
                }));
            self.airport_search_name.connect_activate(
                clone!(@weak self as window => move |_search| {
                    window.search();
                }),
            );
            self.airport_search_lat.connect_activate(
                clone!(@weak self as window => move |_search| {
                    window.search();
                }),
            );
            self.airport_search_long.connect_activate(
                clone!(@weak self as window => move |_search| {
                    window.search();
                }),
            );

            let actions = SimpleActionGroup::new();
            self.airport_window
                .get()
                .insert_action_group("airport", Some(&actions));

            let action = SimpleAction::new("add_to_plan", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                view.add_selected_to_plan();
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_navaids_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                if let Some((model, iter)) = view.airport_list.selection().selected() {
                    let name = TreeModelExtManual::get::<String>(&model, &iter, 0);
                    if let Some(airport) = earth::get_earth_model().get_airport_by_id(name.as_str()) {
                        match get_navaid_view(&view.airport_window.get()) {
                            Some(navaid_view) => {
                                show_navaid_view(&view.airport_window.get());
                                navaid_view.imp().search_near(&airport.get_loc());
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
                if let Some((model, iter)) = view.airport_list.selection().selected() {
                    let name = TreeModelExtManual::get::<String>(&model, &iter, 0);
                    if let Some(airport) = earth::get_earth_model().get_airport_by_id(name.as_str()) {
                        match get_fix_view(&view.airport_window.get()) {
                            Some(fix_view) => {
                                show_fix_view(&view.airport_window.get());
                                fix_view.imp().search_near(&airport.get_loc());
                                ()
                            },
                            None => (),
                        }
                    }
               }
            }));

            actions.add_action(&action);
            let action = SimpleAction::new("view", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                if let Some((model, iter)) = view.airport_list.selection().selected() {
                    let name = TreeModelExtManual::get::<String>(&model, &iter, 0);
                    if let Some(airport) = earth::get_earth_model().get_airport_by_id(name.as_str()) {
                        match get_airport_map_view(&view.airport_window.get()) {
                            Some(airport_map_view) => {
                                airport_map_view.imp().set_airport(airport);
                            },
                            None => (),
                        }
                    }
               }
            }));
            actions.add_action(&action);
        }
    }

    impl WidgetImpl for AirportView {}
}

glib::wrapper! {
    pub struct AirportView(ObjectSubclass<imp::AirportView>) @extends gtk::Widget;
}

impl AirportView {
    pub fn new() -> Self {
        glib::Object::new::<AirportView>()
    }
}

impl Default for AirportView {
    fn default() -> Self {
        Self::new()
    }
}

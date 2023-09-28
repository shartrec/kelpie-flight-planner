/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use gtk::{Button, Entry, ListStore, TreeView};
use gtk::{self, CompositeTemplate, glib, prelude::*, subclass::prelude::*};
use crate::window::Window;

mod imp {
    use std::boxed::Box;
    use std::ops::Deref;
    use std::sync::{Arc, RwLock};

    use glib::subclass::InitializingObject;
    use gtk::{Button, Entry, ScrolledWindow};
    use gtk::glib::clone;

    use crate::earth;
    use crate::model::location::Location;
    use crate::model::plan::Plan;
    use crate::util::lat_long_format::LatLongFormat;
    use crate::util::location_filter::{CombinedFilter, Filter, NameIdFilter, RangeFilter};
    use crate::window::plan_view::PlanView;
    use crate::window::util::show_error_dialog;
    use crate::window::Window;

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
                    show_error_dialog(&self.obj().root(), "Enter both a Latitude and Longitude for search.");
                    return;
                } else {
                    let mut lat_as_float = 0.0;
                    let lat_format = LatLongFormat::lat_format();
                    match lat_format.parse(lat.as_str()) {
                        Ok(latitude) => lat_as_float = latitude,
                        Err(_) => {
                            show_error_dialog(&self.obj().root(), "Invalid Latitude for search.");
                            return;
                        }
                    }
                    let mut long_as_float = 0.0;
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
            let store = ListStore::new(&[String::static_type(), String::static_type(), String::static_type(), String::static_type(), i32::static_type()]);

            for airport in searh_result {
                store.insert_with_values(
                    None, &[
                        (0, &airport.get_id()),
                        (1, &airport.get_name()),
                        (2, &airport.get_lat_as_string()),
                        (3, &airport.get_long_as_string()),
                        (4, &(airport.get_elevation()))
                    ]);
            }
            self.airport_list.set_model(Some(&store));
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

            self.airport_list.connect_row_activated(clone!(@weak self as view => move |list_view, position, col| {
                let model = list_view.model().expect("The model has to exist.");
                if let iter = model.iter(position) {
                    let name = model.get::<String>(&iter.unwrap(), 0);
                    if let Some(airport) = earth::get_earth_model().get_airport_by_id(name.as_str()) {
                        match view.obj().root() {
                            Some(r) => {
                                let our_window = r.downcast::<Window>().unwrap();
                                match our_window.imp().plan_stack.visible_child().and_downcast::<PlanView>() {
                                    Some(plan_view) => {
                                        // get the plan
                                        let plan: Arc<RwLock<Plan>> = plan_view.imp().get_plan();
                                        plan.write().expect("Could not get plan lock").add_airport(*airport);
                                        plan_view.imp().plan_updated();
                                        ()
                                    },
                                    None => (),
                                }
                                ()
                            }
                            None => (),
                        }
                   }
                }
            }));

            let gesture = gtk::GestureClick::new();
            gesture.set_button(3);
            gesture.connect_released(clone!(@weak self as view => move |gesture, _n, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                println!("Button '{}' pressed! at ({}, {})", gesture.button(),x,y);
                if let Some((model, iter)) = view.airport_list.selection().selected() {
                    println!("selected airport: {}", model.get::<String>(&iter, 0));
                }
            }));
            self.airport_list.add_controller(gesture);

            let gesture = gtk::EventControllerKey::new();
            gesture.connect_key_released(clone!(@weak self as view => move |controller, key_code, key_type, modifiers| {
                println!("Key pressed code:'{}' val:'{}', Modifiers:{})", key_code, key_type, modifiers);
                if let Some((model, iter)) = view.airport_list.selection().selected() {
                    println!("selected airport: {}", model.get::<String>(&iter, 0));
                }
            }));
            self.airport_list.add_controller(gesture);

            self.airport_search.connect_clicked(
                clone!(@weak self as window => move |search| {
                window.search();
            }));
            self.airport_search_name.connect_activate(
                clone!(@weak self as window => move |search| {
                window.search();
            }));
            self.airport_search_lat.connect_activate(
                clone!(@weak self as window => move |search| {
                window.search();
            }));
            self.airport_search_long.connect_activate(
                clone!(@weak self as window => move |search| {
                window.search();
            }));
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


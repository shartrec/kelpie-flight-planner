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

mod imp {
    use std::boxed::Box;
    use std::cell::RefCell;
    use std::ops::Deref;
    use std::sync::Arc;

    use glib::subclass::InitializingObject;
    use gtk::{Builder, Button, ColumnView, ColumnViewColumn, CustomFilter, Entry, FilterChange, FilterListModel, Label, PopoverMenu, ScrolledWindow, SingleSelection};
    use gtk::gdk::Rectangle;
    use gtk::gio::{MenuModel, SimpleAction, SimpleActionGroup};
    use gtk::glib::{clone, MainContext, PRIORITY_DEFAULT};
    use log::error;

    use crate::event;
    use crate::earth::airport_list_model::Airports;
    use crate::earth::coordinate::Coordinate;
    use crate::event::Event;
    use crate::model::airport::Airport;
    use crate::model::airport_object::AirportObject;
    use crate::model::location::Location;
    use crate::util::lat_long_format::LatLongFormat;
    use crate::util::location_filter::{CombinedFilter, NameIdFilter, new_airport_filter, NilFilter, RangeFilter, set_airport_filter};
    use crate::window::util::{build_column_factory, get_airport_map_view, get_fix_view, get_navaid_view, get_plan_view, show_airport_map_view, show_error_dialog, show_fix_view, show_navaid_view};

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/airport_view.ui")]
    pub struct AirportView {
        #[template_child]
        pub airport_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub airport_list: TemplateChild<ColumnView>,
        #[template_child]
        pub col_id: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_name: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_lat: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_lon: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_elev: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub airport_search_name: TemplateChild<Entry>,
        #[template_child]
        pub airport_search_lat: TemplateChild<Entry>,
        #[template_child]
        pub airport_search_long: TemplateChild<Entry>,
        #[template_child]
        pub airport_search: TemplateChild<Button>,

        popover: RefCell<Option<PopoverMenu>>,
        my_listener_id: RefCell<usize>,
        filter_list_model: RefCell<Option<FilterListModel>>,

    }

    impl AirportView {
        pub fn initialise(&self) -> () {
            let (tx, rx) = MainContext::channel(PRIORITY_DEFAULT);
            let index = event::manager().register_listener(tx);
            rx.attach(None,clone!(@weak self as view => @default-return glib::source::Continue(true), move |ev: Event| {
                match ev {
                    Event::AirportsLoaded => {
                        view.airport_search.set_sensitive(true);

                        let fm = FilterListModel::new(Some(Airports::new()), Some(new_airport_filter(Box::new(NilFilter::new()))));

                        view.filter_list_model.replace(Some(fm.clone()));

                        let selection_model = SingleSelection::new(Some(fm));
                        selection_model.set_autoselect(false);
                        view.airport_list.set_model(Some(&selection_model));
                    },
                    _ => (),
                }
                glib::source::Continue(true)
            }));
            // self.my_listener.replace(Some(rx));
            self.my_listener_id.replace(index);

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

            let custom_filter = self.filter_list_model.borrow().as_ref().unwrap().filter().unwrap().downcast::<CustomFilter>().unwrap();

            self.airport_list.model().unwrap().unselect_all();
            set_airport_filter(&custom_filter, Box::new(combined_filter));
            custom_filter.changed(FilterChange::Different);
        }

        fn add_selected_to_plan(&self) {
            if let Some(airport) = self.get_selection() {
                self.add_to_plan(airport);
            }
        }

        fn add_item_to_plan(&self, item: u32) {
            if let Some(airport) = self.get_model_airport(item) {
                self.add_to_plan(airport);
            }
        }

        fn add_to_plan(&self, airport: Arc<Airport>) {
            match get_plan_view(&self.airport_window.get()) {
                Some(ref mut plan_view) => {
                    // get the plan
                    plan_view.imp().add_airport_to_plan(airport.clone());
                    ()
                }
                None => (),
            }
        }

        fn get_selection(&self) -> Option<Arc<Airport>> {
            let selection = self.airport_list.model().unwrap().selection();
            let sel_ap = selection.nth(0);
            self.get_model_airport(sel_ap)
        }

        fn get_model_airport(&self, sel_ap: u32) -> Option<Arc<Airport>> {
            let selection = self.airport_list.model().unwrap().item(sel_ap);
            if let Some(object) = selection {
                let airport_object = object.downcast::<AirportObject>()
                    .expect("The item has to be an `Airport`.");

                Some(airport_object.imp().airport())
            } else {
                None
            }
        }

        pub fn search_near(&self, coordinate: &Coordinate) {
            self.airport_search_name
                .set_text("");
            self.airport_search_lat
                .set_text(&coordinate.get_latitude_as_string());
            self.airport_search_long
                .set_text(&coordinate.get_longitude_as_string());
            self.airport_search.emit_clicked();
        }

    }

    #[glib::object_subclass]
    impl ObjectSubclass for AirportView {
        const NAME: &'static str = "AirportView";
        type Type = super::AirportView;
        type ParentType = gtk::Box;

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

            let f = |label: Label, airport: &AirportObject|{
                label.set_label(&airport.imp().airport().get_id());
                label.set_xalign(0.0);
            };
            let factory = build_column_factory(f);
            self.col_id.set_factory(Some(&factory));

            let f = |label: Label, airport: &AirportObject|{
                label.set_label(&airport.imp().airport().get_name());
                label.set_xalign(0.0);
            };
            let factory = build_column_factory(f);
            self.col_name.set_factory(Some(&factory));

            let f = |label: Label, airport: &AirportObject|{
                label.set_label(&airport.imp().airport().get_lat_as_string());
                label.set_xalign(0.0);
            };
            let factory = build_column_factory(f);
            self.col_lat.set_factory(Some(&factory));

            let f = |label: Label, airport: &AirportObject|{
                label.set_label(&airport.imp().airport().get_long_as_string());
                label.set_xalign(0.0);
            };
            let factory = build_column_factory(f);
            self.col_lon.set_factory(Some(&factory));

            let f = |label: Label, airport: &AirportObject|{
                label.set_label(&airport.imp().airport().get_elevation().to_string());
                label.set_xalign(1.0);
            };
            let factory = build_column_factory(f);
            self.col_elev.set_factory(Some(&factory));

            self.airport_list.connect_activate(
                clone!(@weak self as view => move | _list_view, position | {
                    view.add_item_to_plan(position);
                }),
            );

            // build popover menu
            let builder = Builder::from_resource("/com/shartrec/kelpie_planner/airport_popover.ui");
            let menu = builder.object::<MenuModel>("airports-menu");
            match menu {
                Some(popover) => {
                    let popover = PopoverMenu::builder()
                        .menu_model(&popover)
                        .build();
                    popover.set_parent(&self.airport_list.get());
                    let _ = self.popover.replace(Some(popover));
                }
                None => error!(" Not a popover"),
            }

            let gesture = gtk::GestureClick::new();
            gesture.set_button(3);
            gesture.connect_released(clone!(@weak self as view => move | gesture, _n, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                if let Some(popover) = view.popover.borrow().as_ref() {
                        popover.set_pointing_to(Some(&Rectangle::new(x as i32, y as i32, 1, 1)));
                        popover.popup();
                };
            }));
            self.airport_list.add_controller(gesture);

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

            let action = SimpleAction::new("find_airports_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                    if let Some(airport) = view.get_selection() {
                        view.search_near(airport.get_loc());
                    }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_navaids_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                    if let Some(airport) = view.get_selection() {
                        match get_navaid_view(&view.airport_window.get()) {
                            Some(navaid_view) => {
                                show_navaid_view(&view.airport_window.get());
                                navaid_view.imp().search_near(&airport.get_loc());
                                ()
                            },
                            None => (),
                        }
                    }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_fixes_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                if let Some(airport) = view.get_selection() {
                        match get_fix_view(&view.airport_window.get()) {
                            Some(fix_view) => {
                                show_fix_view(&view.airport_window.get());
                                fix_view.imp().search_near(airport.get_loc());
                                ()
                            },
                            None => (),
                        }
                    }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("view", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                if let Some(airport) = view.get_selection() {
                        match get_airport_map_view(&view.airport_window.get()) {
                            Some(airport_map_view) => {
                                show_airport_map_view(&view.airport_window.get());
                                airport_map_view.imp().set_airport(airport.clone());
                            },
                            None => (),
                        }
                    }
            }));
            actions.add_action(&action);
        }

        fn dispose(&self) {
            if let Some(popover) = self.popover.borrow().as_ref() {
                popover.unparent();
            };
            event::manager().unregister_listener(self.my_listener_id.borrow().deref());
        }
    }

    impl BoxImpl for AirportView {}
    impl WidgetImpl for AirportView {}
}

glib::wrapper! {
    pub struct AirportView(ObjectSubclass<imp::AirportView>) @extends gtk::Widget, gtk::Box;
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

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

use gtk::{self, CompositeTemplate, glib, prelude::*, subclass::prelude::*};

mod imp {
    use std::boxed::Box;
    use std::cell::RefCell;
    use std::sync::Arc;

    use glib::subclass::InitializingObject;
    use gtk::{Builder, Button, ColumnView, ColumnViewColumn, CustomFilter, CustomSorter,
              Entry, FilterChange, FilterListModel, Label, Ordering, PopoverMenu, ScrolledWindow,
              SingleSelection, SortListModel};
    use gtk::gdk::{Key, ModifierType, Rectangle};
    use gtk::gio::{MenuModel, SimpleAction, SimpleActionGroup};
    use gtk::glib::{clone, MainContext};
    use log::error;

    use crate::earth::airport_list_model::Airports;
    use crate::earth::coordinate::Coordinate;
    use crate::event;
    use crate::event::Event;
    use crate::glib::Propagation;
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
        filter_list_model: RefCell<Option<FilterListModel>>,

    }

    impl AirportView {
        pub fn initialise(&self) {
            // Add a sorter
            self.col_id.set_sorter(Some(&Self::get_id_sorter()));
            self.col_name.set_sorter(Some(&Self::get_name_sorter()));
            self.col_lat.set_sorter(Some(&Self::get_lat_sorter()));
            self.col_lon.set_sorter(Some(&Self::get_long_sorter()));

            let sorter = self.airport_list.sorter();

            let fm = FilterListModel::new(Some(Airports::new()), Some(new_airport_filter(Box::new(NilFilter::new()))));
            self.filter_list_model.replace(Some(fm.clone()));

            let slm = SortListModel::new(Some(fm), sorter);
            slm.set_incremental(true);

            let selection_model = SingleSelection::new(Some(slm));
            selection_model.set_autoselect(false);
            self.airport_list.set_model(Some(&selection_model));
            self.airport_list.set_single_click_activate(true);

            if let Some(rx) = event::manager().register_listener() {
                MainContext::default().spawn_local(clone!(#[weak(rename_to = view)] self, async move {
                    while let Ok(ev) = rx.recv().await {
                        if let Event::AirportsLoaded = ev {
                            view.airport_search.set_sensitive(true);
                        }
                    }
                }));
            }
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
                    let lat_format = LatLongFormat::lat_format();
                    let lat_as_float = match lat_format.parse(lat.as_str()) {
                        Ok(latitude) => latitude,
                        Err(_) => {
                            show_error_dialog(&self.obj().root(), "Invalid Latitude for search.");
                            return;
                        }
                    };
                    let long_format = LatLongFormat::long_format();
                    let long_as_float = match long_format.parse(long.as_str()) {
                        Ok(longitude) => longitude,
                        Err(_) => {
                            show_error_dialog(&self.obj().root(), "Invalid Longitude for search.");
                            return;
                        }
                    };
                    if let Some(filter) = RangeFilter::new(lat_as_float, long_as_float, 100.0) {
                        combined_filter.add(Box::new(filter));
                    }
                }
            }

            if let Some(filter_ref) = self.filter_list_model.borrow().as_ref() {
                let custom_filter = filter_ref.filter().unwrap().downcast::<CustomFilter>().unwrap();

                self.airport_list.model().unwrap().unselect_all();
                set_airport_filter(&custom_filter, Box::new(combined_filter));
                custom_filter.changed(FilterChange::Different);
            }
        }

        fn add_to_plan(&self, airport: Arc<Airport>) {
            if let Some(ref mut plan_view) = get_plan_view(&self.airport_window.get()) {
                // get the plan
                plan_view.imp().add_airport_to_plan(airport.clone());
            }
        }

        fn get_selected_airport(&self) -> Option<Arc<Airport>> {
            self.get_selection().map(|airport| airport.imp().airport().clone())
        }

        fn get_selection(&self) -> Option<AirportObject> {
            let selection = self.airport_list.model().unwrap().selection();
            let sel_ap = selection.nth(0);
            self.get_model_airport(sel_ap)
        }

        fn get_model_airport(&self, sel_ap: u32) -> Option<AirportObject> {
            let selection = self.airport_list.model().unwrap().item(sel_ap);
            if let Some(object) = selection {
                object.downcast::<AirportObject>().ok()
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

        fn get_id_sorter() -> CustomSorter {
            let f = |a: Arc<Airport>, b: Arc<Airport>| {
                Ordering::from(a.get_id().partial_cmp(b.get_id()).unwrap())
            };
            Self::get_common_sorter(f)
        }

        fn get_name_sorter() -> CustomSorter {
            let f = |a: Arc<Airport>, b: Arc<Airport>| {
                Ordering::from(a.get_name().partial_cmp(b.get_name()).unwrap())
            };
            Self::get_common_sorter(f)
        }

        fn get_lat_sorter() -> CustomSorter {
            let f = |a: Arc<Airport>, b: Arc<Airport>| {
                Ordering::from(a.get_lat().partial_cmp(b.get_lat()).unwrap())
            };
            Self::get_common_sorter(f)
        }

        fn get_long_sorter() -> CustomSorter {
            let f = |a: Arc<Airport>, b: Arc<Airport>| {
                Ordering::from(a.get_long().partial_cmp(b.get_long()).unwrap())
            };
            Self::get_common_sorter(f)
        }

        fn get_common_sorter(f: fn(Arc<Airport>, Arc<Airport>) -> Ordering) -> CustomSorter {
            CustomSorter::new(move |a, b| {
                let airport_a = a.clone().downcast::<AirportObject>()
                    .expect("The item has to be an `Airport`.");
                let airport_b = b.clone().downcast::<AirportObject>()
                    .expect("The item has to be an `Airport`.");

                f(airport_a.imp().airport(), airport_b.imp().airport())
            })
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

            self.col_id.set_factory(Some(&build_column_factory(|label: Label, airport: &AirportObject| {
                label.set_label(airport.imp().airport().get_id());
                label.set_xalign(0.0);
                airport.imp().set_ui(Some(label.clone()));
            })));

            self.col_name.set_factory(Some(&build_column_factory(|label: Label, airport: &AirportObject| {
                label.set_label(airport.imp().airport().get_name());
                label.set_xalign(0.0);
            })));

            self.col_lat.set_factory(Some(&build_column_factory(|label: Label, airport: &AirportObject| {
                label.set_label(&airport.imp().airport().get_lat_as_string());
                label.set_xalign(0.0);
            })));

            self.col_lon.set_factory(Some(&build_column_factory(|label: Label, airport: &AirportObject| {
                label.set_label(&airport.imp().airport().get_long_as_string());
                label.set_xalign(0.0);
            })));

            self.col_elev.set_factory(Some(&build_column_factory(|label: Label, airport: &AirportObject| {
                label.set_label(&airport.imp().airport().get_elevation().to_string());
                label.set_xalign(1.0);
            })));

            self.airport_list.connect_activate(
                clone!(#[weak(rename_to = view)] self, move | _list_view, position | {
                    if let Some(airport) = view.get_model_airport(position) {
                        view.add_to_plan(airport.imp().airport().clone());
                    }
                }),
            );

            // build popover menu
            let builder = Builder::from_resource("/com/shartrec/kelpie_planner/airport_popover.ui");
            let menu = builder.object::<MenuModel>("airports-menu");
            match menu {
                Some(popover) => {
                    let popover = PopoverMenu::builder()
                        .menu_model(&popover)
                        .has_arrow(false)
                        .build();
                    popover.set_parent(&self.airport_list.get());
                    let _ = self.popover.replace(Some(popover));
                }
                None => error!(" Not a popover"),
            }

            // Enable context menu key
            let ev_key = gtk::EventControllerKey::new();
            ev_key.connect_key_pressed(clone!(#[weak(rename_to = view)] self, #[upgrade_or] Propagation::Proceed,
                    move | _event, key_val, _key_code, modifier | {
                if key_val == Key::Menu && modifier == ModifierType::empty() {
                    if let Some(airport) = view.get_selection() {
                        if let Some(label) = airport.imp().ui().as_ref() {
                            let rect = label.compute_bounds(&view.airport_list.get()).unwrap();
                            let rect = Rectangle::new(rect.x() as i32, rect.y() as i32, 1, 1);
                            if let Some(popover) = view.popover.borrow().as_ref() {
                                popover.set_pointing_to(Some(&rect));
                                popover.popup();
                            };
                        }
                    }
                    Propagation::Stop
                } else {
                    Propagation::Proceed
                }

            }));
            self.airport_list.add_controller(ev_key);

            let gesture = gtk::GestureClick::new();
            gesture.set_button(3);
            gesture.connect_released(clone!(#[weak(rename_to = view)] self, move | gesture, _n, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                if let Some(popover) = view.popover.borrow().as_ref() {
                        popover.set_pointing_to(Some(&Rectangle::new(x as i32, y as i32, 1, 1)));
                        popover.popup();
                };
            }));
            self.airport_list.add_controller(gesture);

            // If the user clicks search or pressses enter in any of the search fields do the search
            self.airport_search
                .connect_clicked(clone!(#[weak(rename_to = window)] self, move |_search| {
                    window.search();
                }));
            self.airport_search_name.connect_activate(
                clone!(#[weak(rename_to = window)] self, move |_search| {
                    window.search();
                }),
            );
            self.airport_search_lat.connect_activate(
                clone!(#[weak(rename_to = window)] self, move |_search| {
                    window.search();
                }),
            );
            self.airport_search_long.connect_activate(
                clone!(#[weak(rename_to = window)] self, move |_search| {
                    window.search();
                }),
            );

            let actions = SimpleActionGroup::new();
            self.airport_window
                .get()
                .insert_action_group("airport", Some(&actions));

            let action = SimpleAction::new("add_to_plan", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                if let Some(airport) = view.get_selected_airport() {
                    view.add_to_plan(airport);
                }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_airports_near", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                    if let Some(airport) = view.get_selected_airport() {
                        view.search_near(airport.get_loc());
                    }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_navaids_near", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                    if let Some(airport) = view.get_selected_airport() {
                        if let Some(navaid_view) = get_navaid_view(&view.airport_window.get()) {
                            show_navaid_view(&view.airport_window.get());
                            navaid_view.imp().search_near(airport.get_loc());
                        }
                    }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_fixes_near", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                if let Some(airport) = view.get_selected_airport() {
                        if let Some(fix_view) = get_fix_view(&view.airport_window.get()) {
                            show_fix_view(&view.airport_window.get());
                            fix_view.imp().search_near(airport.get_loc());
                        }
                    }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("view", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                if let Some(airport) = view.get_selected_airport() {
                    if let Some(airport_map_view) = get_airport_map_view(&view.airport_window.get()) {
                        show_airport_map_view(&view.airport_window.get());
                        airport_map_view.imp().set_airport(airport.clone());
                    }
                }
            }));
            actions.add_action(&action);
        }

        fn dispose(&self) {
            if let Some(popover) = self.popover.borrow().as_ref() {
                popover.unparent();
            };
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

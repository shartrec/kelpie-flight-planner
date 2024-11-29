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
    use std::cell::{Cell, RefCell};
    use std::ops::Deref;
    use std::sync::Arc;

    use glib::subclass::InitializingObject;
    use gtk::{Builder, Button, ColumnView, ColumnViewColumn, CustomFilter, CustomSorter, Entry, FilterChange, FilterListModel, Label, Ordering, PopoverMenu, ScrolledWindow, SingleSelection, SortListModel};
    use gtk::gdk::Rectangle;
    use gtk::gio::{MenuModel, SimpleAction, SimpleActionGroup};
    use gtk::glib::{clone, MainContext};
    use log::error;

    use crate::earth::coordinate::Coordinate;
    use crate::earth::navaid_list_model::Navaids;
    use crate::event;
    use crate::event::Event;
    use crate::model::location::Location;
    use crate::model::navaid::Navaid;
    use crate::model::navaid_object::NavaidObject;
    use crate::model::waypoint::Waypoint;
    use crate::util::lat_long_format::LatLongFormat;
    use crate::util::location_filter::{CombinedFilter, NameIdFilter, new_navaid_filter, NilFilter, RangeFilter, set_navaid_filter};
    use crate::window::util::{build_column_factory, get_airport_view, get_fix_view, get_plan_view, show_airport_view, show_error_dialog, show_fix_view};

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/navaid_view.ui")]
    pub struct NavaidView {
        #[template_child]
        pub navaid_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub navaid_list: TemplateChild<ColumnView>,
        #[template_child]
        pub col_id: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_name: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_lat: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_lon: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_freq: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub navaid_search_name: TemplateChild<Entry>,
        #[template_child]
        pub navaid_search_lat: TemplateChild<Entry>,
        #[template_child]
        pub navaid_search_long: TemplateChild<Entry>,
        #[template_child]
        pub navaid_search: TemplateChild<Button>,

        popover: RefCell<Option<PopoverMenu>>,
        my_listener_id: RefCell<usize>,
        filter_list_model: RefCell<Option<FilterListModel>>,

    }

    impl NavaidView {
        pub fn initialise(&self) {
            let (tx, rx) = async_channel::unbounded::<Event>();
            let index = event::manager().register_listener(tx);

            MainContext::default().spawn_local(clone!(#[weak(rename_to = view)] self, async move {
                while let Ok(ev) = rx.recv().await {
                    if let Event::NavaidsLoaded = ev {
                        view.navaid_search.set_sensitive(true);

                        let fm = FilterListModel::new(Some(Navaids::new()), Some(new_navaid_filter(Box::new(NilFilter::new()))));

                        view.filter_list_model.replace(Some(fm.clone()));

                         // Add a sorter
                        view.col_id.set_sorter(Some(&Self::get_id_sorter()));
                        view.col_name.set_sorter(Some(&Self::get_name_sorter()));
                        view.col_lat.set_sorter(Some(&Self::get_lat_sorter()));
                        view.col_lon.set_sorter(Some(&Self::get_long_sorter()));

                        let sorter = view.navaid_list.sorter();

                        let slm = SortListModel::new(Some(fm), sorter);
                        slm.set_incremental(true);

                        let selection_model = SingleSelection::new(Some(slm));
                        selection_model.set_autoselect(false);
                        view.navaid_list.set_model(Some(&selection_model));
                        view.navaid_list.set_single_click_activate(true);
                    }
                }
            }));
            self.my_listener_id.replace(index);
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
                            show_error_dialog(&self.obj().root(), "Invalid Latitude for search.");
                            return;
                        }
                    };
                    if let Some(filter) = RangeFilter::new(lat_as_float, long_as_float, 100.0) {
                        combined_filter.add(Box::new(filter));
                    }
                }
            }

            let custom_filter = self.filter_list_model.borrow().as_ref().unwrap().filter().unwrap().downcast::<CustomFilter>().unwrap();

            self.navaid_list.model().unwrap().unselect_all();
            set_navaid_filter(&custom_filter, Box::new(combined_filter));
            custom_filter.changed(FilterChange::Different);
        }

        fn add_to_plan(&self, navaid: Arc<Navaid>) {
            if let Some(ref mut plan_view) = get_plan_view(&self.navaid_window.get()) {
                // get the plan
                plan_view.imp().add_waypoint_to_plan(Waypoint::Navaid {
                    navaid: navaid.clone(),
                    elevation: Cell::new(0),
                    locked: true,
                });
            }
        }

        fn get_selection(&self) -> Option<Arc<Navaid>> {
            let selection = self.navaid_list.model().unwrap().selection();
            let sel_ap = selection.nth(0);
            self.get_model_navaid(sel_ap)
        }

        fn get_model_navaid(&self, sel_ap: u32) -> Option<Arc<Navaid>> {
            let selection = self.navaid_list.model().unwrap().item(sel_ap);
            if let Some(object) = selection {
                let navaid_object = object.downcast::<NavaidObject>()
                    .expect("The item has to be an `Navaid`.");

                Some(navaid_object.imp().navaid())
            } else {
                None
            }
        }

        pub fn search_near(&self, coordinate: &Coordinate) {
            self.navaid_search_name
                .set_text("");
            self.navaid_search_lat
                .set_text(&coordinate.get_latitude_as_string());
            self.navaid_search_long
                .set_text(&coordinate.get_longitude_as_string());
            self.navaid_search.emit_clicked();
        }

        fn get_id_sorter() -> CustomSorter {
            let f = |a: Arc<Navaid>, b: Arc<Navaid>| {
                Ordering::from(a.get_id().partial_cmp(b.get_id()).unwrap())
            };
            Self::get_common_sorter(f)
        }

        fn get_name_sorter() -> CustomSorter {
            let f = |a: Arc<Navaid>, b: Arc<Navaid>| {
                Ordering::from(a.get_name().partial_cmp(b.get_name()).unwrap())
            };
            Self::get_common_sorter(f)
        }

        fn get_lat_sorter() -> CustomSorter {
            let f = |a: Arc<Navaid>, b: Arc<Navaid>| {
                Ordering::from(a.get_lat().partial_cmp(b.get_lat()).unwrap())
            };
            Self::get_common_sorter(f)
        }

        fn get_long_sorter() -> CustomSorter {
            let f = |a: Arc<Navaid>, b: Arc<Navaid>| {
                Ordering::from(a.get_long().partial_cmp(b.get_long()).unwrap())
            };
            Self::get_common_sorter(f)
        }

        fn get_common_sorter(f: fn(Arc<Navaid>, Arc<Navaid>) -> Ordering) -> CustomSorter {
            CustomSorter::new(move |a, b| {
                let navaid_a = a.clone().downcast::<NavaidObject>()
                    .expect("The item has to be an `Navaid`.");
                let navaid_b = b.clone().downcast::<NavaidObject>()
                    .expect("The item has to be an `Navaid`.");

                f(navaid_a.imp().navaid(), navaid_b.imp().navaid())
            })
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

            self.col_id.set_factory(Some(&build_column_factory(|label: Label, navaid: &NavaidObject| {
                label.set_label(navaid.imp().navaid().get_id());
                label.set_xalign(0.0);
            })));

            self.col_name.set_factory(Some(&build_column_factory(|label: Label, navaid: &NavaidObject| {
                label.set_label(navaid.imp().navaid().get_name());
                label.set_xalign(0.0);
            })));

            self.col_lat.set_factory(Some(&build_column_factory(|label: Label, navaid: &NavaidObject| {
                label.set_label(&navaid.imp().navaid().get_lat_as_string());
                label.set_xalign(0.0);
            })));

            self.col_lon.set_factory(Some(&build_column_factory(|label: Label, navaid: &NavaidObject| {
                label.set_label(&navaid.imp().navaid().get_long_as_string());
                label.set_xalign(0.0);
            })));

            self.col_freq.set_factory(Some(&build_column_factory(|label: Label, navaid: &NavaidObject| {
                label.set_label(&navaid.imp().navaid().get_freq().to_string());
                label.set_xalign(1.0);
            })));

            // ToDo Disabled for now.
            // self.navaid_list.connect_activate(
            //     clone!(#[weak(rename_to = view)] self, move | _list_view, position | {
            //         view.add_item_to_plan(position);
            //     }),
            // );

            // build popover menu
            let builder = Builder::from_resource("/com/shartrec/kelpie_planner/navaid_popover.ui");
            let menu = builder.object::<MenuModel>("navaids-menu");
            match menu {
                Some(popover) => {
                    let popover = PopoverMenu::builder()
                        .menu_model(&popover)
                        .has_arrow(false)
                        .build();
                    popover.set_parent(&self.navaid_list.get());
                    let _ = self.popover.replace(Some(popover));
                }
                None => error!(" Not a popover"),
            }

            let gesture = gtk::GestureClick::new();
            gesture.set_button(3);
            gesture.connect_released(clone!(#[weak(rename_to = view)] self, move |gesture, _n, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                if let Some(popover) = view.popover.borrow().as_ref() {
                        popover.set_pointing_to(Some(&Rectangle::new(x as i32, y as i32, 1, 1)));
                        popover.popup();
                };
            }));
            self.navaid_window.add_controller(gesture);

            self.navaid_search
                .connect_clicked(clone!(#[weak(rename_to = window)] self, move |_search| {
                    window.search();
                }));
            self.navaid_search_name.connect_activate(
                clone!(#[weak(rename_to = window)] self, move |_search| {
                    window.search();
                })
            );
            self.navaid_search_lat.connect_activate(
                clone!(#[weak(rename_to = window)] self, move |_search| {
                    window.search();
                })
            );
            self.navaid_search_long.connect_activate(
                clone!(#[weak(rename_to = window)] self, move |_search| {
                    window.search();
                })
            );

            let actions = SimpleActionGroup::new();
            self.navaid_window
                .get()
                .insert_action_group("navaid", Some(&actions));
            let action = SimpleAction::new("add_to_plan", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
               view.navaid_list.activate();
            }));

            let action = SimpleAction::new("find_airports_near", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                    if let Some(navaid) = view.get_selection() {
                        if let Some(airport_view) = get_airport_view(&view.navaid_window.get()) {
                            show_airport_view(&view.navaid_window.get());
                            airport_view.imp().search_near(navaid.get_loc());
                        }
                    }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_navaids_near", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                if let Some(navaid) = view.get_selection() {
                    view.search_near(navaid.get_loc());
                }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_fixes_near", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                if let Some(navaid) = view.get_selection() {
                        if let Some(fix_view) = get_fix_view(&view.navaid_window.get()) {
                            show_fix_view(&view.navaid_window.get());
                            fix_view.imp().search_near(navaid.get_loc());
                        }
               }
            }));

            actions.add_action(&action);
            let action = SimpleAction::new("add_to_plan", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                if let Some(navaid) = view.get_selection() {
                    view.add_to_plan(navaid);
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

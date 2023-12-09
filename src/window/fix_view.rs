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
    use std::cell::{Cell, RefCell};
    use std::ops::Deref;
    use std::sync::Arc;

    use glib::subclass::InitializingObject;
    use gtk::{Builder, Button, ColumnView, ColumnViewColumn, CustomFilter, CustomSorter, Entry, FilterChange, FilterListModel, Label, Ordering, PopoverMenu, ScrolledWindow, SingleSelection, SortListModel};
    use gtk::gdk::Rectangle;
    use gtk::gio::{MenuModel, SimpleAction, SimpleActionGroup};
    use gtk::glib::{clone, MainContext, PRIORITY_DEFAULT};
    use log::error;

    use crate::earth::coordinate::Coordinate;
    use crate::earth::fix_list_model::Fixes;
    use crate::event;
    use crate::event::Event;
    use crate::model::fix::Fix;
    use crate::model::fix_object::FixObject;
    use crate::model::location::Location;
    use crate::model::waypoint::Waypoint;
    use crate::util::lat_long_format::LatLongFormat;
    use crate::util::location_filter::{CombinedFilter, IdFilter, new_fix_filter, NilFilter, RangeFilter, set_fix_filter};
    use crate::window::util::{build_column_factory, get_airport_view, get_plan_view, show_airport_view, show_error_dialog, show_navaid_view};

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/fix_view.ui")]
    pub struct FixView {
        #[template_child]
        pub fix_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub fix_list: TemplateChild<ColumnView>,
        #[template_child]
        pub col_id: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_lat: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_lon: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub fix_search_name: TemplateChild<Entry>,
        #[template_child]
        pub fix_search_lat: TemplateChild<Entry>,
        #[template_child]
        pub fix_search_long: TemplateChild<Entry>,
        #[template_child]
        pub fix_search: TemplateChild<Button>,

        popover: RefCell<Option<PopoverMenu>>,
        my_listener_id: RefCell<usize>,
        filter_list_model: RefCell<Option<FilterListModel>>,

    }

    impl FixView {
        pub fn initialise(&self) -> () {
            let (tx, rx) = MainContext::channel(PRIORITY_DEFAULT);
            let index = event::manager().register_listener(tx);
            rx.attach(None, clone!(@weak self as view => @default-return glib::source::Continue(true), move |ev: Event| {
                match ev {
                    Event::FixesLoaded => {
                        view.fix_search.set_sensitive(true);
                        let fm = FilterListModel::new(Some(Fixes::new()), Some(new_fix_filter(Box::new(NilFilter::new()))));

                        view.filter_list_model.replace(Some(fm.clone()));

                         // Add a sorter
                        view.col_id.set_sorter(Some(&Self::get_id_sorter()));
                        view.col_lat.set_sorter(Some(&Self::get_lat_sorter()));
                        view.col_lon.set_sorter(Some(&Self::get_long_sorter()));

                        let sorter = view.fix_list.sorter();

                        let slm = SortListModel::new(Some(fm), sorter);
                        slm.set_incremental(true);

                        let selection_model = SingleSelection::new(Some(slm));
                        selection_model.set_autoselect(false);
                        view.fix_list.set_model(Some(&selection_model));
                    },
                    _ => (),
                }
                glib::source::Continue(true)
            }));
            // self.my_listener.replace(Some(rx));
            self.my_listener_id.replace(index);
        }

        pub fn search(&self) {
            let term = self.fix_search_name.text();
            let lat = self.fix_search_lat.text();
            let long = self.fix_search_long.text();

            let mut combined_filter = CombinedFilter::new();
            if !term.is_empty() {
                if let Some(filter) = IdFilter::new(term.as_str()) {
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

            self.fix_list.model().unwrap().unselect_all();
            set_fix_filter(&custom_filter, Box::new(combined_filter));
            custom_filter.changed(FilterChange::Different);
        }

        fn add_selected_to_plan(&self) {
            if let Some(navaid) = self.get_selection() {
                self.add_to_plan(navaid);
            }
        }

        fn add_item_to_plan(&self, item: u32) {
            if let Some(navaid) = self.get_model_navaid(item) {
                self.add_to_plan(navaid);
            }
        }

        fn add_to_plan(&self, fix: Arc<Fix>) {
            match get_plan_view(&self.fix_window.get()) {
                Some(ref mut plan_view) => {
                    // get the plan
                    plan_view.imp().add_waypoint_to_plan(Waypoint::Fix {
                        fix: fix.clone(),
                        elevation: Cell::new(0),
                        locked: true,
                    });
                }
                None => (),
            }
        }

        fn get_selection(&self) -> Option<Arc<Fix>> {
            let selection = self.fix_list.model().unwrap().selection();
            let sel_ap = selection.nth(0);
            self.get_model_navaid(sel_ap)
        }

        fn get_model_navaid(&self, sel_ap: u32) -> Option<Arc<Fix>> {
            let selection = self.fix_list.model().unwrap().item(sel_ap);
            if let Some(object) = selection {
                let fix_object = object.downcast::<FixObject>()
                    .expect("The item has to be an `Fix`.");
                Some(fix_object.imp().fix())
            } else {
                None
            }
        }

        pub fn search_near(&self, coordinate: &Coordinate) {
            self.fix_search_name
                .set_text("");
            self.fix_search_lat
                .set_text(&coordinate.get_latitude_as_string());
            self.fix_search_long
                .set_text(&coordinate.get_longitude_as_string());
            self.fix_search.emit_clicked();
        }

        fn get_id_sorter() -> CustomSorter {
            let f = |a: Arc<Fix>, b: Arc<Fix> | {
                Ordering::from(a.get_id().partial_cmp(b.get_id()).unwrap())
            };
            Self::get_common_sorter(f)
        }

        fn get_lat_sorter() -> CustomSorter {
            let f = |a: Arc<Fix>, b: Arc<Fix> | {
                Ordering::from(a.get_lat().partial_cmp(b.get_lat()).unwrap())
            };
            Self::get_common_sorter(f)
        }

        fn get_long_sorter() -> CustomSorter {
            let f = |a: Arc<Fix>, b: Arc<Fix> | {
                Ordering::from(a.get_long().partial_cmp(b.get_long()).unwrap())
            };
            Self::get_common_sorter(f)
        }

        fn get_common_sorter(f: fn(Arc<Fix>, Arc<Fix>) -> Ordering) -> CustomSorter {
            CustomSorter::new( move |a, b| {
                let fix_a = a.clone().downcast::<FixObject>()
                    .expect("The item has to be an `Fix`.");
                let fix_b = b.clone().downcast::<FixObject>()
                    .expect("The item has to be an `Fix`.");

                f(fix_a.imp().fix(), fix_b.imp().fix())
            })
        }

    }

    #[glib::object_subclass]
    impl ObjectSubclass for FixView {
        const NAME: &'static str = "FixView";
        type Type = super::FixView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FixView {
        fn constructed(&self) {
            self.parent_constructed();
            self.initialise();

            let f = |label: Label, fix: &FixObject| {
                label.set_label(&fix.imp().fix().get_id());
                label.set_xalign(0.0);
            };
            let factory = build_column_factory(f);
            self.col_id.set_factory(Some(&factory));


            let f = |label: Label, fix: &FixObject| {
                label.set_label(&fix.imp().fix().get_lat_as_string());
                label.set_xalign(0.0);
            };
            let factory = build_column_factory(f);
            self.col_lat.set_factory(Some(&factory));

            let f = |label: Label, fix: &FixObject| {
                label.set_label(&fix.imp().fix().get_long_as_string());
                label.set_xalign(0.0);
            };
            let factory = build_column_factory(f);
            self.col_lon.set_factory(Some(&factory));

            self.fix_list.connect_activate(
                clone!(@weak self as view => move | _list_view, position | {
                    view.add_item_to_plan(position);
                }),
            );

            // build popover menu

            let builder = Builder::from_resource("/com/shartrec/kelpie_planner/fix_popover.ui");
            let menu = builder.object::<MenuModel>("fixes-menu");
            match menu {
                Some(popover) => {
                    let popover = PopoverMenu::builder()
                        .menu_model(&popover)
                        .build();
                    popover.set_parent(&self.fix_window.get());
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
            self.fix_window.add_controller(gesture);

            self.fix_search
                .connect_clicked(clone!(@weak self as window => move |_search| {
                    window.search();
                }));
            self.fix_search_name
                .connect_activate(clone!(@weak self as window => move |_search| {
                    window.search();
                }));
            self.fix_search_lat
                .connect_activate(clone!(@weak self as window => move |_search| {
                    window.search();
                }));
            self.fix_search_long
                .connect_activate(clone!(@weak self as window => move |_search| {
                    window.search();
                }));

            let actions = SimpleActionGroup::new();
            self.fix_window
                .get()
                .insert_action_group("fix", Some(&actions));
            let action = SimpleAction::new("add_to_plan", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
               view.fix_list.activate();
            }));

            let action = SimpleAction::new("find_airports_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                    if let Some(fix) = view.get_selection() {
                        match get_airport_view(&view.fix_window.get()) {
                            Some(airport_view) => {
                                show_airport_view(&view.fix_window.get());
                                airport_view.imp().search_near(&fix.get_loc());
                                ()
                            },
                            None => (),
                        }
                    }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_navaids_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
               if let Some(fix) = view.get_selection() {
                    match get_airport_view(&view.fix_window.get()) {
                        Some(navaid_view) => {
                            show_navaid_view(&view.fix_window.get());
                            navaid_view.imp().search_near(&fix.get_loc());
                            ()
                        },
                        None => (),
                    }
                }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_fixes_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                if let Some(fix) = view.get_selection() {
                        view.search_near(&fix.get_loc());
               }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("add_to_plan", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                view.add_selected_to_plan();
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

    impl WidgetImpl for FixView {}

    impl BoxImpl for FixView {}
}

glib::wrapper! {
    pub struct FixView(ObjectSubclass<imp::FixView>) @extends gtk::Widget, gtk::Box;
}

impl FixView {
    pub fn new() -> Self {
        glib::Object::new::<FixView>()
    }
}

impl Default for FixView {
    fn default() -> Self {
        Self::new()
    }
}

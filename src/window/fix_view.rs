/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use gtk::{self, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use std::cell::Cell;

    use glib::subclass::InitializingObject;
    use gtk::gdk::Rectangle;
    use gtk::gio::{MenuModel, SimpleAction, SimpleActionGroup};
    use gtk::glib::clone;
    use gtk::{Builder, Button, Entry, ListStore, PopoverMenu, ScrolledWindow, TreeView};
    use log::error;

    use crate::earth;
    use crate::earth::coordinate::Coordinate;
    use crate::model::location::Location;
    use crate::model::waypoint::Waypoint;
    use crate::util::lat_long_format::LatLongFormat;
    use crate::util::location_filter::{CombinedFilter, Filter, IdFilter, RangeFilter};
    use crate::window::util::{
        get_airport_view, get_navaid_view, get_plan_view, show_airport_view, show_error_dialog,
        show_navaid_view,
    };

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/fix_view.ui")]
    pub struct FixView {
        #[template_child]
        pub fix_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub fix_list: TemplateChild<TreeView>,
        #[template_child]
        pub fix_search_name: TemplateChild<Entry>,
        #[template_child]
        pub fix_search_lat: TemplateChild<Entry>,
        #[template_child]
        pub fix_search_long: TemplateChild<Entry>,
        #[template_child]
        pub fix_search: TemplateChild<Button>,
    }

    impl FixView {
        pub fn initialise(&self) -> () {}

        pub fn fixes_loaded(&self) {
            self.fix_search.set_sensitive(true);
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
            let fixes = earth::get_earth_model().get_fixes().read().unwrap();
            let searh_result = fixes.iter().filter(move |a| {
                let fix: &dyn Location = &***a;
                combined_filter.filter(fix)
            });
            let store = ListStore::new(&[
                String::static_type(),
                String::static_type(),
                String::static_type(),
                String::static_type(),
            ]);
            for fix in searh_result {
                store.insert_with_values(
                    None,
                    &[
                        (0, &fix.get_id()),
                        (1, &fix.get_lat_as_string()),
                        (2, &fix.get_long_as_string()),
                    ],
                );
            }
            self.fix_list.set_model(Some(&store));
        }

        fn add_selected_to_plan(&self) {
            if let Some((model, iter)) = self.fix_list.selection().selected() {
                let id = TreeModelExtManual::get::<String>(&model, &iter, 0);
                if let Some(fix) = earth::get_earth_model().get_fix_by_id(id.as_str()) {
                    match get_plan_view(&self.fix_window.get()) {
                        Some(view) => {
                            // get the plan
                            view.imp().add_waypoint_to_plan(Waypoint::Fix {
                                fix: fix.clone(),
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
            self.fix_search_lat
                .set_text(&coordinate.get_latitude_as_string());
            self.fix_search_long
                .set_text(&coordinate.get_longitude_as_string());
            self.fix_search.activate();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FixView {
        const NAME: &'static str = "FixView";
        type Type = super::FixView;
        type ParentType = gtk::Widget;

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

            self.fix_list.connect_row_activated(
                clone!(@weak self as view => move |_list_view, _position, _col| {
                     view.add_selected_to_plan();
                }),
            );

            let gesture = gtk::GestureClick::new();
            gesture.set_button(3);
            gesture.connect_released(clone!(@weak self as view => move |gesture, _n, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);

                let builder = Builder::from_resource("/com/shartrec/kelpie_planner/fix_popover.ui");
                let menu = builder.object::<MenuModel>("fixes-menu");
                match menu {
                    Some(popover) => {
                        let popover = PopoverMenu::builder()
                            .menu_model(&popover)
                            .pointing_to(&Rectangle::new(x as i32, y as i32, 1, 1))
                            .build();
                        popover.set_parent(&view.fix_window.get());
                        popover.popup();
                    }
                    None => error!(" Not a popover"),
                }
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
                if let Some((model, iter)) = view.fix_list.selection().selected() {
                    let id = TreeModelExtManual::get::<String>(&model, &iter, 0);
                    if let Some(fix) = earth::get_earth_model().get_fix_by_id(id.as_str()) {
                        match get_airport_view(&view.fix_window.get()) {
                            Some(airport_view) => {
                                show_airport_view(&view.fix_window.get());
                                airport_view.imp().search_near(&fix.get_loc());
                                ()
                            },
                            None => (),
                        }
                    }
               }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_navaids_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                if let Some((model, iter)) = view.fix_list.selection().selected() {
                    let id = TreeModelExtManual::get::<String>(&model, &iter, 0);
                    if let Some(fix) = earth::get_earth_model().get_fix_by_id(id.as_str()) {
                        match get_navaid_view(&view.fix_window.get()) {
                            Some(navaid_view) => {
                                show_navaid_view(&view.fix_window.get());
                                navaid_view.imp().search_near(&fix.get_loc());
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
                view.add_selected_to_plan();
            }));
            actions.add_action(&action);
        }
    }

    impl WidgetImpl for FixView {}
}

glib::wrapper! {
    pub struct FixView(ObjectSubclass<imp::FixView>) @extends gtk::Widget;
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

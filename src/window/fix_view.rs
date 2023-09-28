/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use std::ops::Deref;

use gtk::{Button, Entry, ListStore, TreeView};
use gtk::{self, CompositeTemplate, glib, prelude::*, subclass::prelude::*};

mod imp {
    use glib::subclass::InitializingObject;
    use gtk::{Button, Entry, ScrolledWindow};
    use gtk::glib::clone;

    use crate::earth;
    use crate::model::location::Location;
    use crate::util::lat_long_format::LatLongFormat;
    use crate::util::location_filter::{Filter, CombinedFilter, IdFilter, RangeFilter};
    use crate::window::util::show_error_dialog;

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
            let fixes = earth::get_earth_model().get_fixes().read().unwrap();
            let searh_result = fixes.iter().filter(move |a| {
                let fix: &dyn Location = &***a;
                combined_filter.filter(fix)
            });
            let store = ListStore::new(&[String::static_type(), String::static_type(), String::static_type(), String::static_type()]);
            for fix in searh_result {
                store.insert_with_values(
                    None, &[
                        (0, &fix.get_id()),
                        (1, &fix.get_lat_as_string()),
                        (2, &fix.get_long_as_string())
                    ]);
            }
            self.fix_list.set_model(Some(&store));
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

            self.fix_search.connect_clicked(
                clone!(@weak self as window => move |search| {
                window.search();
            }));
            self.fix_search_name.connect_activate(
                clone!(@weak self as window => move |search| {
                window.search();
            }));
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

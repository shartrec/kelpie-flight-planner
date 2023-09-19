/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use std::ops::Deref;

use gtk::{Button, Entry, ListStore, TreeView};
use gtk::{self, CompositeTemplate, glib, prelude::*, subclass::prelude::*};
use regex::RegexBuilder;

mod imp {
    use glib::subclass::InitializingObject;
    use gtk::{Button, Entry};
    use gtk::glib::clone;

    use crate::earth;
    use crate::model::location::Location;

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/navaid_view.ui")]
    pub struct NavaidView {
        #[template_child]
        pub navaid_list: TemplateChild<TreeView>,
        #[template_child]
        pub navaid_search_name: TemplateChild<Entry>,
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
            let sterm = term.as_str();
            let regex = RegexBuilder::new(sterm)
                .case_insensitive(true)
                .build();

            match regex {
                Ok(r) => {
                    let navaids = earth::get_earth_model().get_navaids().read().unwrap();
                    let searh_result = navaids.iter().filter(move |a| {
                        a.get_id().eq_ignore_ascii_case(sterm) || r.is_match(a.get_name().as_str())
                    });
                    let store = ListStore::new(&[String::static_type(), String::static_type(), String::static_type(), String::static_type(), i32::static_type()]);
                    for navaid in searh_result {
                        store.insert_with_values(
                            None, &[
                                (0, &navaid.get_id()),
                                (1, &navaid.get_name()),
                                (2, &navaid.get_lat_as_string()),
                                (3, &navaid.get_long_as_string()),
                                (4, &(navaid.get_elevation()))
                            ]);
                    }
                    self.navaid_list.set_model(Some(&store));
                }
                Err(_) => (),
            }
        }
    }


    #[glib::object_subclass]
    impl ObjectSubclass for NavaidView {
        const NAME: &'static str = "NavaidView";
        type Type = super::NavaidView;
        type ParentType = gtk::Widget;

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

            self.navaid_search.connect_clicked(
                clone!(@weak self as window => move |search| {
                window.search();
            }));
            self.navaid_search_name.connect_activate(
                clone!(@weak self as window => move |search| {
                window.search();
            }));
        }
    }

    impl WidgetImpl for NavaidView {}
}

glib::wrapper! {
    pub struct NavaidView(ObjectSubclass<imp::NavaidView>) @extends gtk::Widget;
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

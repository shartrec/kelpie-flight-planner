/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use std::ops::Deref;

use gtk::{Button, Entry, ListStore, TreeView};
use gtk::{self, CompositeTemplate, glib, prelude::*, subclass::prelude::*};

mod imp {
    use glib::subclass::InitializingObject;
    use gtk::{Button, Entry};
    use gtk::glib::clone;

    use crate::earth;
    use crate::model::location::Location;

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/fix_view.ui")]
    pub struct FixView {
        #[template_child]
        pub fix_list: TemplateChild<TreeView>,
        #[template_child]
        pub fix_search_name: TemplateChild<Entry>,
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
            let sterm = term.as_str();

            let fixes = earth::get_earth_model().get_fixes().read().unwrap();
            let searh_result = fixes.iter().filter(move |a| {
                a.get_id().eq_ignore_ascii_case(sterm)
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

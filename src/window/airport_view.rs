/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use gtk::{Button, Entry, ListStore, TreeView};
use gtk::{self, CompositeTemplate, glib, prelude::*, subclass::prelude::*};

mod imp {
    use glib::subclass::InitializingObject;
    use gtk::{Button, Entry};
    use gtk::glib::clone;
    use regex::RegexBuilder;

    use crate::earth;
    use crate::model::location::Location;

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/airport_view.ui")]
    pub struct AirportView {
        #[template_child]
        pub airport_list: TemplateChild<TreeView>,
        #[template_child]
        pub airport_search_name: TemplateChild<Entry>,
        #[template_child]
        pub airport_search: TemplateChild<Button>,
    }

    impl AirportView {
        pub fn initialise(&self) -> () {}

        pub fn airports_loaded(&self) {
            self.airport_search.set_sensitive(true);
        }

        pub fn search(&self) {
            let term = self.airport_search_name.text();
            let sterm = term.as_str();
            let regex = RegexBuilder::new(sterm)
                .case_insensitive(true)
                .build();

            match regex {
                Ok(r) => {
                    let airports = earth::get_earth_model().get_airports().read().unwrap();
                    let searh_result = airports.iter().filter(move |a| {
                        a.get_id().eq_ignore_ascii_case(sterm) || r.is_match(a.get_name().as_str())
                    });
                    let store = ListStore::new(&[String::static_type(), String::static_type(), f64::static_type(), f64::static_type(), f64::static_type()]);
                    for airport in searh_result {
                        store.insert_with_values(
                            None, &[
                                (0, &airport.get_id()),
                                (1, &airport.get_name()),
                                (2, &airport.get_lat()),
                                (3, &airport.get_long()),
                                (4, &airport.get_elevation(),
                                )]);
                    }
                    self.airport_list.set_model(Some(&store));
                }
                Err(_) => (),
            }
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

            self.airport_search.connect_clicked(
                clone!(@weak self as window => move |search| {
                window.search();
            }));
            self.airport_search_name.connect_activate(
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

/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use gtk::{ListStore, TreeView};
use gtk::{self, glib, prelude::*, subclass::prelude::*, CompositeTemplate};
use gtk::ffi::GtkWidget;
use gtk::glib::Object;

mod imp {
    use glib::subclass::InitializingObject;
    use gtk::ffi::{GtkScrolledWindow, GtkWidget};

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/airport_view.ui")]
    pub struct AirportView {
        #[template_child]
        pub airport_list: TemplateChild<TreeView>,
    }

    impl AirportView {

        pub fn initialise(&self) -> () {

            let store = ListStore::new(&[String::static_type(), String::static_type(), f64::static_type(), f64::static_type(), f64::static_type()]);
            self.airport_list.set_model(Some(&store));

            store.insert_with_values(None, &[(0, &"YSSY"), (1, &"Sydney"), (2, &-34.5), (3, &151.0), (4, &25.0)]);
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

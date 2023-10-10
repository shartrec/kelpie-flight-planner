/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use gtk::{self, CompositeTemplate, glib, subclass::prelude::*};

mod imp {
    use glib::subclass::InitializingObject;

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/world_map_view.ui")]
    pub struct WorldMapView {
        // #[template_child]
        // pub fix_window: TemplateChild<ScrolledWindow>,
    }

    impl WorldMapView {
        pub fn initialise(&self) -> () {}
    }


    #[glib::object_subclass]
    impl ObjectSubclass for WorldMapView {
        const NAME: &'static str = "WorldMapView";
        type Type = super::WorldMapView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WorldMapView {
        fn constructed(&self) {
            self.parent_constructed();
            self.initialise();
        }

    }

    impl WidgetImpl for WorldMapView {}
}

glib::wrapper! {
    pub struct WorldMapView(ObjectSubclass<imp::WorldMapView>) @extends gtk::Widget;
}

impl WorldMapView {
    pub fn new() -> Self {
        glib::Object::new::<WorldMapView>()
    }
}

impl Default for WorldMapView {
    fn default() -> Self {
        Self::new()
    }
}

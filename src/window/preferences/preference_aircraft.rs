use gtk::{self, glib};

mod imp {
    use gtk::{CompositeTemplate, glib};
    use gtk::glib::subclass::InitializingObject;
    use gtk::subclass::prelude::{BoxImpl, CompositeTemplate, ObjectImpl, ObjectSubclass, WidgetClassSubclassExt};
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_planner/preference_aircraft.ui")]
    pub struct PreferenceAircraftPage {
    }

    impl PreferenceAircraftPage {}

    #[glib::object_subclass]
    impl ObjectSubclass for PreferenceAircraftPage {
        const NAME: &'static str = "PreferenceAircraftPage";
        type Type = super::PreferenceAircraftPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }

    }


    impl ObjectImpl for PreferenceAircraftPage {}

    impl BoxImpl for PreferenceAircraftPage {
    }
    impl WidgetImpl for PreferenceAircraftPage {}

}

glib::wrapper! {
    pub struct PreferenceAircraftPage(ObjectSubclass<imp::PreferenceAircraftPage>)
        @extends gtk::Box, gtk::Widget;
}

impl PreferenceAircraftPage {
    pub fn new() -> Self {
        glib::Object::new::<PreferenceAircraftPage>()
    }
}

impl Default for PreferenceAircraftPage {
    fn default() -> Self {
        Self::new()
    }
}

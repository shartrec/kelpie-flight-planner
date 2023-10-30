use gtk::{self, glib};

mod imp {
    use gtk::{CompositeTemplate, glib};
    use gtk::glib::subclass::InitializingObject;
    use gtk::subclass::prelude::{BoxImpl, CompositeTemplate, ObjectImpl, ObjectSubclass, WidgetClassSubclassExt};
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_planner/preference_planner.ui")]
    pub struct PreferencePlannerPage {
    }

    impl PreferencePlannerPage {}

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencePlannerPage {
        const NAME: &'static str = "PreferencePlannerPage";
        type Type = super::PreferencePlannerPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }

    }


    impl ObjectImpl for PreferencePlannerPage {}

    impl BoxImpl for PreferencePlannerPage {
    }
    impl WidgetImpl for PreferencePlannerPage {}

}

glib::wrapper! {
    pub struct PreferencePlannerPage(ObjectSubclass<imp::PreferencePlannerPage>)
        @extends gtk::Box, gtk::Widget;
}

impl PreferencePlannerPage {
    pub fn new() -> Self {
        glib::Object::new::<PreferencePlannerPage>()
    }
}

impl Default for PreferencePlannerPage {
    fn default() -> Self {
        Self::new()
    }
}

use glib::Object;
use gtk::{self, gio, glib, prelude::*, subclass::prelude::*, Window};
use gtk::prelude::GtkWindowExt;

mod imp {
    use gtk::{CompositeTemplate, glib};
    use gtk::glib::subclass::InitializingObject;
    use gtk::subclass::prelude::{CompositeTemplate, ObjectImpl, ObjectSubclass, ObjectSubclassType, WidgetClassSubclassExt, WindowImpl};
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_planner/preference_general.ui")]
    pub struct PreferenceGeneralPage {
    }

    impl PreferenceGeneralPage {}

    #[glib::object_subclass]
    impl ObjectSubclass for PreferenceGeneralPage {
        const NAME: &'static str = "PreferenceGeneralPage";
        type Type = super::PreferenceGeneralPage;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }

    }


    impl ObjectImpl for PreferenceGeneralPage {}

    impl WidgetImpl for PreferenceGeneralPage {}

}

glib::wrapper! {
    pub struct PreferenceGeneralPage(ObjectSubclass<imp::PreferenceGeneralPage>)
        @extends gtk::Widget;
}

impl PreferenceGeneralPage {
    pub fn new() -> Self {
        glib::Object::new::<PreferenceGeneralPage>()
    }
}

impl Default for PreferenceGeneralPage {
    fn default() -> Self {
        Self::new()
    }
}

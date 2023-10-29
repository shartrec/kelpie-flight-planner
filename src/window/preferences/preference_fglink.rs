use glib::Object;
use gtk::{self, gio, glib, prelude::*, subclass::prelude::*, Window};
use gtk::prelude::GtkWindowExt;

mod imp {
    use gtk::{CompositeTemplate, glib};
    use gtk::glib::subclass::InitializingObject;
    use gtk::subclass::prelude::{CompositeTemplate, ObjectImpl, ObjectSubclass, ObjectSubclassType, WidgetClassSubclassExt, WindowImpl};
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_planner/preference_fglink.ui")]
    pub struct PreferenceFgLinkPage {
    }

    impl PreferenceFgLinkPage {}

    #[glib::object_subclass]
    impl ObjectSubclass for PreferenceFgLinkPage {
        const NAME: &'static str = "PreferenceFgLinkPage";
        type Type = super::PreferenceFgLinkPage;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }

    }


    impl ObjectImpl for PreferenceFgLinkPage {}

    impl WidgetImpl for PreferenceFgLinkPage {}

}

glib::wrapper! {
    pub struct PreferenceFgLinkPage(ObjectSubclass<imp::PreferenceFgLinkPage>)
        @extends gtk::Widget;
}

impl PreferenceFgLinkPage {
    pub fn new() -> Self {
        glib::Object::new::<PreferenceFgLinkPage>()
    }
}

impl Default for PreferenceFgLinkPage {
    fn default() -> Self {
        Self::new()
    }
}

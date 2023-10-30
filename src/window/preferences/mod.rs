use gtk::{self, gio, glib};

mod preference_general;
mod preference_fglink;
mod preference_planner;
mod preference_aircraft;

mod imp {
    use gtk::{CompositeTemplate, glib, TemplateChild};
    use gtk::glib::subclass::InitializingObject;
    use gtk::subclass::prelude::{CompositeTemplate, ObjectImpl, ObjectSubclass, WidgetClassSubclassExt, WindowImpl};
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};

    use crate::window::preferences::preference_aircraft::PreferenceAircraftPage;
    use crate::window::preferences::preference_fglink::PreferenceFgLinkPage;
    use crate::window::preferences::preference_general::PreferenceGeneralPage;
    use crate::window::preferences::preference_planner::PreferencePlannerPage;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_planner/preference.ui")]
    pub struct PreferenceDialog {
        #[template_child]
        pub general_page: TemplateChild<PreferenceGeneralPage>,
        #[template_child]
        pub planner_page: TemplateChild<PreferencePlannerPage>,
        #[template_child]
        pub aircraft_page: TemplateChild<PreferenceAircraftPage>,
        #[template_child]
        pub fglink_page: TemplateChild<PreferenceFgLinkPage>,
    }

    impl PreferenceDialog {}

    #[glib::object_subclass]
    impl ObjectSubclass for PreferenceDialog {
        const NAME: &'static str = "PreferenceDialog";
        type Type = super::PreferenceDialog;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }

    }


    impl ObjectImpl for PreferenceDialog {}

    impl WidgetImpl for PreferenceDialog {}
    impl WindowImpl for PreferenceDialog {}

}

glib::wrapper! {
    pub struct PreferenceDialog(ObjectSubclass<imp::PreferenceDialog>)
        @extends gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl PreferenceDialog {
    pub fn new() -> Self {
        glib::Object::new::<PreferenceDialog>()
    }
}

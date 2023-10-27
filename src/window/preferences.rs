use glib::Object;
use gtk::{self, gio, glib, prelude::*, subclass::prelude::*, Window};
use gtk::prelude::GtkWindowExt;

mod imp {
    use gtk::{CompositeTemplate, glib};
    use gtk::glib::subclass::InitializingObject;
    use gtk::subclass::prelude::{CompositeTemplate, ObjectImpl, ObjectSubclass, ObjectSubclassType, WidgetClassSubclassExt, WindowImpl};
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_planner/preference.ui")]
    pub struct PreferenceDialog {
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

/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */


use gtk::{gio, glib};

mod imp {
    use std::cell::RefCell;

    use gtk::{CompositeTemplate, Entry, glib, TemplateChild};
    use gtk::glib::subclass::InitializingObject;
    use gtk::subclass::prelude::{CompositeTemplate, ObjectImpl, ObjectSubclass, ObjectSubclassIsExt, WidgetClassSubclassExt, WindowImpl};
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};
    use gtk::traits::EditableExt;

    use crate::hangar::hangar::get_hangar;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_planner/preference_aircraft_dialog.ui")]
    pub struct AircraftDialog {
        #[template_child]
        pub ac_name: TemplateChild<Entry>,
        #[template_child]
        pub ac_cruise_speed: TemplateChild<Entry>,
        #[template_child]
        pub ac_cruise_alt: TemplateChild<Entry>,
        #[template_child]
        pub ac_climb_speed: TemplateChild<Entry>,
        #[template_child]
        pub ac_climb_rate: TemplateChild<Entry>,
        #[template_child]
        pub ac_sink_speed: TemplateChild<Entry>,
        #[template_child]
        pub ac_sink_rate: TemplateChild<Entry>,

        aircraft_name: RefCell<Option<String>>
    }

    impl AircraftDialog {

        pub fn set_aircraft(&self, name: String) {

            let hangar = get_hangar().imp();
            if let Some(aircraft) = hangar.get(name.as_str()) {
                self.ac_name.set_text(&aircraft.get_name());
                self.ac_cruise_speed.set_text(&aircraft.get_cruise_speed().to_string());
                self.ac_cruise_alt.set_text(&aircraft.get_cruise_altitude().to_string());
                self.ac_climb_speed.set_text(&aircraft.get_climb_speed().to_string());
                self.ac_climb_rate.set_text(&aircraft.get_climb_rate().to_string());
                self.ac_sink_speed.set_text(&aircraft.get_sink_speed().to_string());
                self.ac_sink_rate.set_text(&aircraft.get_sink_rate().to_string());

            }
            self.aircraft_name.replace(Some(name));

        }



    }

    #[glib::object_subclass]
    impl ObjectSubclass for AircraftDialog {
        const NAME: &'static str = "AircraftDialog";
        type Type = super::AircraftDialog;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }

    }


    impl ObjectImpl for AircraftDialog {}

    impl WidgetImpl for AircraftDialog {}
    impl WindowImpl for AircraftDialog {}

}

glib::wrapper! {
    pub struct AircraftDialog(ObjectSubclass<imp::AircraftDialog>)
        @extends gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl AircraftDialog {
    pub fn new() -> Self {
        glib::Object::new::<AircraftDialog>()
    }
}


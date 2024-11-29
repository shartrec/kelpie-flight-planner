/*
 * Copyright (c) 2003-2003-2023. Trevor Campbell and others.
 *
 * This file is part of Kelpie Flight Planner.
 *
 * Kelpie Flight Planner is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * Kelpie Flight Planner is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Kelpie Flight Planner; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 * Contributors:
 *      Trevor Campbell
 *
 */


use gtk::{Entry, gio, glib};
use gtk::prelude::{EditableExt, WidgetExt};

use crate::window::util::show_error_dialog;

mod imp {
    use gtk::{AlertDialog, Button, CompositeTemplate, Entry, glib, TemplateChild};
    use gtk::glib::clone;
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::{ButtonExt, EditableExt, GtkWindowExt, WidgetExt};
    use gtk::subclass::prelude::{CompositeTemplate, ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt, WidgetClassExt, WindowImpl};
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};

    use crate::hangar::hangar::get_hangar;
    use crate::model::aircraft::Aircraft;
    use crate::window::preferences::preference_edit_aircraft::{number_from, validate_not_empty, validate_numeric};

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
        #[template_child]
        pub btn_ok: TemplateChild<Button>,
        #[template_child]
        pub btn_cancel: TemplateChild<Button>,

    }

    impl AircraftDialog {
        pub fn set_aircraft(&self, name: Option<String>) {
            match name {
                Some(name) => {
                    let hangar = get_hangar().imp();
                    match hangar.get(name.as_str()) {
                        Some(aircraft) => {
                            self.ac_name.set_text(aircraft.get_name());
                            self.ac_cruise_speed.set_text(&aircraft.get_cruise_speed().to_string());
                            self.ac_cruise_alt.set_text(&aircraft.get_cruise_altitude().to_string());
                            self.ac_climb_speed.set_text(&aircraft.get_climb_speed().to_string());
                            self.ac_climb_rate.set_text(&aircraft.get_climb_rate().to_string());
                            self.ac_sink_speed.set_text(&aircraft.get_sink_speed().to_string());
                            self.ac_sink_rate.set_text(&aircraft.get_sink_rate().to_string());
                            self.ac_name.set_sensitive(false);
                        }
                        None => {
                            self.ac_name.set_sensitive(true);
                        }
                    }
                }
                None => {
                    self.ac_name.set_text("");
                    self.ac_cruise_speed.set_text("");
                    self.ac_cruise_alt.set_text("");
                    self.ac_climb_speed.set_text("");
                    self.ac_climb_rate.set_text("");
                    self.ac_sink_speed.set_text("");
                    self.ac_sink_rate.set_text("");
                    self.ac_name.set_sensitive(true);
                }
            }
        }

        fn validate(&self) -> bool {
            validate_not_empty(&self.ac_name,"Name" ) &&
            validate_numeric(&self.ac_cruise_speed,"Cruise Speed" ) &&
            validate_numeric(&self.ac_cruise_alt,"Cruise Altitude" ) &&
            validate_numeric(&self.ac_climb_speed,"Climb Speed" ) &&
            validate_numeric(&self.ac_climb_rate,"Climb Rate" ) &&
            validate_numeric(&self.ac_sink_speed,"Sink Speed" ) &&
            validate_numeric(&self.ac_sink_rate,"Sink Rate" )
        }

        fn save_aircraft(&self) -> bool {
            // Check we have a name
            if self.ac_name.text().is_empty() {
                let buttons = vec!["Ok".to_string()];
                let alert = AlertDialog::builder()
                    .message("Please enter a name")
                    .buttons(buttons)
                    .build();
                alert.show(Some(&self.obj().clone()));
                false
            } else {
                let hangar = get_hangar().imp();
                let string = self.ac_name.text();
                let name = string.as_str();
                let aircraft = Aircraft::new(
                    name.to_string(),
                    number_from(&self.ac_cruise_speed),
                    number_from(&self.ac_cruise_alt),
                    number_from(&self.ac_climb_speed),
                    number_from(&self.ac_climb_rate),
                    number_from(&self.ac_sink_speed),
                    number_from(&self.ac_sink_rate),
                    false,
                );
                hangar.put(aircraft);
                true
            }
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

    impl ObjectImpl for AircraftDialog {
        fn constructed(&self) {
            self.parent_constructed();

            self.btn_cancel.connect_clicked(clone!(#[weak(rename_to = window)] self, move |_button| {
               window.obj().close();
            }));

            self.btn_ok.connect_clicked(clone!(#[weak(rename_to = window)] self, move |_button| {
                if window.validate() && window.save_aircraft() {
                    window.obj().close();
                }
            }));
        }
    }

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

impl Default for AircraftDialog {
    fn default() -> Self {
        Self::new()
    }
}

fn number_from(entry: &Entry) -> i32 {
    entry.text().as_str().parse::<i32>().unwrap_or(0)
}
fn validate_numeric(entry: &Entry, name: &str) -> bool {
    match entry.text().as_str().parse::<i32>() {
        Ok(_) => {true}
        Err(_) => {
            show_error_dialog(&entry.root(), format!("{} should be numeric", name).as_str());
            false
        }
    }
}

fn validate_not_empty(entry: &Entry, name: &str) -> bool {
    if entry.text().as_str().is_empty() {
        show_error_dialog(&entry.root(), format!("{} is required", name).as_str());
        false
    } else {
        true
    }

}
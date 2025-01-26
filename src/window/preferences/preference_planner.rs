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

use gtk::{self, glib};

mod imp {
    use gtk::{CheckButton, CompositeTemplate, Entry, glib, TemplateChild};
    use gtk::glib::subclass::InitializingObject;
    use adw::prelude::{CheckButtonExt, EditableExt};
    use adw::subclass::prelude::{BoxImpl, CompositeTemplate, ObjectImpl, ObjectImplExt, ObjectSubclass, WidgetClassExt};
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};

    use crate::preference::{ADD_WAYPOINTS, AUTO_PLAN, MAX_DEVIATION, MAX_LEG_LENGTH, MIN_LEG_LENGTH, PLAN_TYPE, USE_FIXES, USE_GPS, USE_RADIO_BEACONS, VOR_ONLY, VOR_PREFERED};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_planner/preference_planner.ui")]
    pub struct PreferencePlannerPage {
        #[template_child]
        btn_auto_plan: TemplateChild<CheckButton>,
        #[template_child]
        planner_max_leg: TemplateChild<Entry>,
        #[template_child]
        planner_min_leg: TemplateChild<Entry>,
        #[template_child]
        planner_deviation: TemplateChild<Entry>,
        #[template_child]
        btn_use_radios: TemplateChild<CheckButton>,
        #[template_child]
        btn_use_fixes: TemplateChild<CheckButton>,
        #[template_child]
        btn_use_gps: TemplateChild<CheckButton>,
        #[template_child]
        btn_vor_only: TemplateChild<CheckButton>,
        #[template_child]
        btn_vor_preferred: TemplateChild<CheckButton>,
        #[template_child]
        btn_add_gps: TemplateChild<CheckButton>,

    }

    impl PreferencePlannerPage {
        fn initialise(&self) {
            let prefs = crate::preference::manager();
            self.btn_auto_plan.set_active(prefs.get::<bool>(AUTO_PLAN).unwrap_or(true));
            self.planner_max_leg.set_text(prefs.get::<String>(MAX_LEG_LENGTH).unwrap_or("100".to_string()).as_str());
            self.planner_min_leg.set_text(prefs.get::<String>(MIN_LEG_LENGTH).unwrap_or("10".to_string()).as_str());
            self.planner_deviation.set_text(prefs.get::<String>(MAX_DEVIATION).unwrap_or("10".to_string()).as_str());
            if let Some(_type) = prefs.get::<String>(PLAN_TYPE) {
                match _type.as_str() {
                    USE_RADIO_BEACONS => self.btn_use_radios.set_active(true),
                    USE_FIXES => self.btn_use_fixes.set_active(true),
                    USE_GPS => self.btn_use_gps.set_active(true),
                    _ => ()
                }
            }
            self.btn_vor_only.set_active(prefs.get::<bool>(VOR_ONLY).unwrap_or(false));
            self.btn_vor_preferred.set_active(prefs.get::<bool>(VOR_PREFERED).unwrap_or(false));
            self.btn_add_gps.set_active(prefs.get::<bool>(ADD_WAYPOINTS).unwrap_or(false));
        }
    }

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


    impl ObjectImpl for PreferencePlannerPage {
        fn constructed(&self) {
            self.parent_constructed();

            self.initialise();

            self.btn_auto_plan.connect_toggled(|button| {
                crate::preference::manager().put(AUTO_PLAN, button.is_active());
            });
            self.planner_max_leg.connect_changed(|editable| {
                crate::preference::manager().put(MAX_LEG_LENGTH, editable.text());
            });
            self.planner_min_leg.connect_changed(|editable| {
                crate::preference::manager().put(MIN_LEG_LENGTH, editable.text());
            });
            self.planner_deviation.connect_changed(|editable| {
                crate::preference::manager().put(MAX_DEVIATION, editable.text());
            });
            self.btn_use_radios.connect_toggled(|button| {
                if button.is_active() {
                    crate::preference::manager().put(PLAN_TYPE, USE_RADIO_BEACONS);
                }
            });
            self.btn_use_fixes.connect_toggled(|button| {
                if button.is_active() {
                    crate::preference::manager().put(PLAN_TYPE, USE_FIXES);
                }
            });
            self.btn_use_gps.connect_toggled(|button| {
                if button.is_active() {
                    crate::preference::manager().put(PLAN_TYPE, USE_GPS);
                }
            });
            self.btn_vor_only.connect_toggled(|button| {
                crate::preference::manager().put(VOR_ONLY, button.is_active());
            });
            self.btn_vor_preferred.connect_toggled(|button| {
                crate::preference::manager().put(VOR_PREFERED, button.is_active());
            });
            self.btn_add_gps.connect_toggled(|button| {
                crate::preference::manager().put(ADD_WAYPOINTS, button.is_active());
            });
        }
    }

    impl BoxImpl for PreferencePlannerPage {}

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

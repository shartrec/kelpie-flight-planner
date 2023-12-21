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
    use gtk::{Button, CheckButton, CompositeTemplate, Entry, glib, TemplateChild};
    use gtk::glib::clone;
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::{ButtonExt, CheckButtonExt, EditableExt};
    use gtk::subclass::prelude::{BoxImpl, CompositeTemplate, ObjectImpl, ObjectImplExt, ObjectSubclass, WidgetClassSubclassExt};
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};

    use crate::preference::*;
    use crate::window::preferences::process_file_browse;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_planner/preference_general.ui")]
    pub struct PreferenceGeneralPage {
        #[template_child]
        fg_path: TemplateChild<Entry>,
        #[template_child]
        fg_browse: TemplateChild<Button>,
        #[template_child]
        btn_use_dft_paths: TemplateChild<CheckButton>,
        #[template_child]
        apt_path: TemplateChild<Entry>,
        #[template_child]
        apt_browse: TemplateChild<Button>,
        #[template_child]
        nav_path: TemplateChild<Entry>,
        #[template_child]
        nav_browse: TemplateChild<Button>,
        #[template_child]
        fix_path: TemplateChild<Entry>,
        #[template_child]
        fix_browse: TemplateChild<Button>,
        #[template_child]
        btn_use_mag_hdg: TemplateChild<CheckButton>,
        #[template_child]
        btn_dist_nm: TemplateChild<CheckButton>,
        #[template_child]
        btn_dist_mi: TemplateChild<CheckButton>,
        #[template_child]
        btn_dist_km: TemplateChild<CheckButton>,

    }

    impl PreferenceGeneralPage {

        fn initialise(&self) {
            let prefs = crate::preference::manager();
            self.btn_use_mag_hdg.set_active(prefs.get::<bool>(USE_MAGNETIC_HEADINGS).unwrap_or(false));
            if let Some(unit) = prefs.get::<String>(UNITS) {
                match unit.as_str() {
                    UNITS_NM => self.btn_dist_nm.set_active(true),
                    UNITS_MI => self.btn_dist_mi.set_active(true),
                    UNITS_KM => self.btn_dist_km.set_active(true),
                    _ => ()
                }
            }
            self.btn_use_dft_paths.set_active(prefs.get::<bool>(FGFS_USE_DFT_PATH).unwrap_or(false));
            self.fg_path.set_text(prefs.get::<String>(FGFS_DIR).unwrap_or("".to_string()).as_str());
            self.apt_path.set_text(prefs.get::<String>(AIRPORTS_PATH).unwrap_or("".to_string()).as_str());
            self.nav_path.set_text(prefs.get::<String>(NAVAIDS_PATH).unwrap_or("".to_string()).as_str());
            self.fix_path.set_text(prefs.get::<String>(FIXES_PATH).unwrap_or("".to_string()).as_str());
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferenceGeneralPage {
        const NAME: &'static str = "PreferenceGeneralPage";
        type Type = super::PreferenceGeneralPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }

    }


    impl ObjectImpl for PreferenceGeneralPage {
        fn constructed(&self) {
            self.parent_constructed();

            self.initialise();

            self.btn_use_dft_paths.connect_toggled(| button| {
                    crate::preference::manager().put(FGFS_USE_DFT_PATH, button.is_active());
            });
            self.btn_use_mag_hdg.connect_toggled(|button| {
                    crate::preference::manager().put(USE_MAGNETIC_HEADINGS, button.is_active());
            });
            self.btn_dist_nm.connect_toggled(| button | {
                if button.is_active() {
                    crate::preference::manager().put(UNITS, UNITS_NM);
                }
            });
            self.btn_dist_mi.connect_toggled(| button | {
                if button.is_active() {
                    crate::preference::manager().put(UNITS, UNITS_MI);
                }
            });
            self.btn_dist_km.connect_toggled( | button| {
                if button.is_active() {
                    crate::preference::manager().put(UNITS, UNITS_KM);
                }
            });
            self.fg_path.connect_changed(| editable | {
                crate::preference::manager().put(FGFS_DIR, editable.text());
            });
            self.apt_path.connect_changed(| editable | {
                crate::preference::manager().put(AIRPORTS_PATH, editable.text());
            });
            self.nav_path.connect_changed(| editable | {
                crate::preference::manager().put(NAVAIDS_PATH, editable.text());
            });
            self.fix_path.connect_changed(| editable | {
                crate::preference::manager().put(FIXES_PATH, editable.text());
            });
            self.fg_browse.connect_clicked(clone!(@weak self as view => move | button | {
                process_file_browse(view.fg_path.clone(), button.clone(), "Flightgear data directory", true);
            }));
            self.apt_browse.connect_clicked(clone!(@weak self as view => move | button | {
                process_file_browse(view.apt_path.clone(), button.clone(), "Location for Flightgear airport data", false);
            }));
            self.nav_browse.connect_clicked(clone!(@weak self as view => move | button | {
                process_file_browse(view.nav_path.clone(), button.clone(), "Location for Flightgear navaid data", false);
            }));
            self.fix_browse.connect_clicked(clone!(@weak self as view => move | button | {
                process_file_browse(view.fix_path.clone(), button.clone(), "Location for Flightgear fix data", false);
            }));
        }
    }

    impl BoxImpl for PreferenceGeneralPage {
    }
    impl WidgetImpl for PreferenceGeneralPage {
    }

}

glib::wrapper! {
    pub struct PreferenceGeneralPage(ObjectSubclass<imp::PreferenceGeneralPage>)
        @extends gtk::Box, gtk::Widget;
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

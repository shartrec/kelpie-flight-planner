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

    use crate::preference::{FGFS_LINK_ENABLED, FGFS_LINK_HOST, FGFS_LINK_PORT};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_planner/preference_fglink.ui")]
    pub struct PreferenceFgLinkPage {
        #[template_child]
        btn_fglink_enabled: TemplateChild<CheckButton>,
        #[template_child]
        fg_host: TemplateChild<Entry>,
        #[template_child]
        fg_port: TemplateChild<Entry>,

    }

    impl PreferenceFgLinkPage {
        fn initialise(&self) {
            let prefs = crate::preference::manager();
            self.btn_fglink_enabled.set_active(prefs.get::<bool>(FGFS_LINK_ENABLED).unwrap_or(false));
            self.fg_host.set_text(prefs.get::<String>(FGFS_LINK_HOST).unwrap_or("100".to_string()).as_str());
            self.fg_port.set_text(prefs.get::<String>(FGFS_LINK_PORT).unwrap_or("10".to_string()).as_str());
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferenceFgLinkPage {
        const NAME: &'static str = "PreferenceFgLinkPage";
        type Type = super::PreferenceFgLinkPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }


    impl ObjectImpl for PreferenceFgLinkPage {
        fn constructed(&self) {
            self.parent_constructed();

            self.initialise();

            self.btn_fglink_enabled.connect_toggled(|button| {
                crate::preference::manager().put(FGFS_LINK_ENABLED, button.is_active());
            });
            self.fg_host.connect_changed(|editable| {
                crate::preference::manager().put(FGFS_LINK_HOST, editable.text());
            });
            self.fg_port.connect_changed(|editable| {
                crate::preference::manager().put(FGFS_LINK_PORT, editable.text());
            });
        }
    }

    impl BoxImpl for PreferenceFgLinkPage {}

    impl WidgetImpl for PreferenceFgLinkPage {}
}

glib::wrapper! {
    pub struct PreferenceFgLinkPage(ObjectSubclass<imp::PreferenceFgLinkPage>)
        @extends gtk::Box, gtk::Widget;
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

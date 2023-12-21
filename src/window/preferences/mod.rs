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

use std::path::Path;

use gtk::{self, Button, Entry, FileDialog, FileFilter, gio, glib, Window};
use gtk::gio::{Cancellable, File, ListStore};
use gtk::prelude::{Cast, EditableExt, FileExt, StaticType, WidgetExt};

mod preference_general;
mod preference_fglink;
mod preference_planner;
mod preference_aircraft;
mod preference_edit_aircraft;

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
impl Default for PreferenceDialog {
    fn default() -> Self {
        PreferenceDialog::new()
    }
}

pub fn process_file_browse (field: Entry, button: Button, title: &str, is_folder: bool) {

    let text = field.text();
    let path = Path::new(&text);
    let f = File::for_path(path);



    let dialog = FileDialog::new();
    dialog.set_initial_file(Some(&f));
    dialog.set_modal(true);
    dialog.set_title(title);
    if !is_folder {
        let store = ListStore::new(FileFilter::static_type());
        let filter = FileFilter::new();
        filter.add_suffix("dat.gz");
        store.append(&filter);
        let filter = FileFilter::new();
        filter.add_suffix("dat");
        store.append(&filter);
        let filter = FileFilter::new();
        filter.add_pattern("*");
        store.append(&filter);
        dialog.set_filters(&store);
    }
    let win = match &button.root() {
        Some(r) => {
            let window = r.clone().downcast::<Window>().unwrap().clone();
            Some(window)
        }
        None => None,
    };

    let closure = move | result: Result<File, _>| {
        if let Ok(file) = result {
            if let Some(path) = file.path() {
                let s = path.to_str();
                {
                    if let Some(s) = s {
                        field.set_text(s);
                    };
                }
            }
        }
    };
    if is_folder {
        dialog.select_folder(win.as_ref(), Some(&Cancellable::default()), closure);
    } else {
        dialog.open(win.as_ref(), Some(&Cancellable::default()), closure);
    }
}
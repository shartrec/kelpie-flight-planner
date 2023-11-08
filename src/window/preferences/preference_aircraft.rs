use gtk::{self, glib};

mod imp {
    use std::cell::RefCell;
    use std::ops::Deref;
    use gtk::{Button, CompositeTemplate, glib, Label, ListItem, ListView, SignalListItemFactory, SingleSelection, StringObject, TemplateChild, Window};
    use gtk::glib::{Cast, CastNone, clone};
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::{GtkWindowExt, SelectionModelExt};
    use gtk::subclass::prelude::{BoxImpl, ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassIsExt, WidgetClassSubclassExt};
    use gtk::subclass::widget::{CompositeTemplate, CompositeTemplateInitializingExt, WidgetImpl};
    use gtk::traits::{ButtonExt, WidgetExt};

    use crate::hangar::hangar::get_hangar;
    use crate::window::preferences::preference_edit_aircraft::AircraftDialog;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_planner/preference_aircraft.ui")]
    pub struct PreferenceAircraftPage {
        #[template_child]
        pub aircraft_list: TemplateChild<ListView>,
        #[template_child]
        pub aircraft_add: TemplateChild<Button>,
        #[template_child]
        pub aircraft_edit: TemplateChild<Button>,
        #[template_child]
        pub aircraft_delete: TemplateChild<Button>,
        #[template_child]
        pub aircraft_default: TemplateChild<Button>,

        aircraft_dialog: RefCell<Option<AircraftDialog>>,
    }

    impl PreferenceAircraftPage {
        fn setup_aircraft_list(&self) {
            let factory = SignalListItemFactory::new();
            factory.connect_setup(move |_, list_item| {
                let label = Label::new(None);
                list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .set_child(Some(&label));
            });

            let selection_model = SingleSelection::new(Some(get_hangar().clone()));
            self.aircraft_list.set_factory(Some(&factory));
            self.aircraft_list.set_model(Some(&selection_model));

            factory.connect_bind(move |_, list_item| {
                // Get `IntegerObject` from `ListItem`
                let string_object = list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .item()
                    .and_downcast::<StringObject>()
                    .expect("The item has to be an `IntegerObject`.");

                // Get `Label` from `ListItem`
                let label = list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .child()
                    .and_downcast::<Label>()
                    .expect("The child has to be a `Label`.");

                // Set "label"
                label.set_label(&string_object.string().to_string());
                label.set_xalign(0.0);
            });
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferenceAircraftPage {
        const NAME: &'static str = "PreferenceAircraftPage";
        type Type = super::PreferenceAircraftPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferenceAircraftPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_aircraft_list();

            let pref_dialog = AircraftDialog::new();
            pref_dialog.set_modal(true);
            pref_dialog.set_hide_on_close(true);
            self.aircraft_dialog.replace(Some(pref_dialog));

            self.aircraft_list.model().unwrap().connect_selection_changed(clone!(@weak self as view => move |model, _position, _count| {
                let selection = model.selection();
                if selection.is_empty() {
                    view.aircraft_edit.set_sensitive(false);
                    view.aircraft_delete.set_sensitive(false);
                    view.aircraft_default.set_sensitive(false);
                } else {
                    view.aircraft_edit.set_sensitive(true);
                    view.aircraft_delete.set_sensitive(true);
                    view.aircraft_default.set_sensitive(true);
                }
            }));

            self.aircraft_edit.connect_clicked(clone!(@weak self as view => move | button | {
                let dialog_ref = view.aircraft_dialog.borrow();
                if let Some(dialog) = dialog_ref.deref() {
                    // Get the selectiion
                    if let Some(selection) = view.aircraft_list.model() {
                        let s = selection.selection();
                        if !s.is_empty() {
                            let index = s.nth(0);
                            if let Some(aircraft) = get_hangar().imp().aircraft_at(index) {
                                dialog.imp().set_aircraft(Some(aircraft.get_name().to_string()));
                            }
                        }
                    }
                    let r = button.root().unwrap();
                    let our_window = r.clone().downcast::<Window>().unwrap();
                    dialog.set_transient_for(Some(&our_window));
                    dialog.present();
                }
            }));

            self.aircraft_add.connect_clicked(clone!(@weak self as view => move | button | {
                let dialog_ref = view.aircraft_dialog.borrow();
                if let Some(dialog) = dialog_ref.deref() {
                    // Get the selectiion
                    dialog.imp().set_aircraft(None);
                    let r = button.root().unwrap();
                    let our_window = r.clone().downcast::<Window>().unwrap();
                    dialog.set_transient_for(Some(&our_window));
                    dialog.present();
                }
            }));

            self.aircraft_delete.connect_clicked(clone!(@weak self as view => move | _button | {
                    // Get the selectiion
                    if let Some(selection) = view.aircraft_list.model() {
                        let s = selection.selection();
                        if !s.is_empty() {
                            let index = s.nth(0);
                            if let Some(aircraft) = get_hangar().imp().aircraft_at(index) {
                                get_hangar().imp().remove(aircraft.get_name());
                            }
                        }
                    }
            }));

            self.aircraft_default.connect_clicked(clone!(@weak self as view => move | _button | {
                    // Get the selectiion
                    if let Some(selection) = view.aircraft_list.model() {
                        let s = selection.selection();
                        if !s.is_empty() {
                            let index = s.nth(0);
                            if let Some(aircraft) = get_hangar().imp().aircraft_at(index) {
                                get_hangar().imp().set_default(aircraft.get_name());
                            }
                        }
                    }
            }));

        }

        fn dispose(&self) {
            if let Some(dialog) = self.aircraft_dialog.borrow().deref() {
                dialog.unparent();
            }
        }
    }

    impl BoxImpl for PreferenceAircraftPage {}

    impl WidgetImpl for PreferenceAircraftPage {}
}

glib::wrapper! {
    pub struct PreferenceAircraftPage(ObjectSubclass<imp::PreferenceAircraftPage>)
        @extends gtk::Box, gtk::Widget;
}

impl PreferenceAircraftPage {
    pub fn new() -> Self {
        glib::Object::new::<PreferenceAircraftPage>()
    }
}

impl Default for PreferenceAircraftPage {
    fn default() -> Self {
        Self::new()
    }
}

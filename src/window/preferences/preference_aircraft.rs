use gtk::{self, glib};

mod imp {
    use gtk::{Button, CompositeTemplate, glib, Label, ListItem, ListView, SignalListItemFactory, SingleSelection, StringObject, TemplateChild, Window};
    use gtk::glib::{Cast, CastNone, clone};
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::{GtkWindowExt, SelectionModelExt};
    use gtk::subclass::prelude::{BoxImpl, ListModelImpl, ObjectImpl, ObjectSubclass, ObjectSubclassIsExt, WidgetClassSubclassExt};
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
            self.setup_aircraft_list();

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
                if let Some(r) =  &button.root() {
                    let our_window = r.clone().downcast::<Window>().unwrap();
                    let pref_dialog = AircraftDialog::new();
                    pref_dialog.set_transient_for(Some(&our_window));
                    // Get the selectiion
                    if let Some(selection) = view.aircraft_list.model() {
                        let s = selection.selection();
                        if !s.is_empty() {
                            let index = s.nth(0);
                            if let Some(name) = get_hangar().imp().item(index) {
                                pref_dialog.imp().set_aircraft(name.downcast::<StringObject>().expect("Ouch").string().to_string());
                            }
                        }
                    }
                    pref_dialog.show();
                }
            }));
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

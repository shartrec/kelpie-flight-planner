use glib::Object;
use gtk::{Application, gio, glib};
use gtk::gio::{SimpleAction, SimpleActionGroup};
use gtk::glib::clone;
use gtk::prelude::{ActionMapExt, WidgetExt};
use gtk::subclass::prelude::ObjectSubclassIsExt;

pub mod imp;
mod airport_view;
mod navaid_view;
mod fix_view;
mod plan_view;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn setup_actions(&self) {
        let action_copy = SimpleAction::new("copy", None);

        action_copy.connect_activate(clone!(@weak self as window => move |action, parameter| {
            // Get state
            println!("Copy menu clicked")
        }));
        let actions = SimpleActionGroup::new();
        self.insert_action_group("plan", Some(&actions));
        actions.add_action(&action_copy);

    }

}

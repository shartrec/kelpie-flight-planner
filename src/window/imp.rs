use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate};

use crate::events::{Event, Subscriber};

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/shartrec/kelpie_planner/window.ui")]
pub struct Window {
    #[template_child]
    pub button: TemplateChild<Button>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "KelpiePlannerWindow";
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        // Connect to "clicked" signal of `button`
        self.button.connect_clicked(move |button| {
            // Set the label to "Hello World!" after the button has been clicked on
            button.set_label("Hello World!");
        });

        subscribe(Event::AirportsLoaded, do_airports_loaded);
        subscribe(Event::NavaidsLoaded, do_navaids_loaded);
        subscribe(Event::FixesLoaded, do_fixes_loaded);
    }
}

// Trait to allow us to add menubas
impl BuildableImpl for Window {}

// Trait shared by all widgets
impl WidgetImpl for Window {}

// Trait shared by all windows
impl WindowImpl for Window {}

// Trait shared by all application windows
impl ApplicationWindowImpl for Window {}

fn subscribe(event: Event, listener: Subscriber) {
    match crate::events::get_publisher().write() {
        Ok(p) => p.subscribe(event, listener),
        Err(_) => (),
    }
}

fn do_airports_loaded() {
	println!("We now have {} airports", crate::earth::get_earth_model().get_airports().read().unwrap().len());
}
fn do_navaids_loaded() {
	println!("We now have {} navaids", crate::earth::get_earth_model().get_navaids().read().unwrap().len());
}
fn do_fixes_loaded() {
	println!("We now have {} fixes", crate::earth::get_earth_model().get_fixes().read().unwrap().len());
}

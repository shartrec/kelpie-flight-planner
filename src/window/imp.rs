use std::thread;

use glib::subclass::InitializingObject;
use gtk::{Builder, Button, CompositeTemplate, glib, Stack};
use gtk::gio::File;
use gtk::glib::{MainContext, PRIORITY_DEFAULT, PropertyGet};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::event::Event;
use crate::window::airport_view::AirportView;
use crate::window::fix_view::FixView;
use crate::window::navaid_view::NavaidView;
use crate::window::plan_view::PlanView;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/shartrec/kelpie_planner/window.ui")]
pub struct Window {
    #[template_child]
    pub airport_view: TemplateChild<AirportView>,
    #[template_child]
    pub navaid_view: TemplateChild<NavaidView>,
    #[template_child]
    pub fix_view: TemplateChild<FixView>,
    #[template_child]
    pub plan_stack: TemplateChild<Stack>,
//todo remove
    #[template_child]
    pub map_button: TemplateChild<Button>,
}

impl Window {
    pub(crate) fn load_plan_from_files(&self, files: &[File]) {
        println!("Nead to load file{:?} here", files[0].path());
        // todo
    }

    pub(crate) fn new_plan(&self) {
        let view = PlanView::new();
        view.imp().new_plan();
        self.plan_stack.add_titled(&view, Some("newxx"), &"New Plan");
    }

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

        let obj = self.obj();
        obj.setup_actions();

        //todo Remove this
        // Connect to "clicked" signal of `button`
        self.map_button.connect_clicked(move |button| {
            // Set the label to "Hello World!" after the button has been clicked on
        });

        let (tx, rx) = MainContext::channel(PRIORITY_DEFAULT);
        let transmitter = tx.clone();

        thread::spawn(move || {
            crate::earth::initialise(transmitter);
        });

        let airport_view = Box::new(self.airport_view.clone());
        let navaid_view = Box::new(self.navaid_view.clone());
        let fix_view = Box::new(self.fix_view.clone());
        rx.attach(None, move |ev: Event| {
            match ev {
                Event::AirportsLoaded => airport_view.imp().airports_loaded(),
                Event::NavaidsLoaded => navaid_view.imp().navaids_loaded(),
                Event::FixesLoaded => fix_view.imp().fixes_loaded(),
            }
            glib::source::Continue(true)
        });
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

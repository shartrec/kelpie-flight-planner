use std::thread;

use glib::subclass::InitializingObject;
use gtk::{CompositeTemplate, glib, Notebook, Paned, Stack};
use gtk::gio::File;
use gtk::glib::{MainContext, PRIORITY_DEFAULT};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::event::Event;
use crate::window::airport_map_view::AirportMapView;
use crate::window::airport_view::AirportView;
use crate::window::fix_view::FixView;
use crate::window::navaid_view::NavaidView;
use crate::window::plan_view::PlanView;
use crate::window::world_map_view::WorldMapView;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/shartrec/kelpie_planner/window.ui")]
pub struct Window {
    #[template_child]
    pub pane_1v: TemplateChild<Paned>,
    #[template_child]
    pub pane_1h: TemplateChild<Paned>,
    #[template_child]
    pub search_notebook: TemplateChild<Notebook>,
    #[template_child]
    pub airport_view: TemplateChild<AirportView>,
    #[template_child]
    pub navaid_view: TemplateChild<NavaidView>,
    #[template_child]
    pub fix_view: TemplateChild<FixView>,
    #[template_child]
    pub airport_map_view: TemplateChild<AirportMapView>,
    #[template_child]
    pub world_map_view: TemplateChild<WorldMapView>,
    #[template_child]
    pub plan_stack: TemplateChild<Stack>,
}

impl Window {
    pub(crate) fn load_plan_from_files(&self, files: &[File]) {
        todo!("Nead to load file{:?} here", files[0].path());
        // todo
    }

    pub(crate) fn new_plan(&self) {
        let view = PlanView::new();
        view.imp().new_plan();
        self.plan_stack.add_titled(&view, Some("newxx"), &"New Plan");
        self.plan_stack.set_visible_child(&view);
    }

    fn layout_panels(&self) {
        let pref = crate::preference::manager();

        // Set the size of the window
        if let Some(p) = pref.get::<i32>("vertical-split-pos") {
            self.pane_1v.set_position(p);
        }
        if let Some(p) = pref.get::<i32>("horizontal-split-pos") {
            self.pane_1h.set_position(p);
        }
    }
    fn save_panel_layout(&self) {
        let pref = crate::preference::manager();

        // Set the size of the window
        pref.put("vertical-split-pos", self.pane_1v.position());
        pref.put("horizontal-split-pos", self.pane_1h.position());
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
        obj.load_window_size();

        self.layout_panels();


        let airport_view = Box::new(self.airport_view.clone());
        let navaid_view = Box::new(self.navaid_view.clone());
        let fix_view = Box::new(self.fix_view.clone());

        let (tx, rx) = MainContext::channel(PRIORITY_DEFAULT);
        let transmitter = tx.clone();

        thread::spawn(move || {
            crate::earth::initialise(transmitter);
        });
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

// Trait to allow us to add menubars
impl BuildableImpl for Window {}

// Trait shared by all widgets
impl WidgetImpl for Window {}

// Trait shared by all windows
impl WindowImpl for Window {
    // Save window state right before the window will be closed
    fn close_request(&self) -> glib::signal::Inhibit {
        self.save_panel_layout();
        // Save window size
        self.obj()
            .save_window_size()
            .expect("Failed to save window state");
        // Allow to invoke other event handlers
        self.parent_close_request()
    }
}

// Trait shared by all application windows
impl ApplicationWindowImpl for Window {}

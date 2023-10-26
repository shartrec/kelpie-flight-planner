/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use gtk::{self, CompositeTemplate, glib, prelude::*, subclass::prelude::*};

mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::thread;

    use glib::subclass::InitializingObject;
    use gtk::{cairo::Context, DrawingArea};
    use gtk::glib::{clone, MainContext, PRIORITY_DEFAULT, PropertySet};

    use crate::event::Event;
    use crate::model::airport::Airport;
    use crate::util::airport_painter::AirportPainter;

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/airport_map_view.ui")]
    pub struct AirportMapView {
        #[template_child]
        pub airport_map_window: TemplateChild<DrawingArea>,

        airport: RefCell<Option<Arc<Airport>>>,
    }

    impl AirportMapView {
        pub fn set_airport(&self, airport: Arc<Airport>) {
            self.airport.set(Some(airport.clone()));
            let (tx, rx) = MainContext::channel(PRIORITY_DEFAULT);

            // Ensure the runways & taxiways are loaded. This can happen in another thread.
            let ap = airport.clone();
            thread::spawn(move || {
                let _ = ap.get_runways();
                let _ = tx.clone().send(Event::AirportsLoaded);
            });

            let view = self.airport_map_window.clone();
            rx.attach(None, move |ev: Event| {
                match ev {
                    Event::AirportsLoaded => view.queue_draw(),
                    _ => (),
                }
                glib::source::Continue(true)
            });

            // let _ = airport.get_runways();
            // &self.airport_map_window.queue_draw();
        }

        pub fn initialise(&self) -> () {}

        fn draw_function(&self, area: &gtk::DrawingArea, cr: &Context) {
            let maybe_airport = self.airport.clone().into_inner();
            match maybe_airport {
                Some(airport) => {
                    let airport_painter = AirportPainter {
                        draw_taxiways: true,
                        draw_runways: true,
                        draw_runway_list: true,
                        draw_compass_rose: true,
                    };
                    airport_painter.draw_airport(&airport, area, cr);
                }
                _ => {
                    ();
                }
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AirportMapView {
        const NAME: &'static str = "AirportMapView";
        type Type = super::AirportMapView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AirportMapView {
        fn constructed(&self) {
            self.parent_constructed();
            self.initialise();

            self.airport_map_window.set_draw_func(
                clone!(@weak self as view => move |area, cr, _x, _y| {
                    view.draw_function(&area, cr);
                }),
            );
        }
    }

    impl WidgetImpl for AirportMapView {}
}

glib::wrapper! {
    pub struct AirportMapView(ObjectSubclass<imp::AirportMapView>) @extends gtk::Widget;
}

impl AirportMapView {
    pub fn new() -> Self {
        glib::Object::new::<AirportMapView>()
    }
}

impl Default for AirportMapView {
    fn default() -> Self {
        Self::new()
    }
}

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
#![forbid(unsafe_code)]

use gtk::{self, CompositeTemplate, glib, prelude::*, subclass::prelude::*};

mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::thread;

    use glib::subclass::InitializingObject;
    use gtk::{cairo::Context, DrawingArea, Label};
    use gtk::glib::{clone, MainContext};

    use crate::event::Event;
    use crate::model::airport::{Airport, RunwayType};
    use crate::util::airport_painter::AirportPainter;

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/airport_map_view.ui")]
    pub struct AirportMapView {
        #[template_child]
        pub airport_map_window: TemplateChild<DrawingArea>,
        #[template_child]
        pub runway_list: TemplateChild<Label>,

        airport: RefCell<Option<Arc<Airport>>>,
    }

    impl AirportMapView {
        pub fn set_airport(&self, airport: Arc<Airport>) {
            // Clear the window
            self.airport.replace(None);
            self.airport_map_window.queue_draw();
            self.runway_list.set_label("");

            let (tx, rx) = async_channel::bounded::<Event>(1);
            // Ensure the runways & taxiways are loaded. This can happen in another thread.
            let ap = airport.clone();
            thread::spawn(move || {
                let _ = ap.get_runways();
                let _ = tx.try_send(Event::AirportsLoaded);
            });

            MainContext::default().spawn_local(clone!(#[weak(rename_to = view)] self, async move {
                while let Ok(ev) = rx.recv().await {
                    if let Event::AirportsLoaded = ev {
                        view.airport.replace(Some(airport.clone()));
                        view.draw_runway_list(&airport);
                        view.airport_map_window.queue_draw()
                    }
                }
            }));
        }

        pub fn initialise(&self) {}

        fn draw_function(&self, area: &DrawingArea, cr: &Context) {
            let maybe_airport = self.airport.clone().into_inner();
            if let Some(airport) = maybe_airport {
                let airport_painter = AirportPainter {
                    draw_taxiways: true,
                    draw_runways: true,
                    draw_compass_rose: true,
                };
                airport_painter.draw_airport(&airport, area, cr);
            }
        }

        fn draw_runway_list(&self, airport: &Airport) {
            let mut buf = String::new();
            buf.push_str("Runways");
            let runways = airport
                .get_runways()
                .read()
                .expect("Could not get airport lock");
            for runway in runways.iter() {
                let ils = airport.get_ils(runway.get_number());
                let ils_opp = airport.get_ils(&runway.get_opposite_number());
                if runway.get_runway_type().is_some_and(|t| t == RunwayType::Runway) {
                    let text = if ils.is_none() && ils_opp.is_none() {
                        format!("{:<9} {:<9}", runway.get_number_pair(), runway.get_length(), )
                    } else {
                        format!(
                            "{:<9} {:<9} ILS {:0.3} / {:0.3}",
                            runway.get_number_pair(),
                            runway.get_length(),
                            ils.unwrap_or(0.),
                            ils_opp.unwrap_or(0.),
                        )
                    };
                    buf.push('\n');
                    buf.push_str(text.as_str());
                }
            }
            self.runway_list.set_label(buf.as_str());
        }

    }

    #[glib::object_subclass]
    impl ObjectSubclass for AirportMapView {
        const NAME: &'static str = "AirportMapView";
        type Type = super::AirportMapView;
        type ParentType = gtk::Box;

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
                clone!(#[weak(rename_to = view)] self, move |area, cr, _x, _y| {
                    view.draw_function(area, cr);
                }),
            );
        }
    }

    impl BoxImpl for AirportMapView {}

    impl WidgetImpl for AirportMapView {}
}

glib::wrapper! {
    pub struct AirportMapView(ObjectSubclass<imp::AirportMapView>)
        @extends gtk::Widget, gtk::Box;
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

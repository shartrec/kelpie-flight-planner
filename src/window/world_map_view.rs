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

use async_channel::Sender;
use chrono::Utc;
use gtk::{self, glib, CompositeTemplate};
use crate::earth::solar::subsolar_point;
use crate::util::fg_link::{get_aircraft_position, AircraftPositionInfo};

mod imp {
    use adw::gdk::ModifierType;
    use adw::prelude::*;
    use adw::subclass::prelude::*;
    use gtk::gdk::Rectangle;
    use gtk::gio::{Menu, MenuItem, SimpleAction, SimpleActionGroup};
    use gtk::glib::subclass::InitializingObject;
    use gtk::glib::{clone, MainContext, Propagation};
    use gtk::graphene::Point;
    use gtk::{glib, Button, EventControllerScroll, EventControllerScrollFlags, GLArea, PopoverMenu, ScrolledWindow, ToggleButton};
    use log::error;
    use scheduling::SchedulerHandle;
    use std::cell::{Cell, RefCell};
    use std::cmp::Ordering::Equal;
    use std::ops::Deref;
    use std::rc::Rc;
    use std::sync::Arc;
    use gettextrs::gettext;
    use crate::earth::coordinate::Coordinate;
    use crate::event::Event;
    use crate::model::airport::{Airport, AirportType};
    use crate::model::location::Location;
    use crate::model::navaid::{Navaid, NavaidType};
    use crate::model::plan::Plan;
    use crate::model::waypoint::Waypoint;
    use crate::util::fg_link::AircraftPositionInfo;
    use crate::window::render_gl::Renderer;
    use crate::window::util::{get_airport_map_view, get_airport_view, get_fix_view, get_navaid_view, get_plan_view, show_airport_map_view, show_airport_view, show_fix_view, show_navaid_view};
    use crate::{earth, event};

    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/world_map_view.ui")]
    pub struct WorldMapView {
        #[template_child]
        map_window: TemplateChild<ScrolledWindow>,
        #[template_child]
        gl_area: TemplateChild<GLArea>,
        #[template_child]
        btn_zoom_in: TemplateChild<Button>,
        #[template_child]
        btn_zoom_out: TemplateChild<Button>,
        #[template_child]
        btn_show_airports: TemplateChild<ToggleButton>,
        #[template_child]
        btn_show_navaids: TemplateChild<ToggleButton>,

        popover: RefCell<Option<PopoverMenu>>,
        view_action: RefCell<Option<SimpleAction>>,
        add_action: RefCell<Option<SimpleAction>>,
        add_nav_action: RefCell<Option<SimpleAction>>,

        renderer: RefCell<Option<Renderer>>,
        drag_start: RefCell<Option<[f64; 2]>>,
        drag_last: RefCell<Option<[f64; 2]>>,
        zoom_level: Cell<f32>,
        scheduler_handle_ap: RefCell<Option<SchedulerHandle>>,
        aircraft_position_info: RefCell<Option<AircraftPositionInfo>>,
        scheduler_handle_ssp: RefCell<Option<SchedulerHandle>>,
        sub_solar_point: RefCell<(f64, f64)>,
    }

    const MAX_ZOOM: f32 = 50.;

    impl WorldMapView {
        pub fn initialise(&self) {
            self.zoom_level.replace(1.0);

            if let Some(rx) = event::manager().register_listener() {
                MainContext::default().spawn_local(clone!(#[weak(rename_to = view)] self, async move {
                    while let Ok(ev) = rx.recv().await {
                        match ev {
                            Event::PlanChanged => {
                                if let Some(renderer) = view.renderer.borrow().as_ref() {
                                    renderer.plan_changed();
                                    view.gl_area.queue_draw();
                                }
                            }
                            Event::AirportsLoaded => {
                                if let Some(renderer) = view.renderer.borrow().as_ref() {
                                    renderer.airports_loaded();
                                    view.gl_area.queue_draw();
                                }
                            }
                            Event::NavaidsLoaded => {
                                if let Some(renderer) = view.renderer.borrow().as_ref() {
                                    renderer.navaids_loaded();
                                    view.gl_area.queue_draw();
                                }
                            }
                        _ => {}}
                    }
                }));
            }
            // Set up the scheduled tasks to query the aircraft position
            let (tx, rx) = async_channel::unbounded::<Option<AircraftPositionInfo>>();
            MainContext::default().spawn_local(clone!(#[weak(rename_to = view)] self, async move {
                while let Ok(ap) = rx.recv().await {
                    if let Some(renderer) = view.renderer.borrow().as_ref() {
                        if *view.aircraft_position_info.borrow().deref() != ap {
                            view.aircraft_position_info.replace(ap.clone());
                            renderer.set_aircraft_position(ap);
                            view.gl_area.queue_draw();
                        }
                    }
                }
            }));

            // Get the aircraft position every 5 seconds
            let recurring_handle = scheduling::Scheduler::delayed_recurring(
                std::time::Duration::from_secs(2),
                std::time::Duration::from_secs(5),
                move || get_aircraft_position_task(tx.clone()),
            )
                .start();
            self.scheduler_handle_ap.replace(Some(recurring_handle));

            // Set up the scheduled tasks to query the subsolar point
            let (tx, rx) = async_channel::unbounded::<(f64, f64)>();
            MainContext::default().spawn_local(clone!(#[weak(rename_to = view)] self, async move {
                while let Ok(ap) = rx.recv().await {
                    if let Some(renderer) = view.renderer.borrow().as_ref() {
                        view.sub_solar_point.replace(ap.clone());
                        renderer.set_sub_solar_point(ap);
                        view.gl_area.queue_draw();
                    }
                }
            }));

            // Get the subsolar point every 4 minutes, which is equivalent to a one degree shift
            let recurring_handle = scheduling::Scheduler::delayed_recurring(
                std::time::Duration::from_secs(0),
                std::time::Duration::from_secs(240),
                move || get_subsolar_point(tx.clone()),
            )
                .start();
            self.scheduler_handle_ssp.replace(Some(recurring_handle));
        }

        pub fn center_map(&self, point: Coordinate) {
            if let Some(renderer) = self.renderer.borrow().as_ref() {
                renderer.set_map_centre(point, false);
                self.gl_area.queue_draw();
            }
        }

        pub fn get_center_map(&self) -> Option<Coordinate> {
            self.renderer.borrow().as_ref(). map(|renderer| {
                renderer.get_map_centre()
            })
        }

        pub fn set_plan(&self, plan: Rc<RefCell<Plan>>) {
            if let Some(renderer) = self.renderer.borrow().as_ref() {
                renderer.set_plan(plan);
                self.gl_area.queue_draw();
            }
        }

        fn unproject(&self, x: f64, y: f64) -> Result<Coordinate, String> {
            match self.renderer.borrow().as_ref().unwrap()
                .get_cord_from_win(&self.gl_area, x as f32, y as f32, self.zoom_level.get()) {
                Ok(point) => {
                    Ok(Coordinate::new(point[0] as f64, point[1] as f64))
                }
                Err(x) => Err(x)
            }
        }

        fn zoom(&self, z_factor: f32) {
            let mut zoom = self.zoom_level.get() * z_factor;
            zoom = zoom.max(1.0);
            zoom = zoom.min(MAX_ZOOM);

            self.zoom_level.replace(zoom);
            self.renderer.borrow().as_ref().unwrap().set_zoom_level(zoom);
            self.gl_area.queue_draw();
        }

        fn find_airport_for_point(&self, pos: Coordinate) -> Option<Arc<Airport>> {
            let zoom = self.zoom_level.get();
            let range = 2.0 / zoom;

            // Collect any visible airport in a 2-degree radius and then sort them, returning the nearest
            if self.btn_show_airports.is_active() {
                let rwl = if zoom > 8.0 {
                    0
                } else if zoom > 3.0 {
                    5000
                } else {
                    10000
                };
                let inc_heli = zoom > 6.0;

                let airports = earth::get_earth_model().get_airports().read().unwrap();
                let airport = airports.iter().filter(|a| {
                    (f64::abs(pos.get_latitude() - a.get_lat()) < range as f64)
                        && (f64::abs(pos.get_longitude() - a.get_long()) < range as f64)
                        && (a.get_max_runway_length() > rwl || (inc_heli && a.get_type().unwrap() == AirportType::Heliport))
                })
                    .min_by(|a, b| {
                        a.get_loc().distance_to(&pos)
                            .partial_cmp(&b.get_loc().distance_to(&pos))
                            .unwrap_or(Equal)
                    });
                if let Some(airport) = airport {
                    return Some(airport.clone());
                }
            }

            None
        }
        fn find_navaid_for_point(&self, pos: Coordinate) -> Option<Arc<Navaid>> {
            let zoom = self.zoom_level.get();
            let range = 2.0 / zoom;

            // Collect any visible airport in a 2-degree radius and then sort them, returning the nearest
            if self.btn_show_navaids.is_active() {
                let inc_ndb = zoom > 6.0;

                let navaids = earth::get_earth_model().get_navaids().read().unwrap();
                let navaid = navaids.iter().filter(|a| {
                    (f64::abs(pos.get_latitude() - a.get_lat()) < range as f64)
                        && (f64::abs(pos.get_longitude() - a.get_long()) < range as f64)
                        && (inc_ndb || a.get_type() == NavaidType::Vor)
                })
                    .min_by(|a, b| {
                        a.get_loc().distance_to(&pos)
                            .partial_cmp(&b.get_loc().distance_to(&pos))
                            .unwrap_or(Equal)
                    });
                if let Some(navaid) = navaid {
                    return Some(navaid.clone());
                }
            }

            None
        }

        fn make_airport_popup(&self, model: &Menu, airport: Option<Arc<Airport>>) {
            if let Some(airport) = airport {
                let label = gettext("_View {} layout").replace("{}", airport.get_name());
                let item = MenuItem::new(Some(&*label), Some("world_map.view_airport"));
                model.append_item(&item);
                let label = gettext("Add {} to _plan").replace("{}", airport.get_name());
                let item = MenuItem::new(Some(&*label), Some("world_map.add_to_plan"));
                model.append_item(&item);
            }
        }

        fn make_navaid_popup(&self, model: &Menu, navaid: Option<Arc<Navaid>>) {
            if let Some(navaid) = navaid {
                let label = gettext("Add {} to _plan").replace("{}", navaid.get_name());
                let item = MenuItem::new(Some(&*label), Some("world_map.add_nav_to_plan"));
                model.append_item(&item);
            }
        }

        fn make_popup(&self, airport: Option<Arc<Airport>>, navaid: Option<Arc<Navaid>>) -> PopoverMenu {
            let model = Menu::new();
            self.make_airport_popup(&model, airport);
            self.make_navaid_popup(&model, navaid);

            let item = MenuItem::new(Some(&*gettext("Find _airports near")), Some("world_map.find_airports_near"));
            model.append_item(&item);
            let item = MenuItem::new(Some(&*gettext("Find _navaids near")), Some("world_map.find_navaids_near"));
            model.append_item(&item);
            let item = MenuItem::new(Some(&*gettext("Find _fixes near")), Some("world_map.find_fixes_near"));
            model.append_item(&item);

            let popover = PopoverMenu::builder()
                .menu_model(&model)
                .has_arrow(false)
                .build();
            popover.set_parent(&self.map_window.get());
            popover
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WorldMapView {
        const NAME: &'static str = "WorldMapView";
        type Type = super::WorldMapView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WorldMapView {
        fn constructed(&self) {
            self.parent_constructed();
            self.initialise();

            self.gl_area.set_has_depth_buffer(true);
            self.gl_area.set_has_tooltip(true);

            self.gl_area.connect_realize(clone!(#[weak(rename_to = window)] self, move |area| {
                if let Some(context) = area.context() {
                    context.make_current();
                    if let Some(error) = area.error() {
                        error!("{:?}", error);
                    }

                    let renderer = Renderer::new();
                    renderer.set_zoom_level(1.0);
                    renderer.set_aircraft_position(None);

                    let pref = crate::preference::manager();

                    if let Some(long) = pref.get::<f64>("map-centre-long") {
                        if let Some(lat) = pref.get::<f64>("map-centre-lat") {
                            renderer.set_map_centre(Coordinate::new(lat, long), true);
                        }
                    }

                    window.renderer.replace(Some(renderer));
                }
            }));

            self.gl_area.connect_unrealize(clone!(#[weak(rename_to = window)] self, move |area| {
                let pref = crate::preference::manager();

                if let Some(renderer) = window.renderer.borrow().as_ref(){
                    let centre = renderer.get_map_centre();
                    pref.put("map-centre-long", centre.get_longitude());
                    pref.put("map-centre-lat", centre.get_latitude());
                }

                if let Some(context) = area.context() {
                    context.make_current();
                    window.renderer.borrow().as_ref().unwrap().drop_buffers();
                }
            }));

            self.gl_area.connect_render(clone!(#[weak(rename_to = window)] self, #[upgrade_or] Propagation::Proceed, move |area, _context| {
                let airports = window.btn_show_airports.is_active();
                let navaids = window.btn_show_navaids.is_active();
                window.renderer.borrow().as_ref().unwrap().draw(area, airports, navaids);
                Propagation::Proceed
            }));

            // Use ctl-Scroll to zoom
            let gesture = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
            gesture.connect_scroll(clone!(#[weak(rename_to = window)] self, #[upgrade_or] Propagation::Proceed, move |gesture, _dx, dy| {
                let parent = gesture.upcast_ref::<gtk::EventController>();
                if let Some(ev) = parent.current_event() {
                    if ev.modifier_state().contains(ModifierType::CONTROL_MASK) {
                        if dy > 0.0 {
                            window.zoom(0.95);
                        } else {
                            window.zoom(1.0 / 0.95);
                        }
                        Propagation::Stop
                    } else {
                        Propagation::Proceed
                    }
                } else {
                    Propagation::Proceed
                }
            }));
            self.gl_area.add_controller(gesture);

            // Set double click to centre map
            let gesture = gtk::GestureClick::new();
            gesture.set_button(1);
            gesture.connect_released(clone!(#[weak(rename_to = view)] self, move | gesture, _n, x, y| {
                if _n == 2 {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    if let Ok(point) = view.unproject(x, y) {
                        view.renderer.borrow().as_ref().unwrap().set_map_centre(point, false);
                        view.gl_area.queue_draw();
                    }
                }
            }));
            self.gl_area.add_controller(gesture);

            // Connect popup menu to right click
            let gesture = gtk::GestureClick::new();
            gesture.set_button(3);
            gesture.connect_released(clone!(#[weak(rename_to = view)] self, move |gesture, _n, x, y| {
                if let Some(point) = view.map_window.compute_point(&view.gl_area.get(), &Point::new(x as f32, y as f32)) {
                    let airport = match view.unproject(point.x() as f64, point.y() as f64) {
                        Ok(pos) => {
                            view.find_airport_for_point(pos)
                        }
                        Err(_) => {
                            None
                        }
                    };
                    let navaid = match view.unproject(point.x() as f64, point.y() as f64) {
                        Ok(pos) => {
                            view.find_navaid_for_point(pos)
                        }
                        Err(_) => {
                            None
                        }
                    };
                    let popover = view.make_popup(airport, navaid);

                    gesture.set_state(gtk::EventSequenceState::Claimed);

                    if let Some(old_popover) = view.popover.replace(Some(popover)) {
                        old_popover.unparent();
                    }

                    if let Some(popover) = view.popover.borrow().as_ref() {
                        popover.set_pointing_to(Some(&Rectangle::new(x as i32, y as i32, 1, 1)));
                        popover.popup();
                    };
                };
            }));
            self.map_window.add_controller(gesture);

            // Set Gesture to drag the map about
            let gesture = gtk::GestureDrag::new();
            gesture.set_button(1);
            gesture.connect_drag_begin(clone!(#[weak(rename_to = view)] self, move | _gesture, x, y| {
                view.drag_start.replace(Some([x, y]));
                view.drag_last.replace(Some([x, y]));
            }));
            gesture.connect_drag_update(clone!(#[weak(rename_to = view)] self, move | _gesture, x, y| {
                if let Some(map_drag_start) = view.drag_start.borrow().as_ref() {
                    let x_start = map_drag_start[0];
                    let y_start = map_drag_start[1];
                        if let Some(map_drag_last) = view.drag_last.borrow().as_ref() {
                            let old_lat_long = match view.unproject(map_drag_last[0], map_drag_last[1]) {
                                Ok(old_pos) => {
                                    old_pos
                                }
                                Err(_) => {
                                    // Not in map, we don't care
                                    return;
                                }
                            };

                            let lat_long= match view.unproject(x_start + x, y_start + y) {
                                Ok(pos) => {
                                    pos
                                }
                                Err(_) => {
                                    // Not in map, we don't care
                                    return;
                                }
                            };

                            let old_centre = view.renderer.borrow().as_ref().unwrap().get_map_centre();

                            let mut new_lat = old_centre.get_latitude() + (old_lat_long.get_latitude() - lat_long.get_latitude());
                            if new_lat < -90.0 {
                                new_lat = -180.0 - new_lat;
                            }
                            if new_lat > 90.0 {
                                new_lat = 180.0 - new_lat;
                            }

                            let mut new_long = old_centre.get_longitude() + (old_lat_long.get_longitude() - lat_long.get_longitude());
                            if new_long < -180.0 {
                                new_long += 360.0;
                            }
                            if new_long > 180.0 {
                                new_long -= 360.0;
                            }

                            view.renderer.borrow().as_ref().unwrap().set_map_centre(Coordinate::new(new_lat, new_long), true);
                            view.gl_area.queue_draw();
                        };
                    view.drag_last.replace(Some([x_start + x, y_start + y]));
                };
            }));
            gesture.connect_drag_end(clone!(#[weak(rename_to = view)] self, move | _gesture, _x, _y| {
                view.drag_start.replace(None);
            }));
            self.gl_area.add_controller(gesture);

            self.gl_area.connect_query_tooltip(clone!(#[weak(rename_to = view)] self, #[upgrade_or] false, move | _glarea, x, y, _kbm, tooltip | {
                match view.unproject(x as f64,y as f64) {
                    Ok(pos) => {
                        if let Some(airport) = view.find_airport_for_point(pos) {
                            tooltip.set_text(Some(airport.get_name()));
                            true

                        } else {
                            tooltip.set_text(None);
                            false
                        }
                    }
                    Err(_) => {
                        tooltip.set_text(None);
                        false
                    }
                }
            }));

            self.btn_show_airports.connect_clicked(clone!(#[weak(rename_to = view)] self, move |_| {
                    view.gl_area.queue_draw();
                }));

            self.btn_show_navaids.connect_clicked(clone!(#[weak(rename_to = view)] self, move |_| {
                    view.gl_area.queue_draw();
                }));

            self.btn_zoom_in.connect_clicked(clone!(#[weak(rename_to = view)] self, move |_| {
                    let z_factor = 1.0 / 0.75;
                    view.zoom(z_factor);
                }));

            self.btn_zoom_out.connect_clicked(clone!(#[weak(rename_to = view)] self, move |_| {
                    let z_factor = 0.75;
                    view.zoom(z_factor);
                }));

            let actions = SimpleActionGroup::new();
            self.map_window
                .get()
                .insert_action_group("world_map", Some(&actions));

            let action = SimpleAction::new("view_airport", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                let win_pos = view.popover.borrow().as_ref().unwrap().pointing_to();
                if let Ok(loc) = view.unproject(win_pos.1.x() as f64, win_pos.1.y() as f64) {
                    if let Some(airport) = view.find_airport_for_point(loc) {
                        if let Some(airport_map_view) = get_airport_map_view(&view.map_window.get()) {
                            show_airport_map_view(&view.map_window.get());
                            airport_map_view.imp().set_airport(airport);
                        }
                    }
                }
            }));
            actions.add_action(&action);
            self.view_action.replace(Some(action));

            let action = SimpleAction::new("add_to_plan", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                let win_pos = view.popover.borrow().as_ref().unwrap().pointing_to();
                if let Ok(loc) = view.unproject(win_pos.1.x() as f64, win_pos.1.y() as f64) {
                    if let Some(airport) = view.find_airport_for_point(loc) {
                        if let Some(ref mut plan_view) = get_plan_view(&view.map_window.get()) {
                            // get the plan
                            plan_view.imp().add_airport_to_plan(airport);
                        }
                    }
                }
            }));
            actions.add_action(&action);
            self.add_action.replace(Some(action));

            let action = SimpleAction::new("add_nav_to_plan", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                let win_pos = view.popover.borrow().as_ref().unwrap().pointing_to();
                if let Ok(loc) = view.unproject(win_pos.1.x() as f64, win_pos.1.y() as f64) {
                    if let Some(navaid) = view.find_navaid_for_point(loc) {
                        if let Some(ref mut plan_view) = get_plan_view(&view.map_window.get()) {
                            // get the plan
                            plan_view.imp().add_waypoint_to_plan(Waypoint::Navaid {navaid: navaid.clone(), elevation: Cell::new(0), locked: true,});
                        }
                    }
                }
            }));
            actions.add_action(&action);
            self.add_nav_action.replace(Some(action));

            let action = SimpleAction::new("find_airports_near", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                let win_pos = view.popover.borrow().as_ref().unwrap().pointing_to();
                if let Ok(loc) = view.unproject(win_pos.1.x() as f64, win_pos.1.y() as f64) {
                    if let Some(airport_view) = get_airport_view(&view.map_window.get()) {
                        show_airport_view(&view.map_window.get());
                        airport_view.imp().search_near(&loc);
                    }
                }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_navaids_near", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                let win_pos = view.popover.borrow().as_ref().unwrap().pointing_to();
                if let Ok(loc) = view.unproject(win_pos.1.x() as f64, win_pos.1.y() as f64) {
                    if let Some(navaid_view) = get_navaid_view(&view.map_window.get()) {
                        show_navaid_view(&view.map_window.get());
                        navaid_view.imp().search_near(&loc);
                    }
                }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_fixes_near", None);
            action.connect_activate(clone!(#[weak(rename_to = view)] self, move |_action, _parameter| {
                let win_pos = view.popover.borrow().as_ref().unwrap().pointing_to();
                if let Ok(loc) = view.unproject(win_pos.1.x() as f64, win_pos.1.y() as f64) {
                    if let Some(fix_view) = get_fix_view(&view.map_window.get()) {
                        show_fix_view(&view.map_window.get());
                        fix_view.imp().search_near(&loc);
                    }
                }
            }));
            actions.add_action(&action);
        }

        fn dispose(&self) {
            if let Some(popover) = self.popover.borrow().as_ref() {
                popover.unparent();
            };

            if let Some(scheduler) = self.scheduler_handle_ap.borrow_mut().deref() {
                scheduler.cancel();
            }

            if let Some(scheduler) = self.scheduler_handle_ssp.borrow_mut().deref() {
                scheduler.cancel();
            }
        }
    }

    impl WidgetImpl for WorldMapView {}

    impl BoxImpl for WorldMapView {}
}

glib::wrapper! {
    pub struct WorldMapView(ObjectSubclass<imp::WorldMapView>)
        @extends gtk::Widget, gtk::Box, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl WorldMapView {
    pub fn new() -> Self {
        glib::Object::new::<WorldMapView>()
    }
}

impl Default for WorldMapView {
    fn default() -> Self {
        Self::new()
    }
}

fn get_aircraft_position_task(tx: Sender<Option<AircraftPositionInfo>>) {
    // Your task implementation goes here
    let ap = get_aircraft_position();
    let _ = tx.try_send(ap);
}

fn get_subsolar_point(tx: Sender<(f64, f64)>) {
    // Your task implementation goes here
    let ap = subsolar_point(Utc::now());
    let _ = tx.try_send(ap);
}
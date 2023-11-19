/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use gtk::{self, Adjustment, CompositeTemplate, glib};
use gtk::prelude::AdjustmentExt;

mod imp {
    use std::cell::{Cell, RefCell};
    use std::cmp::Ordering::Equal;
    use std::rc::Rc;
    use std::sync::Arc;

    use gtk::{Button, GLArea, glib, Inhibit, PopoverMenu, ScrolledWindow, ToggleButton};
    use gtk::gdk::Rectangle;
    use gtk::gio::{Menu, MenuItem, SimpleAction, SimpleActionGroup};
    use gtk::glib::clone;
    use gtk::glib::subclass::InitializingObject;
    use gtk::graphene::Point;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use log::error;

    use crate::earth;
    use crate::earth::coordinate::Coordinate;
    use crate::model::airport::{Airport, AirportType};
    use crate::model::location::Location;
    use crate::model::plan::Plan;
    use crate::window::render_gl::Renderer;
    use crate::window::util::{get_airport_map_view, get_airport_view, get_fix_view, get_navaid_view, get_plan_view, show_airport_map_view, show_airport_view, show_fix_view, show_navaid_view};

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

        renderer: RefCell<Option<Renderer>>,
        drag_start: RefCell<Option<[f64; 2]>>,
        drag_last: RefCell<Option<[f64; 2]>>,
        zoom_level: Cell<f32>,

    }

    const MAX_ZOOM: f32 = 20.;

    impl WorldMapView {
        pub fn initialise(&self) -> () {
            self.zoom_level.replace(1.0);
        }

        pub fn airports_loaded(&self) {
            if let Some(renderer) = self.renderer.borrow().as_ref() {
                renderer.airports_loaded();
            };
            self.gl_area.queue_draw();
        }
        pub fn navaids_loaded(&self) {
            if let Some(renderer) = self.renderer.borrow().as_ref() {
                renderer.navaids_loaded();
            }
            self.gl_area.queue_draw();
        }

        pub fn center_map(&self, point: Coordinate) {
            if let Some(renderer) = self.renderer.borrow().as_ref() {
                renderer.set_map_centre(point, false);
                center_scrollbar(&self.map_window.hadjustment());
                center_scrollbar(&self.map_window.vadjustment());
                self.gl_area.queue_draw();
            }
        }

        pub fn set_plan(&self, plan: Rc<RefCell<Plan>>) {
            if let Some(renderer) = self.renderer.borrow().as_ref() {
                renderer.set_plan(plan);
                self.gl_area.queue_draw();
            }
        }

        fn unproject(&self, x: f64, y: f64) -> Result<Coordinate, String> {
            match self.renderer.borrow().as_ref().unwrap().get_cord_from_win(&self.gl_area, x as f32, y as f32) {
                Ok(point) => {
                    Ok(Coordinate::new(point[0] as f64, point[1] as f64))
                }
                Err(x) => Err(x)
            }
        }

        fn zoom(&self, z_factor: f32) {
            let zoom = self.zoom_level.get() * z_factor;
            if zoom < MAX_ZOOM && zoom > 1. {
                self.zoom_level.replace(zoom);
                self.renderer.borrow().as_ref().unwrap().set_zoom_level(zoom);

                // Save the old scrollbar height & Width
                let old_ha_upper = self.map_window.hadjustment().upper();
                let old_va_upper = self.map_window.vadjustment().upper();
                let old_ha_value = self.map_window.hadjustment().value();
                let old_va_value = self.map_window.vadjustment().value();

                let h = self.gl_area.height();
                let w = self.gl_area.width();
                self.gl_area.set_width_request((w as f32 * z_factor) as i32);
                self.gl_area.set_height_request((h as f32 * z_factor) as i32);
                self.gl_area.queue_draw();

                // adjust scroll position
                self.map_window.hadjustment().set_upper(old_ha_upper * z_factor as f64);
                self.map_window.vadjustment().set_upper(old_va_upper * z_factor as f64);
                let ha_upper = self.map_window.hadjustment().upper();
                let va_upper = self.map_window.vadjustment().upper();
                let ha_value = (old_ha_value + (ha_upper - old_ha_upper) / 2.0).max(0.0);
                let va_value = (old_va_value + (va_upper - old_va_upper) / 2.0).max(0.0);
                self.map_window.hadjustment().set_value(ha_value);
                self.map_window.vadjustment().set_value(va_value);
            }
        }

        fn find_location_for_point(&self, pos: Coordinate) -> Option<Arc<Airport>> {
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
                        && (a.get_max_runway_length() > rwl || (inc_heli && a.get_type().unwrap() != AirportType::HELIPORT))
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

        fn make_popup(&self, airport: Option<Arc<Airport>>) -> PopoverMenu {
            let model = Menu::new();
            if let Some(airport) = airport {
                let item = MenuItem::new(Some(format!("View {} layout", airport.get_name()).as_str()), Some("world_map.view_airport"));
                model.append_item(&item);
                let item = MenuItem::new(Some(format!("Add {} to plan", airport.get_name()).as_str()), Some("world_map.add_to_plan"));
                model.append_item(&item);
            }
            let item = MenuItem::new(Some("Find airports near"), Some("world_map.find_airports_near"));
            model.append_item(&item);
            let item = MenuItem::new(Some("Find navaids near"), Some("world_map.find_navaids_near"));
            model.append_item(&item);
            let item = MenuItem::new(Some("Find fixes near"), Some("world_map.find_fixes_near"));
            model.append_item(&item);

            let popover = PopoverMenu::builder()
                .menu_model(&model)
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


            self.gl_area.connect_realize(clone!(@weak self as window => move |area| {
                if let Some(context) = area.context() {
                    context.make_current();
                    if let Some(error) = area.error() {
                        error!("{:?}", error);
                    }

                    let renderer = Renderer::new();
                    renderer.set_zoom_level(1.0);
                    window.renderer.replace(Some(renderer));
                }
            }));

            self.gl_area.connect_unrealize(clone!(@weak self as window => move |area| {
                if let Some(context) = area.context() {
                    context.make_current();
                    window.renderer.borrow().as_ref().unwrap().drop_buffers();
                }
            }));

            self.gl_area.connect_render(clone!(@weak self as window => @default-return Inhibit(false), move |area, _context| {
                let airports = window.btn_show_airports.is_active();
                let navaids = window.btn_show_navaids.is_active();
                window.renderer.borrow().as_ref().unwrap().draw(area, airports, navaids);
                Inhibit{ 0: false }
            }));

            // Set double click to centre map
            let gesture = gtk::GestureClick::new();
            gesture.set_button(1);
            gesture.connect_released(clone!(@weak self as view => move | gesture, _n, x, y| {
                if _n == 2 {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    if let Ok(point) = view.unproject(x, y) {
                        view.renderer.borrow().as_ref().unwrap().set_map_centre(point, false);
                        center_scrollbar(&view.map_window.hadjustment());
                        center_scrollbar(&view.map_window.vadjustment());
                        view.gl_area.queue_draw();
                    }
                }
            }));
            self.gl_area.add_controller(gesture);

            // Connect popup menu to right click
            let gesture = gtk::GestureClick::new();
            gesture.set_button(3);
            gesture.connect_released(clone!(@weak self as view => move |gesture, _n, x, y| {
                if let Some(point) = view.map_window.compute_point(&view.gl_area.get(), &Point::new(x as f32, y as f32)) {
                    let airport = match view.unproject(point.x() as f64, point.y() as f64) {
                        Ok(pos) => {
                            view.find_location_for_point(pos)
                        }
                        Err(_) => {
                            None
                        }
                    };
                    let popover = view.make_popup(airport);

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
            gesture.connect_drag_begin(clone!(@weak self as view => move | _gesture, x, y| {
                view.drag_start.replace(Some([x, y]));
                view.drag_last.replace(Some([x, y]));
            }));
            gesture.connect_drag_update(clone!(@weak self as view => move | _gesture, x, y| {
                if let Some(map_drag_start) = view.drag_start.borrow().as_ref() {
                    let x_start = map_drag_start[0];
                    let y_start = map_drag_start[1];
                        if let Some(map_drag_last) = view.drag_last.borrow().as_ref() {
                            let old_lat_long: Coordinate;
                            match view.unproject(map_drag_last[0], map_drag_last[1]) {
                                Ok(old_pos) => {
                                    old_lat_long = old_pos;
                                }
                                Err(_) => {
                                    // Not in map, we don't care
                                    return;
                                }
                            };

                            let lat_long: Coordinate;
                            match view.unproject(x_start + x, y_start + y) {
                                Ok(pos) => {
                                    lat_long = pos;
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
            gesture.connect_drag_end(clone!(@weak self as view => move | _gesture, _x, _y| {
                view.drag_start.replace(None);
            }));
            self.gl_area.add_controller(gesture);

            self.gl_area.connect_query_tooltip(clone!(@weak self as view => @default-return false, move | _glarea, x, y, _kbm, tooltip | {
                match view.unproject(x as f64,y as f64) {
                    Ok(pos) => {
                        if let Some(airport) = view.find_location_for_point(pos) {
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

            self.btn_show_airports.connect_clicked(clone!(@weak self as view => move |_| {
                    view.gl_area.queue_draw();
                }));

            self.btn_show_navaids.connect_clicked(clone!(@weak self as view => move |_| {
                    view.gl_area.queue_draw();
                }));

            self.btn_zoom_in.connect_clicked(clone!(@weak self as view => move |_| {
                    let z_factor = 1.0 / 0.75;
                    view.zoom(z_factor);
                }));

            self.btn_zoom_out.connect_clicked(clone!(@weak self as view => move |_| {
                    let z_factor = 0.75;
                    view.zoom(z_factor);
                }));

            let actions = SimpleActionGroup::new();
            self.map_window
                .get()
                .insert_action_group("world_map", Some(&actions));

            let action = SimpleAction::new("view_airport", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                let win_pos = view.popover.borrow().as_ref().unwrap().pointing_to();
                if let Some(point) = view.map_window.compute_point(&view.gl_area.get(), &Point::new(win_pos.1.x() as f32, win_pos.1.y() as f32)) {
                    if let Ok(loc) = view.unproject(point.x() as f64, point.y() as f64) {
                        if let Some(airport) = view.find_location_for_point(loc) {
                            match get_airport_map_view(&view.map_window.get()) {
                                Some(airport_map_view) => {
                                    airport_map_view.imp().set_airport(airport);
                                    show_airport_map_view(&view.map_window.get());
                                },
                                None => (),
                            }
                        }
                    }
                }
            }));
            actions.add_action(&action);
            self.view_action.replace(Some(action));

            let action = SimpleAction::new("add_to_plan", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                let win_pos = view.popover.borrow().as_ref().unwrap().pointing_to();
                if let Some(point) = view.map_window.compute_point(&view.gl_area.get(), &Point::new(win_pos.1.x() as f32, win_pos.1.y() as f32)) {
                    if let Ok(loc) = view.unproject(point.x() as f64, point.y() as f64) {
                        if let Some(airport) = view.find_location_for_point(loc) {
                            match get_plan_view(&view.map_window.get()) {
                            Some(ref mut plan_view) => {
                                // get the plan
                                plan_view.imp().add_airport_to_plan(airport);
                                ()
                            }
                            None => (),
                            }
                        }
                    }
                }
            }));
            actions.add_action(&action);
            self.add_action.replace(Some(action));

            let action = SimpleAction::new("find_airports_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                let win_pos = view.popover.borrow().as_ref().unwrap().pointing_to();
                if let Some(point) = view.map_window.compute_point(&view.gl_area.get(), &Point::new(win_pos.1.x() as f32, win_pos.1.y() as f32)) {
                    if let Ok(loc) = view.unproject(point.x() as f64, point.y() as f64) {
                        match get_airport_view(&view.map_window.get()) {
                            Some(airport_view) => {
                                show_airport_view(&view.map_window.get());
                                airport_view.imp().search_near(&loc);
                                ()
                            },
                            None => (),
                        }
                    }
                }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_navaids_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                let win_pos = view.popover.borrow().as_ref().unwrap().pointing_to();
                if let Some(point) = view.map_window.compute_point(&view.gl_area.get(), &Point::new(win_pos.1.x() as f32, win_pos.1.y() as f32)) {
                    if let Ok(loc) = view.unproject(point.x() as f64, point.y() as f64) {
                        match get_navaid_view(&view.map_window.get()) {
                            Some(navaid_view) => {
                                show_navaid_view(&view.map_window.get());
                                navaid_view.imp().search_near(&loc);
                                ()
                            },
                            None => (),
                        }
                    }
                }
            }));
            actions.add_action(&action);

            let action = SimpleAction::new("find_fixes_near", None);
            action.connect_activate(clone!(@weak self as view => move |_action, _parameter| {
                let win_pos = view.popover.borrow().as_ref().unwrap().pointing_to();
                if let Some(point) = view.map_window.compute_point(&view.gl_area.get(), &Point::new(win_pos.1.x() as f32, win_pos.1.y() as f32)) {
                    if let Ok(loc) = view.unproject(point.x() as f64, point.y() as f64) {
                        match get_fix_view(&view.map_window.get()) {
                            Some(fix_view) => {
                                show_fix_view(&view.map_window.get());
                                fix_view.imp().search_near(&loc);
                                ()
                            },
                            None => (),
                        }
                    }
                }
            }));
            actions.add_action(&action);
        }

        fn dispose(&self) {
            if let Some(popover) = self.popover.borrow().as_ref() {
                popover.unparent();
            };
        }
    }

    impl WidgetImpl for WorldMapView {}

    impl BoxImpl for WorldMapView {}
}

glib::wrapper! {
    pub struct WorldMapView(ObjectSubclass<imp::WorldMapView>) @extends gtk::Widget, gtk::Box;
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

fn center_scrollbar(adjustment: &Adjustment) {
    let page_size = adjustment.page_size();
    let upper = adjustment.upper();
    let value = (upper - page_size) / 2.0;
    adjustment.set_value(value);
}
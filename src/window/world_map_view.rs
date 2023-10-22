/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use gtk::{self, CompositeTemplate, glib, subclass::prelude::*};
use gtk::prelude::{GLAreaExt, WidgetExt};

mod imp {
    use std::cell::RefCell;

    use gtk::{GLArea, glib, Inhibit};
    use gtk::glib::clone;
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use crate::earth::coordinate::Coordinate;

    use crate::window::render_gl::Renderer;

    use super::*;


    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_planner/world_map_view.ui")]
    pub struct WorldMapView {
        #[template_child]
        gl_area: TemplateChild<GLArea>,

        renderer: RefCell<Option<Renderer>>,
    }

    impl WorldMapView {
        pub fn initialise(&self) -> () {}

        fn init_buffer_objects(&self) -> Option<glib::Error> {
            None
        }

        fn unproject(&self, x: f64, y: f64) -> Coordinate {
            let width = self.gl_area.width();
            let height = self.gl_area.height();

            let aspect_ratio = if height < width {
                [height as f64 / width as f64, 1.0]
            } else {
                [1.0, width as f64 / height as f64]
            };
            let centre = self.renderer.borrow().as_ref().unwrap().get_map_centre();
            // x is Longitude, so do that first
            let sphere_width = width as f64 * aspect_ratio[0];
            let sphere_height = height as f64 * aspect_ratio[1];
            let long_shift = ((x - width as f64/2.) / (sphere_width /2.)).asin().to_degrees();
            let lat_shift = ((y - height as f64/2.) / (sphere_height/2.)).asin().to_degrees();
            Coordinate::new(centre.get_latitude() + lat_shift, centre.get_longitude() + long_shift)
        }



    }

    #[glib::object_subclass]
    impl ObjectSubclass for WorldMapView {
        const NAME: &'static str = "WorldMapView";
        type Type = super::WorldMapView;
        type ParentType = gtk::Widget;

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

            self.gl_area.connect_realize(clone!(@weak self as window => move |area| unsafe {
                if let Some(context) = area.context() {
                    context.make_current();
                    *window.renderer.borrow_mut() = Some(Renderer::new());
                }
            }));

            self.gl_area.connect_unrealize(clone!(@weak self as window => move |area| unsafe {
                if let Some(context) = area.context() {
                    context.make_current();
                    window.renderer.borrow().as_ref().unwrap().drop_buffers();
                }
            }));

            self.gl_area.connect_render(clone!(@weak self as window => @default-return Inhibit(false), move |area, context| unsafe {
                window.renderer.borrow().as_ref().unwrap().draw(area);
                Inhibit{ 0: false }
            }));

            // Set double click to centre map
            let gesture = gtk::GestureClick::new();
            gesture.set_button(1);
            gesture.connect_released(clone!(@weak self as view => move |gesture, _n, x, y| {
                println!("Num clicks{}", _n);
                if _n == 2 {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    let point = view.unproject(x, y);
                    view.renderer.borrow().as_ref().unwrap().set_map_centre(point);
                    view.gl_area.queue_render();
                }
            }));
            self.gl_area.add_controller(gesture);
        }
    }

    impl WidgetImpl for WorldMapView {}

    impl GLAreaImpl for WorldMapView {}
}

glib::wrapper! {
    pub struct WorldMapView(ObjectSubclass<imp::WorldMapView>) @extends gtk::Widget;
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
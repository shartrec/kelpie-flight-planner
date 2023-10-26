use glib::Object;
use gtk::{Application, gio, glib};
use gtk::prelude::GtkWindowExt;

mod airport_map_view;
mod airport_view;
mod fix_view;
pub(crate) mod imp;
mod navaid_view;
mod plan_view;
mod util;
mod world_map_view;
pub(crate) mod map_utils;
pub(crate) mod render_gl;
mod sphere_renderer;
mod shoreline_renderer;
mod airport_renderer;
mod navaid_renderer;
mod plan_renderer;

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

    fn setup_actions(&self) {}
    pub fn save_window_size(&self) -> Result<(), glib::BoolError> {
        // Get the size of the window
        let size = self.default_size();

        // Set the window state in `settings`
        let pref = crate::preference::manager();
        pref.put("window-width", size.0);
        pref.put("window-height", size.1);
        pref.put("window-is-maximized", self.is_maximized());

        Ok(())
    }

    fn load_window_size(&self) {
        // Get the window state from `settings`
        let pref = crate::preference::manager();

        // Set the size of the window
        if let Some(w) = pref.get::<i32>("window-width") {
            if let Some(h) = pref.get::<i32>("window-height") {
                self.set_default_size(w, h);
            }
        }

        // If the window was maximized when it was closed, maximize it again
        if let Some(is_maximised) = pref.get::<bool>("is-maximized") {
            if is_maximised {
                self.maximize();
            }
        }
    }

}

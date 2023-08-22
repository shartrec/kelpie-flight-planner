mod earth;
mod events;
mod model;
mod preference;
mod util;
mod window;

use simplelog::*;
use std::thread;

use gtk::prelude::*;
use gtk::{gio, glib, Application};
use window::Window;

const APP_ID: &str = "com.shartrec.KelpiePlanner";

fn main() -> glib::ExitCode {
    init_logger();

    // Register and include resources
    gio::resources_register_include!("kelpie_planner.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    app.connect_startup(|_app| {
        thread::spawn(|| {
            crate::earth::initialise();
        });
    });

    // Run the application
    app.run()
}

fn init_logger() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            std::fs::File::create("kelpie-planner.log").unwrap(),
        ),
    ]).expect("Unable to initiate logger.");
}

fn build_ui(app: &Application) {
    // Create new window and present it
    let window = Window::new(app);
    window.present();
}

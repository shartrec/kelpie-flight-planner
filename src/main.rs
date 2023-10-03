use gtk::{Application, CssProvider, gio, glib};
use gtk::gdk::Display;
use gtk::gio::{File, SimpleAction};
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use simplelog::*;

use window::Window;

mod earth;
mod event;
mod hangar;
mod model;
mod preference;
mod planner;
mod util;
mod window;

const APP_ID: &str = "com.shartrec.KelpiePlanner";


fn main() -> glib::ExitCode {
    init_logger();

    // Register and include resources
    gio::resources_register_include!("kelpie_planner.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gtk::gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_startup(|app| {
        load_css();
    });

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    app.connect_open(build_and_open);

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

fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_resource("/com/shartrec/kelpie_planner/style.css");

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn connect_actions(app: &Application, window: &Window) {

    let action = SimpleAction::new("new", None);
    action.connect_activate(clone!(@weak window => move |action, parameter| {
       let _ = &window.imp().new_plan();
    }));
    app.add_action(&action);

    let action = SimpleAction::new("open", None);
    action.connect_activate(move |action, parameter| {
        todo!("Open clicked");
    });
    app.add_action(&action);

    let action = SimpleAction::new("save", None);
    action.connect_activate(move |action, parameter| {
        todo!("Save clicked");
    });
    app.add_action(&action);

    let action = SimpleAction::new("quit", None);
    action.connect_activate(clone!(@weak app => move |action, parameter| {
        app.quit()
    }));
    app.add_action(&action);
}

fn build_ui(app: &Application) {
    // Create new window and present it
    let window = Window::new(app);
    connect_actions(app, &window);
    window.imp().new_plan();
    window.present();
}

fn build_and_open(app: &Application, files: &[File], name: &str) {
    // Create new window and present it
    // todo Load plan into window.
    let window = Window::new(app);
    connect_actions(app, &window);
    window.imp().load_plan_from_files(files);
    window.present();
}

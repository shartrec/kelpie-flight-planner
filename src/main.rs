#![allow(deprecated)]

use std::ptr;
use gtk::gdk::Display;
use gtk::gio::{File, SimpleAction};
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib, Application, CssProvider};
use simplelog::*;

use window::Window;

mod earth;
mod event;
mod hangar;
mod model;
mod planner;
mod preference;
mod util;
mod window;

const APP_ID: &str = "com.shartrec.KelpiePlanner";

fn main() -> glib::ExitCode {
    init_logger();

    init_opengl();

    // Register and include resources
    gio::resources_register_include!("kelpie_planner.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gtk::gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_startup(|_app| {
        load_css();
    });

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    app.connect_open(build_and_open);

    // Run the application
    app.run()
}

fn init_opengl() {
    #[cfg(target_os = "macos")]
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.0.dylib") }.unwrap();
    #[cfg(all(unix, not(target_os = "macos")))]
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();
    #[cfg(windows)]
        let library = libloading::os::windows::Library::open_already_loaded("libepoxy-0.dll")
        .or_else(|_| libloading::os::windows::Library::open_already_loaded("epoxy-0.dll"))
        .unwrap();

    epoxy::load_with(|name| {
        unsafe { library.get::<_>(name.as_bytes()) }
            .map(|symbol| *symbol)
            .unwrap_or(ptr::null())
    });

    gl::load_with(|name| {
        epoxy::get_proc_addr(name)
    });

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
    ])
    .expect("Unable to initiate logger.");
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
    action.connect_activate(clone!(@weak window => move |_action, _parameter| {
       let _ = &window.imp().new_plan();
    }));
    app.add_action(&action);

    let action = SimpleAction::new("open", None);
    action.connect_activate(move |_action, _parameter| {
        todo!("Open clicked");
    });
    app.add_action(&action);

    let action = SimpleAction::new("save", None);
    action.connect_activate(move |_action, _parameter| {
        todo!("Save clicked");
    });
    app.add_action(&action);

    let action = SimpleAction::new("quit", None);
    action.connect_activate(clone!(@weak app => move |_action, _parameter| {
        app.quit()
    }));
    app.add_action(&action);

    let action = SimpleAction::new("preferences", None);
    action.connect_activate(clone!(@weak app => move |_action, _parameter| {
        // todo : show preference dialog
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

fn build_and_open(app: &Application, files: &[File], _name: &str) {
    // Create new window and present it
    // todo Load plan into window.
    let window = Window::new(app);
    connect_actions(app, &window);
    window.imp().load_plan_from_files(files);
    window.present();
}

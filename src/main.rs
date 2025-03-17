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

#![windows_subsystem = "windows"]

use std::fs::OpenOptions;
use std::ptr;

use adw::Application;
use gtk::{CssProvider, gio, glib, UriLauncher};
use adw::gdk::Display;
use gtk::gio::{Cancellable, File, SimpleAction};
use gtk::glib::clone;
use adw::prelude::*;
use adw::subclass::prelude::ObjectSubclassIsExt;
use log::error;
use simplelog::*;

use window::{preferences::PreferenceDialog, Window};

use crate::util::info;
use crate::window::util::show_help_about;

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
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
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
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.0.dylib") };
    #[cfg(all(unix, not(target_os = "macos")))]
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") };
    #[cfg(windows)]
        let library = libloading::os::windows::Library::open_already_loaded("libepoxy-0.dll")
        .or_else(|_| libloading::os::windows::Library::open_already_loaded("epoxy-0.dll"));

    match library {
        Ok(library) => {
            epoxy::load_with(|name| {
                unsafe { library.get::<_>(name.as_bytes()) }
                    .map(|symbol| *symbol)
                    .unwrap_or(ptr::null())
            });

            gl::load_with(|name| {
                epoxy::get_proc_addr(name)
            });
        }
        Err(err) => {
            error!("{}", err.to_string());
            error!("Unable to load OpenGl library");
        }
    }
}


fn init_logger() {
    if let Some(home_path) = home::home_dir() {
        let log_path = home_path.join("kelpie-planner.log");
        match OpenOptions::new().append(true).create(true).open(log_path) {
            Ok(file) => {
                let config = ConfigBuilder::new()
                    .set_time_offset_to_local()
                    .unwrap().build();
                let config2 = ConfigBuilder::new()
                    .set_location_level(LevelFilter::Error)
                    .set_time_offset_to_local()
                    .unwrap().build();
                CombinedLogger::init(vec![
                    TermLogger::new(
                        LevelFilter::Warn,
                        config.clone(),
                        TerminalMode::Mixed,
                        ColorChoice::Auto,
                    ),
                    WriteLogger::new(
                        LevelFilter::Info,
                        config2,
                        file,
                    ),
                ]).unwrap_or_else(|e| {
                    println!("Unable to initiate logger: {}.", e)
                });
                return;
            }
            Err(e) => println!("Unable to initiate logger: {}", e)
        }
    }
    TermLogger::init(
        LevelFilter::Warn,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    ).unwrap_or_else(|e| {
        println!("Unable to initiate logger: {}.", e)
    });
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
    action.connect_activate(clone!(#[weak] window, move |_action, _parameter| {
       let _ = &window.imp().new_plan();
    }));
    app.add_action(&action);

    let action = SimpleAction::new("open", None);
    action.connect_activate(clone!(#[weak] window, move |_action, _parameter| {
        let _ = &window.imp().open_plan();
    }));
    app.add_action(&action);

    let action = SimpleAction::new("save", None);
    action.connect_activate(clone!(#[weak] window, move  |_action, _parameter| {
        let _ = &window.imp().save_plan();
    }));
    app.add_action(&action);

    let action = SimpleAction::new("export_fg_rm", None);
    action.connect_activate(clone!(#[weak] window, move  |_action, _parameter| {
        let _ = &window.imp().export_plan();
    }));
    app.add_action(&action);

    let action = SimpleAction::new("quit", None);
    action.connect_activate(clone!(#[weak] app, move |_action, _parameter| {
        app.quit()
    }));
    app.add_action(&action);

    let action = SimpleAction::new("preferences", None);
    action.connect_activate(clone!(#[weak] window, move |_action, _parameter| {
        let pref_dialog = PreferenceDialog::new();
        pref_dialog.set_transient_for(Some(&window));
        pref_dialog.set_visible(true);
    }));
    app.add_action(&action);

    let action = SimpleAction::new("reload", None);
    action.connect_activate(clone!(#[weak] window, move |_action, _parameter| {
        let _ = &window.imp().reload();
    }));
    app.add_action(&action);

    let action = SimpleAction::new("help-about", None);
    action.connect_activate(clone!(#[weak] window, move |_action, _parameter| {
        show_help_about(&window);
    }));
    app.add_action(&action);

    let action = SimpleAction::new("help-contents", None);
    action.connect_activate(clone!(#[weak] window, move |_action, _parameter| {
        UriLauncher::builder()
            .uri(info::DOCSITE)
            .build()
            .launch(Some(&window), Some(&Cancellable::default()), |_| {});
    }));
    app.add_action(&action);
}

fn build_ui(app: &Application) {
    // Create new window and present it
    let window = Window::new(app);
    connect_actions(app, &window);
    window.present();
    window.imp().new_plan();
}

fn build_and_open(app: &Application, files: &[File], _name: &str) {
    // Create new window and present it
    let window = Window::new(app);
    connect_actions(app, &window);
    window.present();
    window.imp().load_plan_from_files(files);
}

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

use std::cell::RefCell;
use async_std::task;
use adw::{TabPage, TabView};
use adw::subclass::prelude::AdwApplicationWindowImpl;
use glib::Propagation;
use glib::subclass::InitializingObject;
use gtk::{AlertDialog, CompositeTemplate, FileDialog, glib, Label, Notebook, Paned};
use gtk::gio::{Cancellable, File};
use gtk::glib::{clone, MainContext};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use crate::event;
use crate::event::Event;

use crate::util::{get_plan_file_filter, plan_writer_route_manager, plan_writer_xml};
use crate::util::plan_reader::read_plan;
use crate::window::airport_map_view::AirportMapView;
use crate::window::airport_view::AirportView;
use crate::window::fix_view::FixView;
use crate::window::navaid_view::NavaidView;
use crate::window::plan_view::PlanView;
use crate::window::preferences::PreferenceDialog;
use crate::window::world_map_view::WorldMapView;

enum SaveType {
    Native,
    FgRouteManager,
}


// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/shartrec/kelpie_planner/window.ui")]
pub struct Window {
    #[template_child]
    pub pane_1v: TemplateChild<Paned>,
    #[template_child]
    pub pane_1h: TemplateChild<Paned>,
    #[template_child]
    pub search_notebook: TemplateChild<Notebook>,
    #[template_child]
    pub map_notebook: TemplateChild<Notebook>,
    #[template_child]
    pub airport_view: TemplateChild<AirportView>,
    #[template_child]
    pub navaid_view: TemplateChild<NavaidView>,
    #[template_child]
    pub fix_view: TemplateChild<FixView>,
    #[template_child]
    pub airport_map_view: TemplateChild<AirportMapView>,
    #[template_child]
    pub world_map_view: TemplateChild<WorldMapView>,
    #[template_child]
    pub plan_tab_view: TemplateChild<TabView>,
    #[template_child]
    pub status_bar: TemplateChild<Label>,
    my_listener_id: RefCell<usize>,
}

impl Window {
    pub(crate) fn load_plan_from_files(&self, files: &[File]) {
        todo!("Nead to load file{:?} here", files[0].path());
        // todo
    }

    pub(crate) fn new_plan(&self) {
        let view = PlanView::new();
        let page = self.plan_tab_view.append(&view);
        view.imp().set_parent_page(page.clone());
        view.imp().new_plan();
        self.plan_tab_view.set_selected_page(&page);
    }

    pub(crate) fn open_plan(&self) {
        let win = self.get_window_handle();

        let dialog = FileDialog::new();
        dialog.set_modal(true);
        dialog.set_title("Open Plan");
        let store = get_plan_file_filter("fgfp");
        dialog.set_filters(Some(&store));

        let x1 = &win.unwrap();
        let x = Some(x1);

        dialog.open(x, Some(&Cancellable::default()),
                    clone!(@weak self as window, => move | result: Result<File, _>| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        if let Ok(mut plan) = read_plan(&path) {
                            let view = PlanView::new();
                            let page = window.plan_tab_view.add_page(&view, None);
                            view.imp().set_parent_page(page.clone());
                            page.set_title(plan.get_name().as_str());
                            plan.set_path(Some(path));
                            window.plan_tab_view.set_selected_page(&page);
                            view.imp().set_plan(plan);
                        }
                    };
                }
            }));
    }


    pub(crate) fn save_plan(&self) {
        self.do_save("Save Plan", SaveType::Native);
    }

    pub(crate) fn export_plan(&self) {
        self.do_save("Export Plan", SaveType::FgRouteManager);
    }

    fn do_save(&self, title: &str, save_type: SaveType) {
        if let Some(page) = self.plan_tab_view.selected_page() {
            self.save_page_plan(title, save_type, &page, false);
        };
    }

    fn save_page_plan(&self, title: &str, save_type: SaveType, page: &TabPage, close_window: bool) {
        if let Ok(view) = page.child().downcast::<PlanView>() {
            let rc = view.imp().get_plan();
            let plan = rc.borrow();

            let ext = match save_type {
                SaveType::Native => "fgfp",
                SaveType::FgRouteManager => "xml",
            };
            let mut name = plan.get_name();
            let full_title = format!("{} : {}", title, name);
            if !name.ends_with(ext) {
                name.push('.');
                name.push_str(ext);
            }

            let win = self.get_window_handle();
            let dialog = FileDialog::new();
            dialog.set_modal(true);
            dialog.set_title(full_title.as_ref());
            dialog.set_initial_name(Some(name.as_str()));
            let store = get_plan_file_filter(ext);
            dialog.set_filters(Some(&store));

            let x1 = &win.unwrap();
            let xx = Some(x1.clone());
            let x = Some(x1);

            dialog.save(x, Some(&Cancellable::default()),
                        clone!(@weak self as window, @weak view, => move | result: Result<File, _>| {

                        if let Ok(file) = result {
                            let writer = match save_type {
                                        SaveType::Native => plan_writer_xml::write_plan,
                                        SaveType::FgRouteManager => plan_writer_route_manager::export_plan_fg,
                                    };
                            if let Some(path) = file.path() {
                                let binding = view.imp().get_plan();
                                let mut plan = binding.borrow_mut();
                                match writer(&plan, &path) {
                                    Ok(_) => {
                                        plan.set_dirty(false);
                                    }
                                    Err(s) => {
                                        let buttons = vec!["Ok".to_string()];
                                        let alert = AlertDialog::builder()
                                            .message(format!("Failed to save plan: {}", s))
                                            .buttons(buttons)
                                            .build();

                                        alert.show(xx.as_ref());
                                    }
                                };
                            };
                        }
                        if close_window {
                            window.obj().close();
                        }
                }));
        }
    }

    pub(crate) fn reload(&self) {
        // Spawn a new task to perform the initialization asynchronously
        task::spawn(async move {
            if let Err(_) = crate::earth::initialise() {
                event::manager().notify_listeners(Event::SetupRequired);
            }
        });
    }


    fn get_window_handle(&self) -> Option<gtk::Window> {
        match self.plan_tab_view.root() {
            Some(r) => {
                let window = r.downcast::<gtk::Window>().unwrap().clone();
                Some(window)
            }
            _ => {
                None
            }
        }
    }

    fn layout_panels(&self) {
        let pref = crate::preference::manager();

        // Set the size of the window
        if let Some(p) = pref.get::<i32>("vertical-split-pos") {
            self.pane_1v.set_position(p);
        }
        if let Some(p) = pref.get::<i32>("horizontal-split-pos") {
            self.pane_1h.set_position(p);
        }
    }

    fn save_panel_layout(&self) {
        let pref = crate::preference::manager();

        // Set the size of the window
        pref.put("vertical-split-pos", self.pane_1v.position());
        pref.put("horizontal-split-pos", self.pane_1h.position());
    }


    fn is_dirty(&self, page: &TabPage) -> (bool, Option<String>) {
        if let Ok(view) = page.child().downcast::<PlanView>() {
            let rc = view.imp().get_plan();
            let plan = rc.borrow();
            (plan.is_dirty(), Some(plan.get_name()))
        } else {
            (false, None)
        }
    }

    fn set_not_dirty(&self, page: &TabPage) {
        if let Ok(view) = page.child().downcast::<PlanView>() {
            let rc = view.imp().get_plan();
            let mut plan = rc.borrow_mut();
            plan.set_dirty(false);
        }
    }

    fn query_save_dirty<F>(&self, name: &str, close_callback: F)
        where F: FnOnce(i32) + 'static
    {

        let win = self.get_window_handle();

        let buttons = vec!["Yes".to_string(), "No".to_string(), "Cancel".to_string(), ];
        let alert = AlertDialog::builder()
            .modal(true)
            .message(format!("Save changes to plan {} before closing", name))
            .buttons(buttons)
            .default_button(0)
            .cancel_button(2)
            .build();

        alert.choose(win.as_ref(), Some(&Cancellable::default()), |result: Result<i32, _>| {
            if let Ok(button) = result {
                close_callback(button);
            }
        });

    }

}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "KelpiePlannerWindow";
    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        let obj = self.obj();
        obj.setup_actions();
        obj.load_window_size();

        self.layout_panels();

        // Listen for setup required message
        let (tx, rx) = async_channel::unbounded::<Event>();
        let index = event::manager().register_listener(tx);
        MainContext::default().spawn_local(clone!(@weak self as window, => async move {
                while let Ok(ev) = rx.recv().await {
                    match ev {
                        Event::SetupRequired => {
                            let buttons = vec!["Ok".to_string()];
                            let alert = AlertDialog::builder()
                                .modal(true)
                                .message("Please set paths to flightgear Airport and Navaid files".to_string())
                                .buttons(buttons)
                                .build();

                            let win = window.get_window_handle();
                            let win_clone = win.clone();
                            alert.choose(win.as_ref(), Some(&Cancellable::default()), move |_| {
                                let pref_dialog = PreferenceDialog::new();
                                pref_dialog.set_transient_for(win_clone.as_ref());
                                pref_dialog.show();
                            });
                        }
                        Event::StatusChange(s) => {
                            window.status_bar.set_label(s.as_str());
                        }
                        _ => {}
                    }
                };
        }));

        self.my_listener_id.replace(index);

        self.plan_tab_view.connect_close_page(clone!(@weak self as window => @default-return Propagation::Proceed, move |view, page|  {

            // Check if plan dirty and if so do save.
            if let (dirty, Some(name)) = window.is_dirty(&page.clone()) {
                if dirty {
                    window.query_save_dirty(&name, clone!(@weak window, @weak view, @weak page, => move |button| {

                        if button == 0 {
                            window.save_page_plan("Save Plan", SaveType::Native, &page, false);
                            view.close_page_finish(&page, true);
                        } else if button == 1 {
                            view.close_page_finish(&page, true);
                        } else {
                            view.close_page_finish(&page, false);
                        }
                    }));
                    Propagation::Stop
                } else {
                    Propagation::Proceed
                }
            } else {
                Propagation::Proceed
            }
        }));

        // Spawn a new task to perform the initialization asynchronously
        task::spawn(async move {
            if let Err(_) = crate::earth::initialise() {
                event::manager().notify_listeners(Event::SetupRequired);
            }
        });
    }

}

// Trait to allow us to add menubars
impl BuildableImpl for Window {}

// Trait shared by all widgets
impl WidgetImpl for Window {
    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        let old_height = self.pane_1h.height() as f32;
        let h_div = self.pane_1h.position() as f32 / old_height;
        let old_width = self.pane_1v.width() as f32;
        let v_div = self.pane_1v.position() as f32 / old_width;

        self.parent_size_allocate(width, height, baseline);

        // If the size = 0, the window probably isn't yet rendered so we don't want to adjust anything
        if old_height > 1.0 {
            let new_h_div = self.pane_1h.height() as f32 * h_div;
            let new_v_div = self.pane_1v.width() as f32 * v_div;

            self.pane_1v.set_position(new_v_div.round() as i32);
            self.pane_1h.set_position(new_h_div.round() as i32);
        }
    }
}

// Trait shared by all windows
impl WindowImpl for Window {
    // Save window state right before the window will be closed
    fn close_request(&self) -> glib::signal::Propagation {
        self.save_panel_layout();
        // Save window size
        self.obj()
            .save_window_size()
            .expect("Failed to save window state");

        let mut propagation = Propagation::Proceed;
        let n_pages = self.plan_tab_view.n_pages();
        for i in 0..n_pages {
            let page = self.plan_tab_view.nth_page(i);
            if let (dirty, Some(name)) = self.is_dirty(&self.plan_tab_view.nth_page(i)) {
                if dirty {
                    propagation = Propagation::Stop;
                    self.query_save_dirty(&name, clone!(@weak self as window => move |button| {
                        if button == 0 {
                            window.save_page_plan("Save Plan", SaveType::Native, &page, true);
                            window.set_not_dirty(&page);
                            if i > 1 {
                                window.plan_tab_view.close_page(&page);
                            }
                        } else if button == 1 {
                            window.set_not_dirty(&page);
                            if i > 1 {
                                window.plan_tab_view.close_page(&page);
                            }
                            let  _ = window.obj().close();
                        } else {
                            // cancel pressed . do nothing
                        }
                    }));
                    break;
                }
            }
        }
        propagation
    }
}

// Trait shared by all application windows
impl ApplicationWindowImpl for Window {}
impl AdwApplicationWindowImpl for Window {}

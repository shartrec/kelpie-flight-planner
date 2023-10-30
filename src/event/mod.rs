/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

use gtk::glib::Sender;
use lazy_static::lazy_static;

#[derive(Clone)]
pub enum Event {
    AirportsLoaded,
    NavaidsLoaded,
    FixesLoaded,
    PreferencesChanged,
}

lazy_static! {
    static ref MANAGER: EventManager = EventManager {
        listeners: RwLock::new(HashMap::new()),
        index: AtomicUsize::new(0),
    };
}
pub fn manager() -> &'static EventManager {
    &MANAGER
}

pub struct EventManager {
    listeners: RwLock<HashMap<usize, Sender<Event>>>,
    index: AtomicUsize,
}

impl EventManager {
    pub fn register_listener(&self, listener: Sender<Event>) -> usize {
        if let Ok(mut listeners) = self.listeners.write() {
            let i = self.index.fetch_add(1, Ordering::Relaxed);
            listeners.insert(i, listener);
            i
        } else {
            0
        }
    }

    pub fn unregister_listener(&self, index: &usize) {
        if let Ok(mut listeners) = self.listeners.write() {
            listeners.remove(index);
        }
    }

    pub fn notify_listeners(&self, ev: Event) {
        if let Ok(listeners) = self.listeners.read() {
            for listener in listeners.iter() {
                let _ = listener.1.send(ev.clone());
            }
        }
    }
}
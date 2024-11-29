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

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

use async_channel::Sender;
use lazy_static::lazy_static;

#[derive(Clone)]
pub enum Event {
    AirportsLoaded,
    NavaidsLoaded,
    FixesLoaded,
    PlanChanged,
    PreferencesChanged,
    SetupRequired,
    StatusChange(String)
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
                let _ = listener.1.try_send(ev.clone());
            }
        }
    }
}
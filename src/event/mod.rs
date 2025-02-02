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

use std::sync::{LazyLock, RwLock};

use async_channel::{Receiver, Sender, TrySendError};
use log::warn;

#[derive(Clone, PartialEq)]
pub enum Event {
    AirportsLoaded,
    NavaidsLoaded,
    FixesLoaded,
    PlanChanged,
    PreferencesChanged,
    SetupRequired,
    StatusChange(String),
}

static MANAGER: LazyLock<EventManager> = LazyLock::new(|| EventManager {
    listeners: RwLock::new(Vec::new()),
});

pub fn manager() -> &'static EventManager {
    &MANAGER
}

pub struct EventManager {
    listeners: RwLock<Vec<Sender<Event>>>,
}

impl EventManager {
    pub fn register_listener(&self) -> Option<Receiver<Event>> {
        let (tx, rx) = async_channel::unbounded::<Event>();

        if let Ok(mut listeners) = self.listeners.write() {
            listeners.push(tx);
            Some(rx)
        } else {
            None
        }
    }

    pub fn notify_listeners(&self, ev: Event) {
        if let Ok(listeners) = self.listeners.read() {
            for listener in listeners.iter() {
                match listener.try_send(ev.clone()) {
                    Ok(_) => {}
                    Err(TrySendError::Closed(_)) => {
                        warn!("Listener channel closed")
                    }
                    Err(TrySendError::Full(_)) => {}
                }
            }
        }
        if let Ok(mut listeners) = self.listeners.write() {
            listeners.retain(|l| !l.is_closed())
        }
    }
}
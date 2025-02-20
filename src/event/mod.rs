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
#[derive(Debug)]
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

        self.listeners.write().ok().map(|mut listeners| {
            listeners.push(tx);
            rx
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_channel::TryRecvError;

    #[test]
    fn test_register_listener() {
        let manager = EventManager {
            listeners: RwLock::new(Vec::new()),
        };

        let receiver = manager.register_listener();
        assert!(receiver.is_some());
    }

    #[test]
    fn test_notify_listeners() {
        let manager = EventManager {
            listeners: RwLock::new(Vec::new()),
        };

        let receiver = manager.register_listener().unwrap();
        manager.notify_listeners(Event::AirportsLoaded);

        match receiver.try_recv() {
            Ok(event) => assert_eq!(event, Event::AirportsLoaded),
            Err(_) => panic!("Expected event not received"),
        }
    }

    #[test]
    fn test_notify_multiple_listeners() {
        let manager = EventManager {
            listeners: RwLock::new(Vec::new()),
        };

        let receiver1 = manager.register_listener().unwrap();
        let receiver2 = manager.register_listener().unwrap();
        manager.notify_listeners(Event::NavaidsLoaded);

        match receiver1.try_recv() {
            Ok(event) => assert_eq!(event, Event::NavaidsLoaded),
            Err(_) => panic!("Expected event not received by listener 1"),
        }

        match receiver2.try_recv() {
            Ok(event) => assert_eq!(event, Event::NavaidsLoaded),
            Err(_) => panic!("Expected event not received by listener 2"),
        }
    }

    #[test]
    fn test_listener_channel_closed() {
        let manager = EventManager {
            listeners: RwLock::new(Vec::new()),
        };

        let receiver = manager.register_listener().unwrap();
        drop(receiver); // Close the receiver

        manager.notify_listeners(Event::FixesLoaded);

        // Ensure no listeners are left
        assert!(manager.listeners.read().unwrap().is_empty());
    }

    #[test]
    fn test_listener_channel_full() {
        let manager = EventManager {
            listeners: RwLock::new(Vec::new()),
        };

        let (tx, rx) = async_channel::bounded::<Event>(1);
        manager.listeners.write().unwrap().push(tx);

        // Fill the channel
        manager.notify_listeners(Event::PlanChanged);
        manager.notify_listeners(Event::PlanChanged);

        match rx.try_recv() {
            Ok(event) => assert_eq!(event, Event::PlanChanged),
            Err(_) => panic!("Expected event not received"),
        }

        // The second event should not be received because the channel is full
        match rx.try_recv() {
            Ok(_) => panic!("Unexpected event received"),
            Err(TryRecvError::Empty) => {}
            Err(_) => panic!("Unexpected error"),
        }
    }
}
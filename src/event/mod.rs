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
use std::sync::{LazyLock, RwLock};

use async_channel::{Receiver, Sender, TrySendError};
use log::warn;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EventType {
    AirportsLoaded,
    NavaidsLoaded,
    FixesLoaded,
    PlanChanged,
    PreferencesChanged,
    SetupRequired,
    StatusChange,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    AirportsLoaded,
    NavaidsLoaded,
    FixesLoaded,
    PlanChanged,
    PreferencesChanged,
    SetupRequired,
    StatusChange(String),
}

impl Event {
    fn event_type(&self) -> EventType {
        match self {
            Event::AirportsLoaded => EventType::AirportsLoaded,
            Event::NavaidsLoaded => EventType::NavaidsLoaded,
            Event::FixesLoaded => EventType::FixesLoaded,
            Event::PlanChanged => EventType::PlanChanged,
            Event::PreferencesChanged => EventType::PreferencesChanged,
            Event::SetupRequired => EventType::SetupRequired,
            Event::StatusChange(_) => EventType::StatusChange,
        }
    }
}

static MANAGER: LazyLock<EventManager> = LazyLock::new(|| EventManager {
    listeners: RwLock::new(HashMap::new()),
});

pub fn manager() -> &'static EventManager {
    &MANAGER
}

pub struct EventManager {
    listeners: RwLock<HashMap<EventType, Vec<Sender<Event>>>>,
}

impl EventManager {
    // Registers a listener for multiple `event_types`.
    // Returns a receiver that will receive copies of those events when notified.
    pub fn register_listener(&self, event_types: &[EventType]) -> Option<Receiver<Event>> {
        let (tx, rx) = async_channel::unbounded::<Event>();

            for event_type in event_types.iter().cloned() {
                self.listeners.write().ok().map(|mut listeners| {
                    listeners
                        .entry(event_type)
                        .or_insert_with(Vec::new)
                        .push(tx.clone());
                });
            }
        Some(rx)
    }

    /// Notify only listeners registered for the specific `ev`.
    pub fn notify_listeners(&self, ev: Event) {
        let key = ev.event_type();

        if let Ok(listeners) = self.listeners.read() {
            if let Some(vec) = listeners.get(&key) {
                for listener in vec.iter() {
                    match listener.try_send(ev.clone()) {
                        Ok(_) => {}
                        Err(TrySendError::Closed(_)) => {
                            warn!("Listener channel closed");
                        }
                        Err(TrySendError::Full(_)) => {}
                    }
                }
            }
        }
        if let Ok(mut listeners) = self.listeners.write() {
            // Remove closed senders and remove empty vectors
            listeners.retain(|_, v| {
                v.retain(|l| !l.is_closed());
                !v.is_empty()
            });
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
            listeners: RwLock::new(HashMap::new()),
        };

        let receiver = manager.register_listener(EventType::AirportsLoaded);
        assert!(receiver.is_some());
    }

    #[test]
    fn test_notify_listeners() {
        let manager = EventManager {
            listeners: RwLock::new(HashMap::new()),
        };

        let receiver = manager.register_listener(EventType::AirportsLoaded).unwrap();
        manager.notify_listeners(Event::AirportsLoaded);

        match receiver.try_recv() {
            Ok(event) => assert_eq!(event, Event::AirportsLoaded),
            Err(_) => panic!("Expected event not received"),
        }
    }

    #[test]
    fn test_notify_multiple_listeners() {
        let manager = EventManager {
            listeners: RwLock::new(HashMap::new()),
        };

        let receiver1 = manager.register_listener(EventType::NavaidsLoaded).unwrap();
        let receiver2 = manager.register_listener(EventType::NavaidsLoaded).unwrap();
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
            listeners: RwLock::new(HashMap::new()),
        };

        let receiver = manager.register_listener(EventType::FixesLoaded).unwrap();
        drop(receiver); // Close the receiver

        manager.notify_listeners(Event::FixesLoaded);

        // Ensure no listeners are left
        assert!(manager.listeners.read().unwrap().is_empty());
    }

    #[test]
    fn test_listener_channel_full() {
        let manager = EventManager {
            listeners: RwLock::new(HashMap::new()),
        };

        let (tx, rx) = async_channel::bounded::<Event>(1);
        manager
            .listeners
            .write()
            .unwrap()
            .entry(EventType::PlanChanged)
            .or_insert_with(Vec::new)
            .push(tx);

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

    #[test]
    fn test_status_change_with_payload() {
        let manager = EventManager {
            listeners: RwLock::new(HashMap::new()),
        };

        let rx = manager.register_listener(EventType::StatusChange).unwrap();
        manager.notify_listeners(Event::StatusChange("hello".to_string()));

        match rx.try_recv() {
            Ok(event) => match event {
                Event::StatusChange(s) => assert_eq!(s, "hello"),
                _ => panic!("Wrong event variant"),
            },
            Err(_) => panic!("Expected status change event"),
        }
    }
}

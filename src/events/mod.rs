use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

lazy_static! {
    static ref MANAGER: EventManager = EventManager {
        publisher: Arc::new(RwLock::new(Publisher::new())),
    };
}

trait EventListener {
    fn event_occured(&self, event_name: &String);
}

/// A subscriber (listener) has type of a callable function.
pub type Subscriber = fn();

pub struct EventManager {
    publisher: Arc<RwLock<Publisher>>,
}

impl EventManager {
    pub fn get_publisher(&self) -> &Arc<RwLock<Publisher>> {
        &self.publisher
    }
}

/// An event type.
#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Event {
    AirportsLoaded,
    NavaidsLoaded,
    FixesLoaded,
}

pub struct Publisher {
    events: Arc<RwLock<HashMap<Event, Vec<fn()>>>>,
}

impl Publisher {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::with_capacity(10))),
        }
    }

    pub fn subscribe(&self, event_type: Event, listener: Subscriber) {
        let mut map = self.events.write().unwrap();
        match map.get_mut(&event_type.clone()) {
            Some(v) => v.push(listener),
            None => {
                let mut v = Vec::with_capacity(10);
                v.push(listener);
                map.insert(event_type.clone(), v);
            }
        }
    }

    pub fn unsubscribe(&self, event_type: Event, listener: Subscriber) {
        self.events
            .write()
            .unwrap()
            .get_mut(&event_type)
            .unwrap()
            .retain(|&x| x != listener);
    }

    pub fn notify(&self, event_type: Event) {
        let map = self.events.write().unwrap();
        match map.get(&event_type.clone()) {
            Some(v) => {
                for listener in v {
                    listener();
                }
            }
            None => (),
        }
    }
}

pub fn get_publisher() -> &'static Arc<RwLock<Publisher>> {
    MANAGER.get_publisher()
}

use gtk::glib::PropertySet;
use std::cell::RefCell;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::model::plan::Plan;

use super::airport::Airport;
use super::waypoint::Waypoint;

pub struct Sector {
    airport_start: RefCell<Option<Waypoint>>,
    airport_end: RefCell<Option<Waypoint>>,
    waypoints: Arc<RwLock<Vec<Waypoint>>>,
}

impl Sector {
    pub fn new() -> Self {
        Sector {
            airport_start: RefCell::new(None),
            airport_end: RefCell::new(None),
            waypoints: Arc::new(RwLock::new(Vec::with_capacity(10))),
        }
    }

    pub fn set_start(&self, start: Arc<Airport>) {
        self.airport_start.set(Some(Waypoint::Airport {
            airport: start.clone(),
            locked: true,
        }));
    }

    pub fn set_end(&self, end: Arc<Airport>) {
        self.airport_end.set(Some(Waypoint::Airport {
            airport: end.clone(),
            locked: true,
        }));
    }
    pub fn clear_start(&self) {
        self.airport_start.set(None);
    }

    pub fn clear_end(&self) {
        self.airport_end.set(None);
    }
    pub fn insert_waypoint(&self, index: usize, waypoint: Waypoint) {
        if let Ok(mut vec) = self.waypoints.write() {
            if index <= vec.len() {
                vec.insert(index, waypoint);
            }
        }
    }

    pub fn add_waypoint(&self, waypoint: Waypoint) {
        if let Ok(mut vec) = self.waypoints.write() {
            vec.push(waypoint);
        }
    }

    pub fn add_all_waypoint(&self, waypoints: Vec<Waypoint>) {
        if let Ok(mut vec) = self.waypoints.write() {
            vec.clear();
            for wp in waypoints {
                vec.push(wp);
            }
        }
    }

    pub fn remove_waypoint(&self, index: usize) -> Option<Waypoint> {
        if let Ok(mut vec) = self.waypoints.write() {
            if index < vec.len() {
                Some(vec.remove(index))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_name(&self) -> String {
        let binding = self.airport_start.borrow();
        let w1 = binding.deref();
        let n1 = match w1 {
            Some(w) => w.get_id(),
            None => "",
        };
        let binding = self.airport_end.borrow();
        let w2 = binding.deref();
        let n2 = match w2 {
            Some(w) => w.get_id(),
            None => "",
        };
        format!("{} --> {}", n1, n2)
    }

    pub fn get_start(&self) -> Option<Waypoint> {
        self.airport_start.borrow().clone()
    }

    pub fn get_end(&self) -> Option<Waypoint> {
        self.airport_end.borrow().clone()
    }

    pub fn get_waypoints(&self) -> &Arc<RwLock<Vec<Waypoint>>> {
        &self.waypoints
    }

    pub fn get_waypoint_count(&self) -> usize {
        self.waypoints.read().unwrap().len()
    }

    pub fn get_waypoint(&self, pos: usize) -> Option<Waypoint> {
        Some(self.waypoints.read().unwrap()[pos].clone())
    }

    //todo
    pub fn get_duration(&self, plan: &Plan) -> f64 {
        match self.waypoints.read() {
            Ok(waypoints) => waypoints
                .iter()
                .map(move |wp| plan.get_time_to(wp))
                .reduce(|acc, t| acc + t)
                .unwrap_or(0.0),
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::earth::coordinate::Coordinate;
    use crate::model::test_utils::make_airport;
    use crate::model::waypoint::Waypoint;

    use super::Sector;

    #[test]
    fn test_set_start() {
        let s = Sector::new();
        s.set_start(make_airport("YSSY"));
        assert_eq!(s.get_start().unwrap().get_id(), "YSSY");
    }

    #[test]
    fn test_set_end() {
        let s = Sector::new();
        s.set_end(make_airport("YMLB"));
        assert_eq!(s.get_end().unwrap().get_id(), "YMLB");
    }

    #[test]
    fn test_waypoints() {
        let s = Sector::new();
        s.set_start(make_airport("YSSY"));
        s.set_end(make_airport("YMLB"));
        let w1 = Waypoint::Simple {
            loc: Coordinate::new(13.0, 111.0),
            elevation: Cell::new(10),
            locked: false,
        };
        let w2 = Waypoint::Simple {
            loc: Coordinate::new(23.0, 121.0),
            elevation: Cell::new(20),
            locked: false,
        };
        let w3 = Waypoint::Simple {
            loc: Coordinate::new(33.0, 131.0),
            elevation: Cell::new(30),
            locked: false,
        };
        let w4 = Waypoint::Simple {
            loc: Coordinate::new(43.0, 141.0),
            elevation: Cell::new(40),
            locked: false,
        };

        s.add_waypoint(w1.clone());
        s.add_waypoint(w2.clone());
        s.add_waypoint(w3.clone());
        s.insert_waypoint(1, w4.clone());

        let wps = s.waypoints.read().unwrap();
        assert_eq!(wps.len(), 4);
        assert_eq!(wps.get(0).unwrap().get_loc(), w1.get_loc());
        assert_eq!(wps.get(1).unwrap().get_loc(), w4.get_loc());
        assert_eq!(wps.get(2).unwrap().get_loc(), w2.get_loc());
        assert_eq!(wps.get(3).unwrap().get_loc(), w3.get_loc());
        drop(wps);

        s.remove_waypoint(2);
        let wps = s.waypoints.read().unwrap();
        assert_eq!(wps.len(), 3);
        assert_eq!(wps.get(0).unwrap().get_loc(), w1.get_loc());
        assert_eq!(wps.get(1).unwrap().get_loc(), w4.get_loc());
        assert_eq!(wps.get(2).unwrap().get_loc(), w3.get_loc());
    }
}

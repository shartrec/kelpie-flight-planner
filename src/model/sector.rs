use std::cell::RefCell;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use gtk::glib::{PropertyGet, PropertySet};

use super::airport::Airport;
use super::location::Location;
use super::waypoint::{AirportWaypoint, Waypoint};

struct Plan {}

pub struct Sector {
    airport_start: RefCell<Option<AirportWaypoint>>,
    airport_end: RefCell<Option<AirportWaypoint>>,
    waypoints: Arc<RwLock<Vec<Box<dyn Waypoint>>>>,
}

impl Sector {
    pub fn new() -> Self {
        Sector {
            airport_start: RefCell::new(None),
            airport_end: RefCell::new(None),
            waypoints: Arc::new(RwLock::new(Vec::with_capacity(10))),
        }
    }

    pub fn set_start(&mut self, start: &Airport) {
        self.airport_start.set(Some(AirportWaypoint::new(
            start.clone(),
            start.get_elevation().clone(),
            true,
        )));
    }

    pub fn set_end(&mut self, end: &Airport) {
        self.airport_end.set(Some(AirportWaypoint::new(
            end.clone(),
            end.get_elevation().clone(),
            true,
        )));
    }
    pub fn add_waypoint(&self, index: usize, waypoint: Box<dyn Waypoint>) {
        if index <= self.waypoints.read().unwrap().len() {
            self.waypoints.write().unwrap().insert(index, waypoint);
        }
    }

    pub fn add_all_waypoint(&self, waypoints: Vec<Box<dyn Waypoint>>) {
        let mut vec = self.waypoints.write().expect("Can't get sector lock");
        vec.clear();
        for wp in waypoints {
            println!("adding wp - {}", wp.get_name());
            vec.push(wp);
        }
    }




    // fn add_waypoint_after(&self, at: Box<dyn Waypoint>, waypoint_: Box<dyn Waypoint>) {
    //     let found_at = {
    //         let wps = self.waypoints.read().unwrap();
    //         wps.iter().position(|w| {
    //             waypoint::eq(w.copy(), at)
    //         })
    //     }; // Lock gets dropped here
    //     match found_at {
    //         Some(i) => self.add_waypoint(i + 1, waypoint_),
    //         None => self.add_waypoint(self.waypoints.read().unwrap().len(), waypoint_),
    //     }
    // }
    //
    // fn add_waypoint_before(&mut self, at: Box<dyn Waypoint>, waypoint_: Box<dyn Waypoint>) {
    //     let found_at = {
    //         let wps = self.waypoints.read().unwrap();
    //         wps.iter().position(|w| waypoint::eq(w.copy(), at))
    //     }; // Lock gets dropped here
    //     match found_at {
    //         Some(i) => self.add_waypoint(i, waypoint_),
    //         None => self.add_waypoint(self.waypoints.read().unwrap().len(), waypoint_),
    //     }
    // }
    //
    // fn remove_waypoint(&mut self, waypoint: Box<dyn Waypoint>) {
    //     let found_at = {
    //         let wps = self.waypoints.read().unwrap();
    //         wps.iter().position(|w| waypoint::eq(w.copy(), waypoint))
    //     }; // Lock gets dropped here
    //     match found_at {
    //         Some(i) => {
    //             self.waypoints.write().unwrap().remove(i);
    //             ()
    //         }
    //         None => (),
    //     }
    // }

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

    pub fn get_start(&self) -> Option<AirportWaypoint> {
        self.airport_start.borrow().clone()
    }

    pub fn get_end(&self) -> Option<AirportWaypoint> {
        self.airport_end.borrow().clone()
    }

    pub fn get_waypoints(&self) -> &Arc<RwLock<Vec<Box<dyn Waypoint>>>> {
        &self.waypoints
    }

    pub fn get_waypoint_count(&self) -> usize {
        self.waypoints.read().unwrap().len()
    }

    pub fn get_waypoint(&self, pos: usize) -> Option<Box<dyn Waypoint>> {
        Some(self.waypoints.read().unwrap()[pos].copy())
    }

    pub fn get_duration(&self) -> f64 {
        10.0
    }
    // Other methods of the Sector struct go here...
}

#[cfg(test)]
mod tests {
    use crate::earth::coordinate::Coordinate;
    use crate::model::test_utils::make_airport;
    use crate::model::waypoint::{SimpleWaypoint, Waypoint};

    use super::Sector;

    #[test]
    fn test_set_start() {
        let mut s = Sector::new();
        s.set_start(&make_airport("YSSY"));
        assert_eq!(s.get_start().unwrap().get_id(), "YSSY");
    }

    #[test]
    fn test_set_end() {
        let mut s = Sector::new();
        s.set_end(&make_airport("YMLB"));
        assert_eq!(s.get_end().unwrap().get_id(), "YMLB");
    }

    #[test]
    fn test_waypoints() {
        let mut s = Sector::new();
        s.set_start(&make_airport("YSSY"));
        s.set_end(&make_airport("YMLB"));
        let w1 =
            SimpleWaypoint::new_gps_waypoint("".to_string(), 10, Coordinate::new(13.0, 111.0));
        let w2 =
            SimpleWaypoint::new_gps_waypoint("".to_string(), 20, Coordinate::new(23.0, 121.0));
        let w3 =
            SimpleWaypoint::new_gps_waypoint("".to_string(), 30, Coordinate::new(33.0, 131.0));
        let w4 =
            SimpleWaypoint::new_gps_waypoint("".to_string(), 40, Coordinate::new(43.0, 141.0));
        // s.add_waypoint(0, Box::new(w1.clone()));
        // s.add_waypoint_after(Box::new(w1.clone()), Box::new(w2.clone()));
        // s.add_waypoint_after(Box::new(w2.clone()), Box::new(w3.clone()));
        // s.add_waypoint_before(Box::new(w2.clone()), Box::new(w4.clone()));
        //
        // let wps = s.waypoints.read().unwrap();
        // assert_eq!(wps.len(), 4);
        // assert_eq!(wps.get(0).unwrap().get_loc(), w1.get_loc());
        // assert_eq!(wps.get(1).unwrap().get_loc(), w4.get_loc());
        // assert_eq!(wps.get(2).unwrap().get_loc(), w2.get_loc());
        // assert_eq!(wps.get(3).unwrap().get_loc(), w3.get_loc());
        // drop(wps);
        //
        // s.remove_waypoint(Box::new(w2.clone()));
        // let wps = s.waypoints.read().unwrap();
        // assert_eq!(wps.len(), 3);
        // assert_eq!(wps.get(0).unwrap().get_loc(), w1.get_loc());
        // assert_eq!(wps.get(1).unwrap().get_loc(), w4.get_loc());
        // assert_eq!(wps.get(2).unwrap().get_loc(), w3.get_loc());
    }
}

use super::airport::Airport;
use super::location::Location;
use super::waypoint;
use super::waypoint::{AirportWaypoint, Waypoint};
use std::sync::{Arc, RwLock};

struct Plan {}

pub struct Sector {
    airport_start: Option<AirportWaypoint>,
    airport_end: Option<AirportWaypoint>,
    waypoints: Arc<RwLock<Vec<Box<dyn Waypoint>>>>,
}

impl Sector {
    pub fn new(start: Option<&Airport>, end: Option<&Airport>) -> Self {
        Sector {
            airport_start: start.map(|a| AirportWaypoint::new(a.clone(), a.get_elevation(), true)),
            airport_end: end.map(|a| AirportWaypoint::new(a.clone(), a.get_elevation(), true)),
            waypoints: Arc::new(RwLock::new(Vec::with_capacity(10))),
        }
    }

    pub fn set_start(&mut self, start: Airport) {
        self.airport_start = Some(AirportWaypoint::new(
            start.clone(),
            start.get_elevation(),
            true,
        ));
    }

    pub fn set_end(&mut self, end: Airport) {
        self.airport_end = Some(AirportWaypoint::new(end.clone(), end.get_elevation(), true));
    }

    pub fn add_waypoint(&self, index: usize, waypoint: Box<dyn Waypoint>) {
        self.waypoints.write().unwrap().insert(index, waypoint);
    }

    fn add_waypoint_after(&self, at: Box<dyn Waypoint>, waypoint_: Box<dyn Waypoint>) {
        let found_at = {
	        let wps = self.waypoints.read().unwrap();
        	wps.iter().position(|w| waypoint::eq(w, &at))
		}; // Lock gets dropped here
        match found_at {
        	Some(i) => self.add_waypoint(i + 1, waypoint_),
        	None => self.add_waypoint(self.waypoints.read().unwrap().len(), waypoint_),
        }
    }

    fn add_waypoint_before(&mut self, at: Box<dyn Waypoint>, waypoint_: Box<dyn Waypoint>) {
        let found_at = {
	        let wps = self.waypoints.read().unwrap();
        	wps.iter().position(|w| waypoint::eq(w, &at))
		}; // Lock gets dropped here
        match found_at {
   	    	Some(i) => self.add_waypoint(i, waypoint_),
        	None => self.add_waypoint(self.waypoints.read().unwrap().len(), waypoint_),
        }
    }

    fn remove_waypoint(&mut self, waypoint: Box<dyn Waypoint>) {
        let found_at = {
	        let wps = self.waypoints.read().unwrap();
        	wps.iter().position(|w| waypoint::eq(w, &waypoint))
		}; // Lock gets dropped here
		match found_at {
   	    	Some(i) => {
   	    		self.waypoints.write().unwrap().remove(i);
   	    		()
   	    		},
   	    	None => (),
        }
    }

    pub fn get_start(&self) -> Option<AirportWaypoint> {
        self.airport_start.clone()
    }

    pub fn get_end(&self) -> Option<AirportWaypoint> {
        self.airport_end.clone()
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
    use crate::model::airport::{Airport, AirportType};
    use crate::model::test_utils::make_airport;
    use crate::model::waypoint::{AirportWaypoint, SimpleWaypoint, Waypoint};

    use super::Sector;

    #[test]
    fn test_set_start() {
        let mut s = Sector::new(None, None);
        s.set_start(make_airport("YSSY"));
        assert_eq!(s.airport_start.unwrap().get_id(), "YSSY");
    }

    #[test]
    fn test_set_end() {
        let mut s = Sector::new(None, None);
        s.set_end(make_airport("YMLB"));
        assert_eq!(s.airport_end.unwrap().get_id(), "YMLB");
    }

    #[test]
    fn test_waypoints() {
        let mut s = Sector::new(Some(&make_airport("YSSY")), Some(&make_airport("YMLB")));
        let w1 =
            SimpleWaypoint::new_gps_waypoint("".to_string(), 10.0, Coordinate::new(13.0, 111.0));
        let w2 =
            SimpleWaypoint::new_gps_waypoint("".to_string(), 20.0, Coordinate::new(23.0, 121.0));
        let w3 =
            SimpleWaypoint::new_gps_waypoint("".to_string(), 30.0, Coordinate::new(33.0, 131.0));
        let w4 =
            SimpleWaypoint::new_gps_waypoint("".to_string(), 40.0, Coordinate::new(43.0, 141.0));
        s.add_waypoint(0, Box::new(w1.clone()));
        s.add_waypoint_after(Box::new(w1.clone()), Box::new(w2.clone()));
        s.add_waypoint_after(Box::new(w2.clone()), Box::new(w3.clone()));
        s.add_waypoint_before(Box::new(w2.clone()), Box::new(w4.clone()));

        let wps = s.waypoints.read().unwrap();
        assert_eq!(wps.len(), 4);
        assert_eq!(wps.get(0).unwrap().get_loc(), w1.get_loc());
        assert_eq!(wps.get(1).unwrap().get_loc(), w4.get_loc());
        assert_eq!(wps.get(2).unwrap().get_loc(), w2.get_loc());
        assert_eq!(wps.get(3).unwrap().get_loc(), w3.get_loc());
        drop(wps);

        s.remove_waypoint(Box::new(w2.clone()));
        let wps = s.waypoints.read().unwrap();
        assert_eq!(wps.len(), 3);
        assert_eq!(wps.get(0).unwrap().get_loc(), w1.get_loc());
        assert_eq!(wps.get(1).unwrap().get_loc(), w4.get_loc());
        assert_eq!(wps.get(2).unwrap().get_loc(), w3.get_loc());
    }
}

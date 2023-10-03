use std::cell::Cell;

use crate::earth::coordinate::Coordinate;
use crate::model::location::Location;

use super::{airport::Airport, fix::Fix, navaid::Navaid};

#[derive(Clone, PartialEq)]
pub enum Waypoint {
    Simple {loc: Coordinate, elevation: Cell<i32>, locked: bool },
    Toc {loc: Coordinate, elevation: Cell<i32>, locked: bool },
    Bod {loc: Coordinate, elevation: Cell<i32>, locked: bool },
    Navaid { navaid: Navaid, elevation: Cell<i32>, locked: bool },
    Fix { fix: Fix, elevation: Cell<i32>, locked: bool },
    Airport { airport: Airport, locked: bool },
}

impl Waypoint {
    pub(crate) fn get_id(&self) -> &str {
        match self {
            Waypoint::Simple { loc, elevation, locked} => {
                "GPS"
            }
            Waypoint::Toc{loc, elevation, locked} => {
                "TOC"
            }
            Waypoint::Bod {loc, elevation, locked} => {
                "BOD"
            }
            Waypoint::Navaid{navaid, elevation, locked} => {
                &navaid.get_id()
            }
            Waypoint::Fix{fix, elevation, locked} => {
                &fix.get_id()
            }
            Waypoint::Airport{airport, locked} => {
                &airport.get_id()
            }
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            Waypoint::Simple{loc, elevation, locked} => {
                "GPS Waypoint"
            }
            Waypoint::Toc{loc, elevation, locked} => {
                "Top of climb"
            }
            Waypoint::Bod{loc, elevation, locked} => {
                "Beginning of descent"
            }
            Waypoint::Navaid{navaid, elevation, locked} => {
                &navaid.get_name()
            }
            Waypoint::Fix{fix, elevation, locked} => {
                &fix.get_name()
            }
            Waypoint::Airport{airport, locked} => {
                &airport.get_name()
            }
        }
    }

    pub(crate) fn get_elevation(&self) -> i32 {
        match self {
            Waypoint::Simple{loc, elevation, locked} => {
                elevation.get().clone()
            }
            Waypoint::Toc{loc, elevation, locked} => {
                elevation.get().clone()
            }
            Waypoint::Bod{loc, elevation, locked} => {
                elevation.get().clone()
            }
            Waypoint::Navaid{navaid, elevation, locked} => {
                elevation.get().clone()
            }
            Waypoint::Fix{fix, elevation, locked} => {
                elevation.get().clone()
            }
            Waypoint::Airport{airport, locked} => {
                airport.get_elevation().clone()
            }
        }
    }

    pub(crate) fn get_loc(&self) -> &Coordinate{
        match self {
            Waypoint::Simple{loc, elevation, locked} => {
                &loc
            }
            Waypoint::Toc{loc, elevation, locked} => {
                &loc
            }
            Waypoint::Bod{loc, elevation, locked} => {
                &loc
            }
            Waypoint::Navaid{navaid, elevation, locked} => {
                &navaid.get_loc()
            }
            Waypoint::Fix{fix, elevation, locked} => {
                &fix.get_loc()
            }
            Waypoint::Airport{airport, locked} => {
                &airport.get_loc()
            }
        }
    }

    pub fn get_lat(&self) -> &f64{
        &self.get_loc().get_latitude()
    }

    pub fn get_freq(&self) -> Option<&f64> {
    None
    }
    pub(crate) fn get_lat_as_string(&self) -> String{
        self.get_loc().get_latitude_as_string()
    }

    pub fn get_long(&self) -> &f64{
        &self.get_loc().get_longitude()
    }

    pub(crate) fn get_long_as_string(&self) -> String{
        self.get_loc().get_longitude_as_string()
    }

    fn is_locked(&self) -> &bool{
        match self {
            Waypoint::Simple{loc, elevation, locked} => {
                &locked
            }
            Waypoint::Toc{loc, elevation, locked} => {
                &locked
            }
            Waypoint::Bod{loc, elevation, locked} => {
                &locked
            }
            Waypoint::Navaid{navaid, elevation, locked} => {
                &locked
            }
            Waypoint::Fix{fix, elevation, locked} => {
                &locked
            }
            Waypoint::Airport{airport, locked} => {
                &locked
            }
        }
    }

    pub(crate) fn set_elevation(&self, elev: &i32) {
        match self {
            Waypoint::Simple{loc, elevation, locked} => {
                &elevation.set(elev.clone());
            }
            Waypoint::Toc{loc, elevation, locked} => {
                &elevation.set(elev.clone());
            }
            Waypoint::Bod{loc, elevation, locked} => {
                &elevation.set(elev.clone());
            }
            Waypoint::Navaid{navaid, elevation, locked} => {
                &elevation.set(elev.clone());
            }
            Waypoint::Fix{fix, elevation, locked} => {
                &elevation.set(elev.clone());
            }
            Waypoint::Airport{airport, locked} => {}
        }
    }







}









/*
#[cfg(test)]
mod tests {
    use crate::earth::coordinate::Coordinate;
    use crate::model::test_utils::make_airport;
    use crate::model::waypoint::{self, AirportWaypoint, SimpleWaypoint, Waypoint};

    #[test]
    fn test_equality() {
        let w1 =
            SimpleWaypoint::new_gps_waypoint("".to_string(), 10, Coordinate::new(13.0, 111.0));
        let w2 =
            SimpleWaypoint::new_gps_waypoint("".to_string(), 20, Coordinate::new(23.0, 121.0));

        let a = Box::new(w1.clone());
        let b = Box::new(w1.clone());
        assert!(do_test(a, b));
        let a = Box::new(w1.clone());
        let b = Box::new(w2.clone());
        assert!(!do_test(a, b));
    }

    #[test]
    fn test_equality_airport_type() {
        let ap = make_airport("YSSY");
        let w1 = AirportWaypoint::new(ap, 20, false);
        let ap = make_airport("YMLB");
        let w2 = AirportWaypoint::new(ap, 20, false);
        let a = Box::new(w1.clone());
        let b = Box::new(w1.clone());
        assert!(do_test(a, b));
        let a = Box::new(w1.clone());
        let b = Box::new(w2.clone());
        assert!(!do_test(a, b));
    }

    #[test]
    fn test_equality_diff_type() {
        let w1 =
            SimpleWaypoint::new_gps_waypoint("".to_string(), 10, Coordinate::new(13.0, 111.0));
        let ap = make_airport("YSSY");
        let w2 = AirportWaypoint::new(ap, 20, false);
        let a = Box::new(w1.clone());
        let b = Box::new(w2.clone());
        assert!(!do_test(a, b));
    }

    fn do_test(a: Box<dyn Waypoint>, b: Box<dyn Waypoint>) -> bool {
        waypoint::eq(a, b)
    }
}
*/
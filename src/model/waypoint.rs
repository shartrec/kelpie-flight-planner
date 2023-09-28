use std::cell::Cell;

use crate::earth::coordinate::Coordinate;

use super::{airport::Airport, fix::Fix, location::Location, navaid::Navaid};

pub trait Waypoint {
    fn get_id(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_type(&self) -> &WaypointType;
    fn get_elevation(&self) -> i32;
    fn get_loc(&self) -> &Coordinate;
    fn get_lat(&self) -> &f64;
    fn get_freq(&self) -> Option<&f64> {
        None
    }
    fn get_lat_as_string(&self) -> String;
    fn get_long(&self) -> &f64;
    fn get_long_as_string(&self) -> String;
    fn is_locked(&self) -> &bool;
    fn copy(&self) -> Box<dyn Waypoint>;
    fn set_elevation(&self, elevation: &i32);
}

pub fn eq(a: Box<dyn Waypoint>, b: Box<dyn Waypoint>) -> bool {
    if a.get_type().ne(&b.get_type()) {
        return false;
    }
    // We already know both a & b are the same type
    match a.get_type() {
        WaypointType::AIRPORT => a.get_id().eq(b.get_id()),
        WaypointType::NAVAID => a.get_id().eq(b.get_id()),
        _ => a.get_loc().eq(&b.get_loc()),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaypointType {
    AIRPORT,
    BOD,
    GPS,
    FIX,
    NAVAID,
    TOC,
}

#[derive(Debug, Clone)]
pub struct SimpleWaypoint {
    id: String,
    elevation: Cell<i32>,
    loc: Coordinate,
    lock: bool,
    type_: WaypointType,
}

impl SimpleWaypoint {
    pub fn new_gps_waypoint(id: String, elevation: i32, loc: Coordinate) -> Self {
        SimpleWaypoint {
            id,
            elevation : Cell::new(elevation),
            loc,
            lock: false,
            type_: WaypointType::GPS,
        }
    }
    pub fn new_toc_waypoint(id: String, elevation: i32, loc: Coordinate) -> Self {
        SimpleWaypoint {
            id,
            elevation : Cell::new(elevation),
            loc,
            lock: false,
            type_: WaypointType::TOC,
        }
    }
    pub fn new_bod_waypoint(id: String, elevation: i32, loc: Coordinate) -> Self {
        SimpleWaypoint {
            id,
            elevation : Cell::new(elevation),
            loc,
            lock: false,
            type_: WaypointType::BOD,
        }
    }
}

impl Waypoint for SimpleWaypoint {
    fn get_id(&self) -> &str {
        &self.id
    }
    fn get_name(&self) -> &str {
        &self.id
    }
    fn get_type(&self) -> &WaypointType {
        &self.type_
    }
    fn get_elevation(&self) -> i32 {
        self.elevation.get()
    }
    fn get_loc(&self) -> &Coordinate {
        &self.loc
    }
    fn get_lat(&self) -> &f64 {
        &self.loc.get_latitude()
    }
    fn get_lat_as_string(&self) -> String {
        self.loc.get_latitude_as_string()
    }
    fn get_long(&self) -> &f64 {
        &self.loc.get_longitude()
    }
    fn get_long_as_string(&self) -> String {
        self.loc.get_longitude_as_string()
    }
    fn is_locked(&self) -> &bool {
        &self.lock
    }

    fn copy(&self) -> Box<dyn Waypoint> {
        Box::new(self.clone())
    }

    fn set_elevation(&self, elevation: &i32) {
        self.elevation.set(elevation.clone());
    }
}

#[derive(Clone)]
pub struct FixWaypoint {
    fix: Fix,
    elevation: Cell<i32>,
    locked: bool,
}

impl FixWaypoint {
    pub fn new(fix: Fix, elevation: i32, locked: bool) -> Self {
        FixWaypoint {
            fix,
            elevation : Cell::new(elevation),
            locked,
        }
    }
}

impl Waypoint for FixWaypoint {
    fn get_id(&self) -> &str {
        &self.fix.get_id()
    }
    fn get_name(&self) -> &str {
        &self.fix.get_name()
    }
    fn get_type(&self) -> &WaypointType {
        &WaypointType::FIX
    }
    fn get_elevation(&self) -> i32 {
        self.elevation.get()
    }
    fn get_loc(&self) -> &Coordinate {
        self.fix.get_loc()
    }
    fn get_lat(&self) -> &f64 {
        self.fix.get_lat()
    }
    fn get_lat_as_string(&self) -> String {
        self.fix.get_lat_as_string()
    }
    fn get_long(&self) -> &f64 {
        self.fix.get_long()
    }
    fn get_long_as_string(&self) -> String {
        self.fix.get_long_as_string()
    }
    fn is_locked(&self) -> &bool {
        &self.locked
    }
    fn copy(&self) -> Box<dyn Waypoint> {
        Box::new(self.clone())
    }
    fn set_elevation(&self, elevation: &i32) {
        self.elevation.set(elevation.clone());
    }
}

#[derive(Clone)]
pub struct NavaidWaypoint {
    navaid: Navaid,
    elevation: Cell<i32>,
    locked: bool,
}

impl NavaidWaypoint {
    pub fn new(navaid: Navaid, elevation: i32, locked: bool) -> Self {
        NavaidWaypoint {
            navaid,
            elevation : Cell::new(elevation),
            locked,
        }
    }
}

impl Waypoint for NavaidWaypoint {
    fn get_id(&self) -> &str {
        self.navaid.get_id()
    }
    fn get_name(&self) -> &str {
        self.navaid.get_name()
    }
    fn get_type(&self) -> &WaypointType {
        &WaypointType::NAVAID
    }
    fn get_elevation(&self) -> i32 {
        self.elevation.get()
    }
    fn get_loc(&self) -> &Coordinate {
        &self.navaid.get_loc()
    }
    fn get_lat(&self) -> &f64 {
        &self.navaid.get_lat()
    }
    fn get_lat_as_string(&self) -> String {
        self.navaid.get_lat_as_string()
    }
    fn get_long(&self) -> &f64 {
        &self.navaid.get_long()
    }
    fn get_long_as_string(&self) -> String {
        self.navaid.get_long_as_string()
    }
    fn is_locked(&self) -> &bool {
        &self.locked
    }
    fn get_freq(&self) -> Option<&f64> {
        Some(self.navaid.get_freq())
    }
    fn copy(&self) -> Box<dyn Waypoint> {
        Box::new(self.clone())
    }
    fn set_elevation(&self, elevation: &i32) {
        self.elevation.set(elevation.clone());
    }
}

#[derive(Clone)]
pub struct AirportWaypoint {
    airport: Airport,
    elevation: Cell<i32>,
    locked: bool,
}

impl AirportWaypoint {
    pub fn new(airport: Airport, elevation: i32, locked: bool) -> Self {
        AirportWaypoint {
            airport,
            elevation : Cell::new(elevation),
            locked,
        }
    }

    pub fn get_airport(&self) -> Airport {
        self.airport.clone()
    }
}

impl Waypoint for AirportWaypoint {
    fn get_id(&self) -> &str {
        &self.airport.get_id()
    }
    fn get_name(&self) -> &str {
        &self.airport.get_name()
    }
    fn get_type(&self) -> &WaypointType {
        &WaypointType::AIRPORT
    }
    fn get_elevation(&self) -> i32 {
        self.elevation.get()
    }
    fn get_loc(&self) -> &Coordinate {
        &self.airport.get_loc()
    }
    fn get_lat(&self) -> &f64 {
        &self.airport.get_lat()
    }
    fn get_lat_as_string(&self) -> String {
        self.airport.get_lat_as_string()
    }
    fn get_long(&self) -> &f64 {
        &self.airport.get_long()
    }
    fn get_long_as_string(&self) -> String {
        self.airport.get_long_as_string()
    }
    fn is_locked(&self) -> &bool {
        &self.locked
    }
    fn copy(&self) -> Box<dyn Waypoint> {
        Box::new(self.clone())
    }
    fn set_elevation(&self, elevation: &i32) {
    }
}

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

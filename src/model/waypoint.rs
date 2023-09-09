use super::{airport::Airport, fix::Fix, location::Location, navaid::Navaid};
use crate::earth::coordinate::Coordinate;

pub trait Waypoint {
    fn get_id(&self) -> String;
    fn get_name(&self) -> String;
    fn get_type(&self) -> WaypointType;
    fn get_elevation(&self) -> f64;
    fn get_loc(&self) -> Coordinate;
    fn get_lat(&self) -> f64;
    fn get_lat_as_string(&self) -> String;
    fn get_long(&self) -> f64;
    fn get_long_as_string(&self) -> String;
    fn is_locked(&self) -> bool;
    fn copy(&self) -> Box<dyn Waypoint>;
}

pub fn eq(a: &Box<dyn Waypoint>, b: &Box<dyn Waypoint>) -> bool {
    if a.get_type().ne(&b.get_type()) {
        return false;
    }

    match a.get_type() {
        WaypointType::AIRPORT => a.get_id().eq(&b.get_id()),
        WaypointType::NAVAID => a.get_id().eq(&b.get_id()),
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
    elevation: f64,
    loc: Coordinate,
    lock: bool,
    type_: WaypointType,
}

impl SimpleWaypoint {
    pub fn new_gps_waypoint(id: String, elevation: f64, loc: Coordinate) -> Self {
        SimpleWaypoint {
            id,
            elevation,
            loc,
            lock: false,
            type_: WaypointType::GPS,
        }
    }
    pub fn new_toc_waypoint(id: String, elevation: f64, loc: Coordinate) -> Self {
        SimpleWaypoint {
            id,
            elevation,
            loc,
            lock: false,
            type_: WaypointType::TOC,
        }
    }
    pub fn new_bod_waypoint(id: String, elevation: f64, loc: Coordinate) -> Self {
        SimpleWaypoint {
            id,
            elevation,
            loc,
            lock: false,
            type_: WaypointType::BOD,
        }
    }
}

impl Waypoint for SimpleWaypoint {
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.id.clone()
    }
    fn get_type(&self) -> WaypointType {
        self.type_.clone()
    }
    fn get_elevation(&self) -> f64 {
        self.elevation
    }
    fn get_loc(&self) -> Coordinate {
        self.loc.clone()
    }
    fn get_lat(&self) -> f64 {
        self.loc.get_latitude()
    }
    fn get_lat_as_string(&self) -> String {
        self.loc.get_latitude_as_string()
    }
    fn get_long(&self) -> f64 {
        self.loc.get_longitude()
    }
    fn get_long_as_string(&self) -> String {
        self.loc.get_longitude_as_string()
    }
    fn is_locked(&self) -> bool {
        self.lock
    }

    fn copy(&self) -> Box<dyn Waypoint> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct FixWaypoint {
    fix: Fix,
    elevation: f64,
    locked: bool,
}

impl FixWaypoint {
    pub fn new(fix: Fix, elevation: f64, locked: bool) -> Self {
        FixWaypoint {
            fix,
            elevation,
            locked,
        }
    }
}

impl Waypoint for FixWaypoint {
    fn get_id(&self) -> String {
        self.fix.get_id()
    }
    fn get_name(&self) -> String {
        self.fix.get_name()
    }
    fn get_type(&self) -> WaypointType {
        WaypointType::FIX
    }
    fn get_elevation(&self) -> f64 {
        self.elevation
    }
    fn get_loc(&self) -> Coordinate {
        self.fix.get_loc()
    }
    fn get_lat(&self) -> f64 {
        self.fix.get_lat()
    }
    fn get_lat_as_string(&self) -> String {
        self.fix.get_lat_as_string()
    }
    fn get_long(&self) -> f64 {
        self.fix.get_long()
    }
    fn get_long_as_string(&self) -> String {
        self.fix.get_long_as_string()
    }
    fn is_locked(&self) -> bool {
        self.locked
    }
    fn copy(&self) -> Box<dyn Waypoint> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct NavaidWaypoint {
    navaid: Navaid,
    elevation: f64,
    locked: bool,
}

impl NavaidWaypoint {
    pub fn new(navaid: Navaid, elevation: f64, locked: bool) -> Self {
        NavaidWaypoint {
            navaid,
            elevation,
            locked,
        }
    }
}

impl Waypoint for NavaidWaypoint {
    fn get_id(&self) -> String {
        self.navaid.get_id()
    }
    fn get_name(&self) -> String {
        self.navaid.get_name()
    }
    fn get_type(&self) -> WaypointType {
        WaypointType::NAVAID
    }
    fn get_elevation(&self) -> f64 {
        self.elevation
    }
    fn get_loc(&self) -> Coordinate {
        self.navaid.get_loc()
    }
    fn get_lat(&self) -> f64 {
        self.navaid.get_lat()
    }
    fn get_lat_as_string(&self) -> String {
        self.navaid.get_lat_as_string()
    }
    fn get_long(&self) -> f64 {
        self.navaid.get_long()
    }
    fn get_long_as_string(&self) -> String {
        self.navaid.get_long_as_string()
    }
    fn is_locked(&self) -> bool {
        self.locked
    }
    fn copy(&self) -> Box<dyn Waypoint> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct AirportWaypoint {
    airport: Airport,
    elevation: f64,
    locked: bool,
}

impl AirportWaypoint {
    pub fn new(airport: Airport, elevation: f64, locked: bool) -> Self {
        AirportWaypoint {
            airport,
            elevation,
            locked,
        }
    }

    pub fn get_airport(&self) -> Airport {
        self.airport.clone()
    }
}

impl Waypoint for AirportWaypoint {
    fn get_id(&self) -> String {
        self.airport.get_id()
    }
    fn get_name(&self) -> String {
        self.airport.get_name()
    }
    fn get_type(&self) -> WaypointType {
        WaypointType::AIRPORT
    }
    fn get_elevation(&self) -> f64 {
        self.elevation
    }
    fn get_loc(&self) -> Coordinate {
        self.airport.get_loc()
    }
    fn get_lat(&self) -> f64 {
        self.airport.get_lat()
    }
    fn get_lat_as_string(&self) -> String {
        self.airport.get_lat_as_string()
    }
    fn get_long(&self) -> f64 {
        self.airport.get_long()
    }
    fn get_long_as_string(&self) -> String {
        self.airport.get_long_as_string()
    }
    fn is_locked(&self) -> bool {
        self.locked
    }
    fn copy(&self) -> Box<dyn Waypoint> {
        Box::new(self.clone())
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
            SimpleWaypoint::new_gps_waypoint("".to_string(), 10.0, Coordinate::new(13.0, 111.0));
        let w2 =
            SimpleWaypoint::new_gps_waypoint("".to_string(), 20.0, Coordinate::new(23.0, 121.0));

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
        let w1 = AirportWaypoint::new(ap, 20.0, false);
        let ap = make_airport("YMLB");
        let w2 = AirportWaypoint::new(ap, 20.0, false);
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
            SimpleWaypoint::new_gps_waypoint("".to_string(), 10.0, Coordinate::new(13.0, 111.0));
        let ap = make_airport("YSSY");
        let w2 = AirportWaypoint::new(ap, 20.0, false);
        let a = Box::new(w1.clone());
        let b = Box::new(w2.clone());
        assert!(!do_test(a, b));
    }

    fn do_test(a: Box<dyn Waypoint>, b: Box<dyn Waypoint>) -> bool {
        waypoint::eq(&a, &b)
    }
}

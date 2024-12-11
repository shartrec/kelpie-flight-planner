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

use std::cell::Cell;
use std::sync::Arc;

use crate::earth::coordinate::Coordinate;
use crate::model::location::Location;

use super::{airport::Airport, fix::Fix, navaid::Navaid};

#[derive(Clone, PartialEq)]
pub enum Waypoint {
    Simple {
        loc: Coordinate,
        elevation: Cell<i32>,
        locked: bool,
    },
    Toc {
        loc: Coordinate,
        elevation: Cell<i32>,
        locked: bool,
    },
    Bod {
        loc: Coordinate,
        elevation: Cell<i32>,
        locked: bool,
    },
    Navaid {
        navaid: Arc<Navaid>,
        elevation: Cell<i32>,
        locked: bool,
    },
    Fix {
        fix: Arc<Fix>,
        elevation: Cell<i32>,
        locked: bool,
    },
    Airport {
        airport: Arc<Airport>,
        locked: bool,
    },
}

impl Waypoint {
    pub(crate) fn get_type_name(&self) -> &str {
        match self {
            Waypoint::Simple { .. } => "GPS",
            Waypoint::Toc { .. } => "TOC",
            Waypoint::Bod { .. } => "BOD",
            Waypoint::Navaid { .. } => "NAVAID",
            Waypoint::Fix { .. } => "FIX",
            Waypoint::Airport { .. } => "AIRPORT",
        }
    }

    pub(crate) fn get_id(&self) -> &str {
        match self {
            Waypoint::Simple { .. } => "GPS",
            Waypoint::Toc { .. } => "TOC",
            Waypoint::Bod { .. } => "BOD",
            Waypoint::Navaid { navaid, .. } => navaid.get_id(),
            Waypoint::Fix { fix, .. } => fix.get_id(),
            Waypoint::Airport { airport, .. } => airport.get_id(),
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            Waypoint::Simple { .. } => "GPS Waypoint",
            Waypoint::Toc { .. } => "Top of climb",
            Waypoint::Bod { .. } => "Beginning of descent",
            Waypoint::Navaid { navaid, .. } => navaid.get_name(),
            Waypoint::Fix { fix, .. } => fix.get_name(),
            Waypoint::Airport { airport, .. } => airport.get_name(),
        }
    }

    pub(crate) fn get_elevation(&self) -> i32 {
        match self {
            Waypoint::Simple {
                loc: _, elevation, ..
            } => elevation.get(),
            Waypoint::Toc {
                loc: _, elevation, ..
            } => elevation.get(),
            Waypoint::Bod {
                loc: _, elevation, ..
            } => elevation.get(),
            Waypoint::Navaid {
                navaid: _,
                elevation,
                ..
            } => elevation.get(),
            Waypoint::Fix {
                fix: _, elevation, ..
            } => elevation.get(),
            Waypoint::Airport { airport, .. } => *airport.get_elevation(),
        }
    }

    pub(crate) fn get_loc(&self) -> &Coordinate {
        match self {
            Waypoint::Simple { loc, .. } => loc,
            Waypoint::Toc { loc, .. } => loc,
            Waypoint::Bod { loc, .. } => loc,
            Waypoint::Navaid { navaid, .. } => navaid.get_loc(),
            Waypoint::Fix { fix, .. } => fix.get_loc(),
            Waypoint::Airport { airport, .. } => airport.get_loc(),
        }
    }

    pub fn get_lat(&self) -> &f64 {
        self.get_loc().get_latitude()
    }

    pub fn get_freq(&self) -> Option<&f64> {
        match self {
            Waypoint::Navaid { navaid, .. } => Some(navaid.get_freq()),
            _ => None,
        }
    }
    pub(crate) fn get_lat_as_string(&self) -> String {
        self.get_loc().get_latitude_as_string()
    }

    pub fn get_long(&self) -> &f64 {
        self.get_loc().get_longitude()
    }

    pub(crate) fn get_long_as_string(&self) -> String {
        self.get_loc().get_longitude_as_string()
    }

    #[allow(dead_code)]
    pub fn is_locked(&self) -> &bool {
        match self {
            Waypoint::Simple {
                loc: _,
                elevation: _,
                locked,
            } => locked,
            Waypoint::Toc {
                loc: _,
                elevation: _,
                locked,
            } => locked,
            Waypoint::Bod {
                loc: _,
                elevation: _,
                locked,
            } => locked,
            Waypoint::Navaid {
                navaid: _,
                elevation: _,
                locked,
            } => locked,
            Waypoint::Fix {
                fix: _,
                elevation: _,
                locked,
            } => locked,
            Waypoint::Airport {
                airport: _,
                locked
            } => locked,
        }
    }

    pub(crate) fn get_airport(&self) -> Arc<Airport> {
        match self {
            Waypoint::Airport { airport, .. } => airport.clone(),
            _ => {
                panic!("Not an airport waypoint")
            }
        }
    }

    pub(crate) fn set_elevation(&self, elev: &i32) {
        match self {
            Waypoint::Simple {
                loc: _, elevation, ..
            } => {
                elevation.set(*elev);
            }
            Waypoint::Toc {
                loc: _, elevation, ..
            } => {
                elevation.set(*elev);
            }
            Waypoint::Bod {
                loc: _, elevation, ..
            } => {
                elevation.set(*elev);
            }
            Waypoint::Navaid {
                navaid: _,
                elevation,
                ..
            } => {
                elevation.set(*elev);
            }
            Waypoint::Fix {
                fix: _, elevation, ..
            } => {
                elevation.set(*elev);
            }
            Waypoint::Airport { airport: _, .. } => {}
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

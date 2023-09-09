/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */

use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::earth;
use crate::earth::coordinate::Coordinate;
use crate::model::fix::Fix;
use crate::model::location::Location;
use crate::model::navaid::{Navaid, NavaidType};
use crate::model::waypoint::{FixWaypoint, NavaidWaypoint, SimpleWaypoint, Waypoint};
use crate::preference::*;

pub const ARRIVAL_BEACON_RANGE: f64 = 10.0;

struct Planner<'a> {
    max_leg_distance: f64,
    min_leg_distance: f64,
    max_deviation: f64,
    vor_only: bool,
    vor_preferred: bool,
    plan_type: String,
    add_gps_waypoints: bool,
    add_waypoint_bias: bool,
    navaids: &'a Arc<RwLock<Vec<Box<Navaid>>>>,
    fixes: &'a Arc<RwLock<Vec<Box<Fix>>>>,
}

impl Planner<'_> {
    pub fn new() -> Self {
        let pref = crate::preference::manager();

        Self {
            max_leg_distance: pref.get::<f64>(MAX_LEG_LENGTH).unwrap_or(100.0),
            min_leg_distance: pref.get::<f64>(MIN_LEG_LENGTH).unwrap_or(25.0),
            max_deviation: pref.get::<f64>(MAX_DEVIATION).unwrap_or(10.0),
            vor_only: pref.get::<bool>(VOR_ONLY).unwrap_or(false),
            vor_preferred: pref.get::<bool>(VOR_PREFERED).unwrap_or(true),
            add_gps_waypoints: pref.get::<bool>(ADD_WAYPOINTS).unwrap_or(false),
            add_waypoint_bias: pref.get::<bool>(ADD_WAYPOINT_BIAS).unwrap_or(false),
            plan_type: pref.get::<String>(PLAN_TYPE).unwrap_or(USE_RADIO_BEACONS.to_string()),
            navaids: earth::get_earth_model().get_navaids(),
            fixes: earth::get_earth_model().get_fixes(),
        }
    }
    fn make_plan(&mut self, from: &dyn Waypoint, to: &dyn Waypoint) -> Vec<Box<dyn Waypoint>> {
        let mut plan: Vec<Box<dyn Waypoint>> = Vec::new();
        if self.plan_type == USE_RADIO_BEACONS {

            // We use an iterative process of finding the navaid
            // nearest to the midpoint and then do the same recursively
            // while the leg length is greater than MAX_LEG_LENGTH
            self.add_navaids_to_plan(from, to, &mut plan);
            if self.add_gps_waypoints {
                self.add_waypoints(from, to, &mut plan);
            }
        } else if self.plan_type == USE_FIXES {

            // We use an iterative process of finding the fix
            // nearest to the midpoint and then do the same recursively
            // while the leg length is greater than MAX_LEG_LENGTH
            self.add_fixes_to_plan(from, to, &mut plan);
            if self.add_gps_waypoints {
                self.add_waypoints(from, to, &mut plan);
            }
        } else {
            self.add_waypoints_between(from, to, &mut plan);
        }
        plan
    }

    fn add_navaids_to_plan(&self, from: &dyn Waypoint,
                           to: &dyn Waypoint,
                           plan: &mut Vec<Box<dyn Waypoint>>) {
        if let Some(arrival_beacon) = self.get_navaid_nearest(&to.get_loc(), ARRIVAL_BEACON_RANGE) {
            let wp: NavaidWaypoint = NavaidWaypoint::new(*arrival_beacon, 0f64, false);

            self.add_navaids_between(from, &wp.clone(), plan);
            plan.push(Box::new(wp.clone()));
        } else {
            self.add_navaids_between(from, to, plan);
        }
    }

    fn add_fixes_to_plan(&self, from: &dyn Waypoint,
                         to: &dyn Waypoint,
                         plan: &mut Vec<Box<dyn Waypoint>>) {
        self.add_fixes_between(from, to, plan);
    }

    fn add_navaids_between(&self, from: &dyn Waypoint,
                           to: &dyn Waypoint,
                           plan: &mut Vec<Box<dyn Waypoint>>) {
        let distance = from.get_loc().distance_to(&to.get_loc());

        if distance < self.max_leg_distance {
            return;
        }

        let heading = from.get_loc().bearing_to(&to.get_loc()).to_degrees();
        let midpoint = from.get_loc().coordinate_at(distance / 2.0, heading);

        if let Some(mid_nav_aid) = self.get_navaid_nearest_midpoint(&from.get_loc(), &to.get_loc(), &midpoint) {
            let wp = NavaidWaypoint::new(*mid_nav_aid, 0f64, false);

            self.add_navaids_between(from, &wp, plan);
            plan.push(Box::new(wp.clone()));

            self.add_navaids_between(&wp, to, plan);
        }
    }

    fn add_fixes_between(&self, from: &dyn Waypoint,
                         to: &dyn Waypoint,
                         plan: &mut Vec<Box<dyn Waypoint>>) {
        let distance = from.get_loc().distance_to(&to.get_loc());

        if distance < self.max_leg_distance {
            return;
        }

        let heading = from.get_loc().bearing_to(&to.get_loc()).to_degrees();
        let midpoint = from.get_loc().coordinate_at(distance / 2.0, heading);

        if let Some(mid_fix_aid) = self.get_fix_nearest_midpoint(&from.get_loc(), &to.get_loc(), &midpoint) {
            let wp = FixWaypoint::new(*mid_fix_aid, 0f64, false);

            self.add_fixes_between(from, &wp, plan);
            plan.push(Box::new(wp.clone()));

            self.add_fixes_between(&wp, to, plan);
        }
    }

    fn get_navaid_nearest_midpoint(
        &self,
        from: &Coordinate,
        to: &Coordinate,
        midpoint: &Coordinate,
    ) -> Option<Box<Navaid>> {
        let leg_distance = from.distance_to(to);
        let heading_from = from.bearing_to_deg(midpoint);
        let heading_to = midpoint.bearing_to_deg(to);

        let range = leg_distance / 2.0; // - _min_leg_distance;

        let near_aids = self.get_locations_near(self.navaids, midpoint, range);
        let mut best_loc: Option<Box<Navaid>> = None;
        let mut best_ndb: Option<Box<Navaid>> = None;
        let mut nearest = 100000.0;
        let mut nearest_ndb = 100000.0;

        for navaid in near_aids {
            if self.vor_only && navaid.get_type() != NavaidType::VOR {
                continue;
            }

            let deviation_to = self.get_deviation(heading_from, from.bearing_to_deg(&navaid.get_loc()));
            let deviation_from = self.get_deviation(heading_to, navaid.get_loc().bearing_to_deg(to));

            if deviation_to > self.max_deviation || deviation_from > self.max_deviation {
                continue;
            }

            if self.vor_preferred && navaid.get_type() != NavaidType::VOR {
                if midpoint.distance_to(&navaid.get_loc()) < nearest_ndb
                    && from.distance_to(&navaid.get_loc()) > self.min_leg_distance
                    && to.distance_to(&navaid.get_loc()) > self.min_leg_distance
                {
                    best_ndb = Some(navaid.clone());
                    nearest_ndb = midpoint.distance_to(&navaid.get_loc());
                }
            } else {
                if midpoint.distance_to(&navaid.get_loc()) < nearest
                    && from.distance_to(&navaid.get_loc()) > self.min_leg_distance
                    && to.distance_to(&navaid.get_loc()) > self.min_leg_distance
                {
                    best_loc = Some(navaid.clone());
                    nearest = midpoint.distance_to(&navaid.get_loc());
                }
            }
        }

        if best_loc.is_none() && best_ndb.is_some() {
            best_loc = best_ndb;
        }

        best_loc
    }

    fn get_navaid_nearest(&self, coord: &Coordinate, max_range: f64) -> Option<Box<Navaid>> {
        let near_aids = self.get_locations_near::<Navaid>(&self.navaids, coord, max_range);

        let mut best_loc: Option<Box<Navaid>> = None;
        let mut best_ndb: Option<Box<Navaid>> = None;
        let mut nearest = 100000.0;
        let mut nearest_ndb = 100000.0;

        for navaid in near_aids {
            if self.vor_only && navaid.get_type() != NavaidType::VOR {
                continue;
            }

            if self.vor_preferred && navaid.get_type() != NavaidType::VOR {
                if coord.distance_to(&navaid.get_loc()) < nearest_ndb {
                    best_ndb = Some(navaid.clone());
                    nearest_ndb = coord.distance_to(&navaid.get_loc());
                }
            } else {
                if coord.distance_to(&navaid.get_loc()) < nearest {
                    best_loc = Some(navaid.clone());
                    nearest = coord.distance_to(&navaid.get_loc());
                }
            }
        }

        if best_loc.is_none() && best_ndb.is_some() {
            best_loc = best_ndb;
        }

        best_loc
    }
    fn get_fix_nearest_midpoint(
        &self,
        from: &Coordinate,
        to: &Coordinate,
        midpoint: &Coordinate,
    ) -> Option<Box<Fix>> {
        let leg_distance = from.distance_to(to);
        let heading_from = from.bearing_to_deg(midpoint);
        let heading_to = midpoint.bearing_to_deg(to);

        let range = leg_distance / 2.0; // - _min_leg_distance;

        let near_aids = self.get_locations_near(self.fixes, midpoint, range);
        let mut best_loc: Option<Box<Fix>> = None;
        let mut nearest = 100000.0;

        for fix in near_aids {
            let deviation_to = self.get_deviation(heading_from, from.bearing_to_deg(&fix.get_loc()));
            let deviation_from = self.get_deviation(heading_to, fix.get_loc().bearing_to_deg(to));

            if deviation_to > self.max_deviation || deviation_from > self.max_deviation {
                continue;
            }

            if midpoint.distance_to(&fix.get_loc()) < nearest
                && from.distance_to(&fix.get_loc()) > self.min_leg_distance
                && to.distance_to(&fix.get_loc()) > self.min_leg_distance
            {
                best_loc = Some(fix.clone());
                nearest = midpoint.distance_to(&fix.get_loc());
            }
        }

        best_loc
    }

    fn get_fix_nearest(&self, coord: &Coordinate, max_range: f64) -> Option<Box<Fix>> {
        let near_aids = self.get_locations_near::<Fix>(&self.fixes, coord, max_range);

        let mut best_loc: Option<Box<Fix>> = None;
        let mut nearest = 100000.0;

        for fix in near_aids {
                if coord.distance_to(&fix.get_loc()) < nearest {
                    best_loc = Some(fix.clone());
                    nearest = coord.distance_to(&fix.get_loc());
                }
        }
        best_loc
    }

    fn add_waypoints_between(&self, from: &dyn Waypoint,
                           to: &dyn Waypoint,
                           plan: &mut Vec<Box<dyn Waypoint>>) {
        let distance = from.get_loc().distance_to(&to.get_loc());

        if distance < self.max_leg_distance {
            return;
        }

        let heading = from.get_loc().bearing_to(&to.get_loc()).to_degrees();
        let midpoint = from.get_loc().coordinate_at(distance / 2.0, heading);

        let wp = SimpleWaypoint::new_gps_waypoint("<Waypoint>".to_string(), 0f64, midpoint);

            self.add_waypoints_between(from, &wp, plan);
            plan.push(Box::new(wp.clone()));

            self.add_waypoints_between(&wp, to, plan);

    }

    fn add_waypoints(&mut self, from: &dyn Waypoint, to: &dyn Waypoint, plan: &mut Vec<Box<dyn Waypoint>>) {

        // Walk the legs and find those that are over the wished-for interval
        let max_leg_interval: f64 = if self.add_waypoint_bias {
            self.max_leg_distance * 0.75
        } else {
            self.max_leg_distance * 1.25
        };

        let mut prev_wp = from.clone();
        if plan.len() > 0 {
            let mut finished = false;
            while !finished {
                for i in 0..plan.len() {
                    let wp = &plan[i];
                    let leg_length = prev_wp.get_loc().distance_to(&wp.get_loc());
                    if leg_length >= max_leg_interval {
                        // The following changes the structure of the plan so we need to get out and start over
                        self.add_waypoints_to_leg(prev_wp, to, plan, i, leg_length);
                        break;
                    }
                    finished = true;
                }
            }
        }
        // Try for the final leg
        let leg_length = prev_wp.get_loc().distance_to(&to.get_loc());
        if leg_length >= max_leg_interval {
            self.add_waypoints_to_leg(prev_wp, to, plan, plan.len(), leg_length);
        }
    }

    fn add_waypoints_to_leg(&mut self, prev_wp: &dyn Waypoint, to: &dyn Waypoint, plan: &mut Vec<Box<dyn Waypoint>>, i: usize, leg_length: f64) {

        let mut additional_points = leg_length / self.max_leg_distance;
        let extra_points = if (self.add_waypoint_bias && (additional_points.fract() > 0.2)) {
            additional_points.ceil()
        } else {
            additional_points.floor()
        } as usize;
        let interval = leg_length / extra_points as f64;

        println!("Interval {}", interval);

        let mut last_wp = prev_wp.copy();

        for a_pos in 0..extra_points - 1 {
            let heading = last_wp.get_loc().bearing_to_deg(&to.get_loc());
            let x_loc = last_wp.get_loc().coordinate_at(interval, heading);
            let x = SimpleWaypoint::new_gps_waypoint("<Waypoint>".to_string(), 0.0, x_loc); //$NON-NLS-1$
            plan.insert(i + a_pos, Box::new(x.clone()));
            last_wp = Box::new(x.clone());
        }
    }

    fn get_locations_near<'a, T: Location>(&'a self, locations: &'a Arc<RwLock<Vec<Box<T>>>>, point: &Coordinate, range: f64)
                                           -> Vec<Box<T>> {
        // We do a little optimization here rather than calculating
        // all distances accurately; we make a quick rough calculation to exclude many coordinates
        let rough_lat_sep = range / 60.0;
        let rough_lon_sep = range / (60.0 * point.get_latitude().to_radians().cos());

        let mut near_locations: Vec<Box<T>> = Vec::new();

        let guard = locations.read().unwrap();
        let locations = guard.deref();
        let x: Vec<&Box<T>> = locations.iter().filter(move |loc| {
            let loc_coord = loc.get_loc();
            {
                if (point.get_latitude() - loc_coord.get_latitude()).abs() > rough_lat_sep {
                    return false;
                }
                if (point.get_longitude() - loc_coord.get_longitude()).abs() > rough_lon_sep {
                    return false;
                }
                let distance = point.distance_to(&loc_coord);
                distance <= range
            }
        }).collect();

        for l in x {
            near_locations.push(Box::new(*l.clone()));
        }
        near_locations
    }

    fn get_deviation(&self, heading_from: f64, bearing_to_deg: f64) -> f64 {
        let mut raw_deviation = (bearing_to_deg - heading_from).abs();
        if raw_deviation > 180.0 {
            raw_deviation = 360.0 - raw_deviation;
        }
        raw_deviation
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use crate::model::test_utils::make_airport_at;
    use crate::model::waypoint::AirportWaypoint;
    use crate::preference::USE_GPS;
    use super::Planner;

    #[test]
    fn test_with_gps() {
        let mut planner = Planner {
            max_leg_distance: 100.0,
            min_leg_distance: 25.0,
            max_deviation: 10.0,
            vor_only: false,
            vor_preferred: true,
            add_gps_waypoints: false,
            add_waypoint_bias: false,
            plan_type: USE_GPS.to_string(),
            navaids: &Arc::new(RwLock::new(Vec::new())),
            fixes: &Arc::new(RwLock::new(Vec::new())),
        };

        let ap = make_airport_at("YSSY", -34.0, 151.0);
        let w1 = AirportWaypoint::new(ap, 20.0, false);
        let ap = make_airport_at("YPER", -32.1, 120.5);
        let w2 = AirportWaypoint::new(ap, 20.0, false);

        let plan = planner.make_plan(&w1, &w2);

        for wp in &plan {
            println!("WP - {}, {}", wp.get_lat_as_string(), wp.get_long_as_string())
        }
        assert_eq!(plan.len(), 15);
    }
}

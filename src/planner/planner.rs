/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};

use crate::earth;
use crate::earth::coordinate::Coordinate;
use crate::model::fix::Fix;
use crate::model::location::Location;
use crate::model::navaid::{Navaid, NavaidType};
use crate::model::plan::Plan;
use crate::model::waypoint::{AirportWaypoint, FixWaypoint, NavaidWaypoint, SimpleWaypoint, Waypoint, WaypointType};
use crate::preference::*;

pub const ARRIVAL_BEACON_RANGE: i32 = 10;

pub struct Planner<'a> {
    max_leg_distance: i32,
    min_leg_distance: i32,
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
            max_leg_distance: pref.get::<i32>(MAX_LEG_LENGTH).unwrap_or(100),
            min_leg_distance: pref.get::<i32>(MIN_LEG_LENGTH).unwrap_or(25),
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
    pub(crate) fn make_plan(&self, from: Option<AirportWaypoint>, to: Option<AirportWaypoint>) -> Vec<Box<dyn Waypoint>> {
        let mut plan: Vec<Box<dyn Waypoint>> = Vec::new();

        if let Some(from) = from {
            if let Some(to) = to {
                if self.plan_type == USE_RADIO_BEACONS {

                    // We use an iterative process of finding the navaid
                    // nearest to the midpoint and then do the same recursively
                    // while the leg length is greater than MAX_LEG_LENGTH
                    self.add_navaids_to_plan(&from, &to, &mut plan);
                    if self.add_gps_waypoints {
                        self.add_waypoints(&from, &to, &mut plan);
                    }
                } else if self.plan_type == USE_FIXES {

                    // We use an iterative process of finding the fix
                    // nearest to the midpoint and then do the same recursively
                    // while the leg length is greater than MAX_LEG_LENGTH
                    self.add_fixes_to_plan(&from, &to, &mut plan);
                    if self.add_gps_waypoints {
                        self.add_waypoints(&from, &to, &mut plan);
                    }
                } else {
                    self.add_fixes_to_plan(&from, &to, &mut plan);
                    if self.add_gps_waypoints {
                        self.add_waypoints(&from, &to, &mut plan);
                    }
                }
            }
        }
        plan
    }

    fn add_navaids_to_plan(&self, from: &dyn Waypoint,
                           to: &dyn Waypoint,
                           plan: &mut Vec<Box<dyn Waypoint>>) {
        if let Some(arrival_beacon) = self.get_navaid_nearest(&to.get_loc(), ARRIVAL_BEACON_RANGE) {
            let wp: NavaidWaypoint = NavaidWaypoint::new(*arrival_beacon, 0, false);

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
        let distance = from.get_loc().distance_to(&to.get_loc()) as i32;

        if distance < self.max_leg_distance {
            return;
        }

        let heading = from.get_loc().bearing_to(&to.get_loc()).to_degrees();
        let midpoint = from.get_loc().coordinate_at(distance / 2, heading);

        if let Some(mid_nav_aid) = self.get_navaid_nearest_midpoint(&from.get_loc(), &to.get_loc(), &midpoint) {
            let wp = NavaidWaypoint::new(*mid_nav_aid, 0, false);

            self.add_navaids_between(from, &wp, plan);
            plan.push(Box::new(wp.clone()));

            self.add_navaids_between(&wp, to, plan);
        }
    }

    fn add_fixes_between(&self, from: &dyn Waypoint,
                         to: &dyn Waypoint,
                         plan: &mut Vec<Box<dyn Waypoint>>) {
        let distance = from.get_loc().distance_to(&to.get_loc()) as i32;

        if distance < self.max_leg_distance {
            return;
        }

        let heading = from.get_loc().bearing_to(&to.get_loc()).to_degrees();
        let midpoint = from.get_loc().coordinate_at(distance / 2, heading);

        if let Some(mid_fix_aid) = self.get_fix_nearest_midpoint(&from.get_loc(), &to.get_loc(), &midpoint) {
            let wp = FixWaypoint::new(*mid_fix_aid, 0, false);

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

        let range = leg_distance as i32 / 2; // - _min_leg_distance;

        let near_aids = self.get_locations_near(self.navaids, midpoint, range);
        let mut best_loc: Option<Box<Navaid>> = None;
        let mut best_ndb: Option<Box<Navaid>> = None;
        let mut nearest = 100000;
        let mut nearest_ndb = 100000;

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

    fn get_navaid_nearest(&self, coord: &Coordinate, max_range: i32) -> Option<Box<Navaid>> {
        let near_aids = self.get_locations_near::<Navaid>(&self.navaids, coord, max_range);

        let mut best_loc: Option<Box<Navaid>> = None;
        let mut best_ndb: Option<Box<Navaid>> = None;
        let mut nearest = 100000;
        let mut nearest_ndb = 100000;

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

        let range = leg_distance / 2; // - _min_leg_distance;

        let near_aids = self.get_locations_near(self.fixes, midpoint, range);
        let mut best_loc: Option<Box<Fix>> = None;
        let mut nearest = 100000;

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

    fn get_fix_nearest(&self, coord: &Coordinate, max_range: i32) -> Option<Box<Fix>> {
        let near_aids = self.get_locations_near::<Fix>(&self.fixes, coord, max_range);

        let mut best_loc: Option<Box<Fix>> = None;
        let mut nearest = 100000;

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
        let midpoint = from.get_loc().coordinate_at(distance / 2, heading);

        let wp = SimpleWaypoint::new_gps_waypoint("<Waypoint>".to_string(), 0, midpoint);

        self.add_waypoints_between(from, &wp, plan);
        plan.push(Box::new(wp.clone()));

        self.add_waypoints_between(&wp, to, plan);
    }

    fn add_waypoints(&self, from: &dyn Waypoint, to: &dyn Waypoint, plan: &mut Vec<Box<dyn Waypoint>>) {

        // Walk the legs and find those that are over the wished-for interval
        let max_leg_interval: i32 = if self.add_waypoint_bias {
            self.max_leg_distance * 3 / 4
        } else {
            self.max_leg_distance * 5 / 4
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

    fn add_waypoints_to_leg(&self, prev_wp: &dyn Waypoint, to: &dyn Waypoint, plan: &mut Vec<Box<dyn Waypoint>>, i: usize, leg_length: i32) {
        let mut additional_points = leg_length as f64 / self.max_leg_distance as f64;
        let extra_points = if (self.add_waypoint_bias && (additional_points.fract() > 0.2)) {
            additional_points.ceil()
        } else {
            additional_points.floor()
        } as i32;
        let interval = leg_length / extra_points ;

        println!("Interval {}", interval);

        let mut last_wp = prev_wp.copy();

        for a_pos in 0..extra_points - 1 {
            let heading = last_wp.get_loc().bearing_to_deg(&to.get_loc());
            let x_loc = last_wp.get_loc().coordinate_at(interval, heading);
            let x = SimpleWaypoint::new_gps_waypoint("<Waypoint>".to_string(), 0, x_loc); //$NON-NLS-1$
            plan.insert(i + a_pos as usize, Box::new(x.clone()));
            last_wp = Box::new(x.clone());
        }
    }

    fn get_locations_near<'a, T: Location>(&'a self, locations: &'a Arc<RwLock<Vec<Box<T>>>>, point: &Coordinate, range: i32)
                                           -> Vec<Box<T>> {
        // We do a little optimization here rather than calculating
        // all distances accurately; we make a quick rough calculation to exclude many coordinates
        let rough_lat_sep = range as f64 / 60.0;
        let rough_lon_sep = range as f64 / (60.0 * point.get_latitude().to_radians().cos());

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

    pub fn recalc_plan_elevations(&self, mut plan: &Plan) {
        for s_ref in plan.get_sectors().deref() {
            let binding = s_ref.borrow();
            let sector = binding.deref();
            if sector.get_start().is_none() || sector.get_end().is_none() {
                continue;
            }

            // Remove the previous top of climb and beginning of descent
            let mut guard = sector.get_waypoints().write().expect("Can't get read lock on sectors");
            let mut waypoints = guard.deref_mut();
            waypoints.retain(|wp| {
                wp.get_type() != WaypointType::TOC && wp.get_type() != WaypointType::BOD
            });

            let max_alt = calc_max_altitude(
                plan,
                &sector.get_start().unwrap(),
                &sector.get_end().unwrap(),
                waypoints,
            );

            add_toc(
                plan,
                &sector.get_start().unwrap(),
                &sector.get_end().unwrap(),
                waypoints,
                max_alt,
            );

            add_bod(
                plan,
                &sector.get_start().unwrap(),
                &sector.get_end().unwrap(),
                waypoints,
                max_alt,
            );

            set_elevations(
                plan,
                &sector.get_start().unwrap(),
                &sector.get_end().unwrap(),
                waypoints,
                max_alt,
            );
        }
    }
}

pub fn add_bod(plan: &Plan, from: &AirportWaypoint, to: &AirportWaypoint, waypoints: &mut Vec<Box<dyn Waypoint>>, max_alt: i32) {
    let mut done = false;
    let alt_to_bod = max_alt - to.get_elevation() as i32;
    let aircraft = plan.get_aircraft();
    let time_to_bod = alt_to_bod  as f64 / aircraft.as_ref().map(|a| a.get_sink_rate().clone() as f64).unwrap_or(500.0) / 60.0;
    let dist_to_bod = aircraft.as_ref().map(|a| a.get_sink_speed().clone() as f64).unwrap_or(100.0) * time_to_bod;

    let mut distance_remaining = dist_to_bod as i32;
    let mut next_wp: &dyn Waypoint = to;

    let mut insertion_spot = None;
    let mut bod = None;

    for i in (0..waypoints.len()).rev() {
        let wp = &waypoints[i];
        let leg_length = wp.get_loc().distance_to(&next_wp.get_loc());

        if leg_length >= distance_remaining {
            let heading = wp.get_loc().bearing_to_deg(&next_wp.get_loc());
            let bod_loc = wp.get_loc().coordinate_at(leg_length - distance_remaining, heading);
            bod = Some(SimpleWaypoint::new_bod_waypoint("BOD".to_string(), max_alt, bod_loc));
            insertion_spot = Some((i + 1).clone());
            done = true;
            break;
        }

        distance_remaining -= leg_length;
        next_wp = wp.as_ref();
    }

    if !done {
        let leg_length = from.get_loc().distance_to(&next_wp.get_loc().clone());

        if leg_length >= distance_remaining {
            let heading = from.get_loc().bearing_to_deg(&next_wp.get_loc());
            let bod_loc = from.get_loc().coordinate_at(leg_length - distance_remaining, heading);
            bod = Some(SimpleWaypoint::new_bod_waypoint("BOD".to_string(), max_alt, bod_loc));
            insertion_spot = Some(0);
        }
    }
    if let Some(i) = insertion_spot {
        waypoints.insert(i, Box::new(bod.unwrap()));
    }

}

pub fn add_toc(plan: &Plan, from: &dyn Waypoint, to: &dyn Waypoint, waypoints: &mut Vec<Box<dyn Waypoint>>, max_alt: i32) {
    let mut done = false;
    let alt_to_toc = max_alt - from.get_elevation();
    let aircraft = plan.get_aircraft();

    let time_to_toc = alt_to_toc as f64 / aircraft.as_ref().map(|a| a.get_climb_rate().clone() as f64).unwrap_or(1000.0)/ 60.0;
    let dist_to_toc = aircraft.as_ref().map(|a| a.get_climb_speed().clone() as f64).unwrap_or(120.0) * time_to_toc;

    let mut distance_remaining = dist_to_toc as i32;
    let mut prev_wp = from.clone();
    let mut insertion_spot = None;
    let mut toc = None;

    for i in 0..waypoints.len() {
        let wp = &waypoints[i];
        let leg_length = prev_wp.get_loc().distance_to(&wp.get_loc());

        if leg_length >= distance_remaining {
            let heading = prev_wp.get_loc().bearing_to_deg(&wp.get_loc());
            let toc_loc = prev_wp.get_loc().coordinate_at(distance_remaining, heading);
            toc = Some(SimpleWaypoint::new_toc_waypoint("TOC".to_string(), max_alt, toc_loc));
            insertion_spot = Some(i);
            done = true;
            break;
        }

        distance_remaining -= leg_length;
        prev_wp = wp.as_ref();
    }

    if !done {
        let leg_length = prev_wp.get_loc().distance_to(&to.get_loc());

        if leg_length >= distance_remaining {
            let heading = prev_wp.get_loc().bearing_to_deg(&to.get_loc());
            let toc_loc = prev_wp.get_loc().coordinate_at(distance_remaining, heading);
            let toc = SimpleWaypoint::new_toc_waypoint("TOC".to_string(), max_alt, toc_loc);
            waypoints.push(Box::new(toc));
        }
    }
    if let Some(i) = insertion_spot {
        waypoints.insert(i, Box::new(toc.unwrap()));
    }
}

fn calc_max_altitude(
    plan: &Plan,
    from: &dyn Waypoint,
    to: &dyn Waypoint,
    waypoints: &Vec<Box<dyn Waypoint>>,
) -> i32 {

    let mut dist = 0;
    let mut prev_wp = from.clone();

    for wp in waypoints {
        let leg_length = prev_wp.get_loc().distance_to(&wp.get_loc()) as i32;
        dist += leg_length;
        prev_wp = wp.deref();
    }

    let leg_length = prev_wp.get_loc().distance_to(&to.get_loc());
    dist += leg_length as i32;

    let mut alt = plan.get_plan_altitude();

    while calc_climb_sink_distance(&plan,to, from, alt) > dist {
        alt -= 500;
    }

    alt
}

pub fn set_elevations(
    plan: &Plan,
    from: &dyn Waypoint,
    to: &dyn Waypoint,
    waypoints: &Vec<Box<dyn Waypoint>>,
    max_alt: i32,
) {
    let mut alt = from.get_elevation() as i32;
    let mut ascent = true;
    let mut descent = false;

    let mut prev_wp = from.clone();

    for wp in waypoints {
        if wp.get_type() == WaypointType::TOC {
            ascent = false;
            alt = max_alt;
        } else if wp.get_type() == WaypointType::BOD {
            ascent = false;
            descent = true;
            alt = max_alt;
        } else if ascent {
            let distance = prev_wp.get_loc().distance_to(&wp.get_loc());
            let leg_time = distance / plan.get_aircraft().as_ref().map(|a| a.get_climb_speed()).unwrap_or(&120);
            alt += leg_time * plan.get_aircraft().as_ref().map(|a| a.get_climb_rate()).unwrap_or(&1000) * 60;

            if alt > max_alt {
                alt = max_alt;
                ascent = false;
            }

            wp.set_elevation(alt);
        } else if descent {
            let distance = prev_wp.get_loc().distance_to(&wp.get_loc());
            let leg_time = distance / plan.get_aircraft().as_ref().map(|a| a.get_sink_speed()).unwrap_or(&80);
            alt -= leg_time * plan.get_aircraft().as_ref().map(|a| a.get_sink_rate()).unwrap_or(&700) * 60;
            wp.set_elevation(alt);
        } else {
            wp.set_elevation(alt);
        }

        prev_wp = wp.deref();
    }
}

fn calc_climb_sink_distance(plan: &Plan, from: &dyn Waypoint, to: &dyn Waypoint, altitude: i32) -> i32 {
    let alt_to_toc = altitude - from.get_elevation();
    let time_to_toc = alt_to_toc / plan.get_aircraft().as_ref().map(|a| a.get_climb_rate()).unwrap_or(&1000) / 60;
    let dist_to_toc = plan.get_aircraft().as_ref().map(|a| a.get_climb_speed()).unwrap_or(&120) * time_to_toc;

    let alt_to_bod = altitude - to.get_elevation();
    let time_to_bod = alt_to_bod / plan.get_aircraft().as_ref().map(|a| a.get_sink_rate()).unwrap_or(&700) / 60;
    let dist_to_bod = plan.get_aircraft().as_ref().map(|a| a.get_sink_speed()).unwrap_or(&80) * time_to_bod;

    dist_to_toc + dist_to_bod
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
            max_leg_distance: 100,
            min_leg_distance: 25,
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
        let w1 = AirportWaypoint::new(ap, 20, false);
        let ap = make_airport_at("YPER", -32.1, 120.5);
        let w2 = AirportWaypoint::new(ap, 20, false);

        let plan = planner.make_plan(Some(w1), Some(w2));

        for wp in &plan {
            println!("WP - {}, {}", wp.get_lat_as_string(), wp.get_long_as_string())
        }
        assert_eq!(plan.len(), 15);
    }
}

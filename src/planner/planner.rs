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
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::earth;
use crate::earth::coordinate::Coordinate;
use crate::model::aircraft::Aircraft;
use crate::model::fix::Fix;
use crate::model::location::Location;
use crate::model::navaid::{Navaid, NavaidType};
use crate::model::plan::Plan;
use crate::model::sector::Sector;
use crate::model::waypoint::Waypoint;
use crate::preference::*;
use crate::util::location_filter::{Filter, RangeFilter};

pub const ARRIVAL_BEACON_RANGE: f64 = 10.0;

pub struct Planner<'a> {
    max_leg_distance: f64,
    min_leg_distance: f64,
    max_deviation: f64,
    vor_only: bool,
    vor_preferred: bool,
    plan_type: String,
    add_gps_waypoints: bool,
    add_waypoint_bias: bool,
    navaids: &'a RwLock<Vec<Arc<Navaid>>>,
    fixes: &'a RwLock<Vec<Arc<Fix>>>,
}

impl Planner<'_> {
    pub fn new() -> Self {
        let pref = manager();

        Self {
            max_leg_distance: pref.get::<f64>(MAX_LEG_LENGTH).unwrap_or(100.0),
            min_leg_distance: pref.get::<f64>(MIN_LEG_LENGTH).unwrap_or(25.0),
            max_deviation: pref.get::<f64>(MAX_DEVIATION).unwrap_or(10.0),
            vor_only: pref.get::<bool>(VOR_ONLY).unwrap_or(false),
            vor_preferred: pref.get::<bool>(VOR_PREFERED).unwrap_or(true),
            add_gps_waypoints: pref.get::<bool>(ADD_WAYPOINTS).unwrap_or(false),
            add_waypoint_bias: pref.get::<bool>(ADD_WAYPOINT_BIAS).unwrap_or(false),
            plan_type: pref
                .get::<String>(PLAN_TYPE)
                .unwrap_or(USE_RADIO_BEACONS.to_string()),
            navaids: earth::get_earth_model().get_navaids(),
            fixes: earth::get_earth_model().get_fixes(),
        }
    }
    pub(crate) fn make_plan(&self, sector: &Sector) -> Vec<Waypoint> {

        let mut plan: Vec<Waypoint> = Vec::new();

        let from = sector.get_start();
        let to = sector.get_end();

        if let Some(from) = from {
            if let Some(to) = to {

                if self.plan_type == USE_RADIO_BEACONS {

                    let mut prev_wp = from.clone();
                    // add all the manually added waypoints into the plan
                    let old_wps = sector.get_waypoints();
                    for wp in old_wps.iter() {
                        if *wp.is_locked() {
                            self.add_navaids_between(&prev_wp, &wp.clone(), &mut plan);
                            plan.push(wp.clone());
                            prev_wp = wp.clone();
                        }
                    }

                    if let Some(wp) = self.get_arrival_beacon(&to) {
                        self.add_navaids_between(&prev_wp, &wp, &mut plan);
                        plan.push(wp);
                    } else {
                        self.add_navaids_between(&prev_wp, &to.clone(), &mut plan);
                    }
                    if self.add_gps_waypoints {
                        self.add_waypoints(&from, &to, &mut plan);
                    }

                } else if self.plan_type == USE_FIXES {
                    let mut prev_wp = from.clone();
                    // add all the manually added waypoints into the plan
                    let old_wps = sector.get_waypoints();
                    for wp in old_wps.iter() {
                        if *wp.is_locked() {
                            self.add_fixes_between(&prev_wp, &wp.clone(), &mut plan);
                            plan.push(wp.clone());
                            prev_wp = wp.clone();
                        }
                    }
                    self.add_fixes_between(&prev_wp, &to, &mut plan);
                    if self.add_gps_waypoints {
                        self.add_waypoints(&from, &to, &mut plan);
                    }
                } else {
                    self.add_waypoints(&from, &to, &mut plan);
                }
            }
        }
        plan
    }

    fn get_arrival_beacon(&self, to: &Waypoint) -> Option<Waypoint> {
        if let Some(arrival_beacon) = self.get_navaid_nearest(to.get_loc(), ARRIVAL_BEACON_RANGE) {
            Some(Waypoint::Navaid {
                navaid: arrival_beacon.clone(),
                elevation: Cell::new(0),
                locked: false,
            })
        } else {
            None
        }
    }

    fn add_navaids_between(&self, from: &Waypoint, to: &Waypoint, plan: &mut Vec<Waypoint>) {
        let distance = from.get_loc().distance_to(to.get_loc());

        if distance < self.max_leg_distance {
            return;
        }

        let heading = from.get_loc().bearing_to(to.get_loc()).to_degrees();
        let midpoint = from.get_loc().coordinate_at(distance / 2.0, heading);

        if let Some(mid_nav_aid) =
            self.get_navaid_nearest_midpoint(from.get_loc(), to.get_loc(), &midpoint)
        {
            let wp = Waypoint::Navaid {
                navaid: mid_nav_aid,
                elevation: Cell::new(0),
                locked: false,
            };
            let save_wp = wp.clone();

            self.add_navaids_between(from, &wp, plan);
            plan.push(wp);
            self.add_navaids_between(&save_wp, to, plan);
        }
    }

    fn add_fixes_between(&self, from: &Waypoint, to: &Waypoint, plan: &mut Vec<Waypoint>) {
        let distance = from.get_loc().distance_to(to.get_loc());

        if distance < self.max_leg_distance {
            return;
        }

        let heading = from.get_loc().bearing_to(to.get_loc()).to_degrees();
        let midpoint = from.get_loc().coordinate_at(distance / 2.0, heading);

        if let Some(mid_fix_aid) =
            self.get_fix_nearest_midpoint(from.get_loc(), to.get_loc(), &midpoint)
        {
            let wp = Waypoint::Fix {
                fix: mid_fix_aid,
                elevation: Cell::new(0),
                locked: false,
            };
            let save_wp = wp.clone();
            self.add_fixes_between(from, &wp, plan);
            plan.push(wp);

            self.add_fixes_between(&save_wp, to, plan);
        }
    }

    fn get_navaid_nearest_midpoint(
        &self,
        from: &Coordinate,
        to: &Coordinate,
        midpoint: &Coordinate,
    ) -> Option<Arc<Navaid>> {
        let leg_distance = from.distance_to(to);
        let heading_from = from.bearing_to_deg(midpoint);
        let heading_to = midpoint.bearing_to_deg(to);

        let range = leg_distance / 2.0; // - _min_leg_distance;

        let near_aids = self.get_navaids_near(self.navaids, midpoint, range);
        let mut best_loc: Option<Arc<Navaid>> = None;
        let mut best_ndb: Option<Arc<Navaid>> = None;
        let mut nearest = 100000.0;
        let mut nearest_ndb = 100000.0;

        for navaid in near_aids {
            if self.vor_only && navaid.get_type() != NavaidType::Vor {
                continue;
            }

            let deviation_to =
                self.get_deviation(heading_from, from.bearing_to_deg(navaid.get_loc()));
            let deviation_from =
                self.get_deviation(heading_to, navaid.get_loc().bearing_to_deg(to));

            if deviation_to > self.max_deviation || deviation_from > self.max_deviation {
                continue;
            }

            if self.vor_preferred && navaid.get_type() != NavaidType::Vor {
                if midpoint.distance_to(navaid.get_loc()) < nearest_ndb
                    && from.distance_to(navaid.get_loc()) > self.min_leg_distance
                    && to.distance_to(navaid.get_loc()) > self.min_leg_distance
                {
                    best_ndb = Some(navaid.clone());
                    nearest_ndb = midpoint.distance_to(navaid.get_loc());
                }
            } else {
                if midpoint.distance_to(navaid.get_loc()) < nearest
                    && from.distance_to(navaid.get_loc()) > self.min_leg_distance
                    && to.distance_to(navaid.get_loc()) > self.min_leg_distance
                {
                    best_loc = Some(navaid.clone());
                    nearest = midpoint.distance_to(navaid.get_loc());
                }
            }
        }

        if best_loc.is_none() && best_ndb.is_some() {
            best_loc = best_ndb;
        }

        best_loc
    }

    fn get_navaid_nearest(&self, coord: &Coordinate, max_range: f64) -> Option<Arc<Navaid>> {
        let near_aids = self.get_navaids_near(self.navaids, coord, max_range);

        let mut best_loc: Option<Arc<Navaid>> = None;
        let mut best_ndb: Option<Arc<Navaid>> = None;
        let mut nearest = 100000.0;
        let mut nearest_ndb = 100000.0;

        for navaid in near_aids {
            if self.vor_only && navaid.get_type() != NavaidType::Vor {
                continue;
            }

            if self.vor_preferred && navaid.get_type() != NavaidType::Vor {
                if coord.distance_to(navaid.get_loc()) < nearest_ndb {
                    best_ndb = Some(navaid.clone());
                    nearest_ndb = coord.distance_to(navaid.get_loc());
                }
            } else {
                if coord.distance_to(navaid.get_loc()) < nearest {
                    best_loc = Some(navaid.clone());
                    nearest = coord.distance_to(navaid.get_loc());
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
    ) -> Option<Arc<Fix>> {
        let leg_distance = from.distance_to(to);
        let heading_from = from.bearing_to_deg(midpoint);
        let heading_to = midpoint.bearing_to_deg(to);

        let range = leg_distance / 2.0; // - _min_leg_distance;

        let near_aids = self.get_fixes_near(self.fixes, midpoint, range);
        let mut best_loc: Option<Arc<Fix>> = None;
        let mut nearest = 100000.0;

        for fix in near_aids {
            let deviation_to =
                self.get_deviation(heading_from, from.bearing_to_deg(fix.get_loc()));
            let deviation_from = self.get_deviation(heading_to, fix.get_loc().bearing_to_deg(to));

            if deviation_to > self.max_deviation || deviation_from > self.max_deviation {
                continue;
            }

            if midpoint.distance_to(fix.get_loc()) < nearest
                && from.distance_to(fix.get_loc()) > self.min_leg_distance
                && to.distance_to(fix.get_loc()) > self.min_leg_distance
            {
                best_loc = Some(fix.clone());
                nearest = midpoint.distance_to(fix.get_loc());
            }
        }

        best_loc
    }

    #[allow(dead_code)]
    fn get_fix_nearest(&self, coord: &Coordinate, max_range: f64) -> Option<Arc<Fix>> {
        let near_aids = self.get_fixes_near(self.fixes, coord, max_range);

        let mut best_loc: Option<Arc<Fix>> = None;
        let mut nearest = 100000.0;

        for fix in near_aids {
            if coord.distance_to(fix.get_loc()) < nearest {
                best_loc = Some(fix.clone());
                nearest = coord.distance_to(fix.get_loc());
            }
        }
        best_loc
    }

    #[allow(dead_code)]
    fn add_waypoints_between(&self, from: &Waypoint, to: &Waypoint, plan: &mut Vec<Waypoint>) {
        let distance = from.get_loc().distance_to(to.get_loc());

        if distance < self.max_leg_distance {
            return;
        }

        let heading = from.get_loc().bearing_to(to.get_loc()).to_degrees();
        let midpoint = from.get_loc().coordinate_at(distance / 2.0, heading);

        let wp = Waypoint::Simple {
            loc: midpoint,
            elevation: Cell::new(0),
            locked: false,
        };
        let save_wp = wp.clone();

        self.add_waypoints_between(from, &wp, plan);
        plan.push(wp);

        self.add_waypoints_between(&save_wp, to, plan);
    }

    fn add_waypoints(&self, from: &Waypoint, to: &Waypoint, plan: &mut Vec<Waypoint>) {
        // Walk the legs and find those that are over the wished-for interval
        let max_leg_interval: f64 = if self.add_waypoint_bias {
            self.max_leg_distance * 0.75
        } else {
            self.max_leg_distance * 1.25
        };

        // Make a copy of the waypoints in the old plan
        let mut prev_wp = from.clone();
        let mut extra: usize = 0;
        let orig_len = plan.len();
        for i in 0..orig_len {
            let wp = plan[i+extra].clone();
            let leg_length = prev_wp.get_loc().distance_to(wp.get_loc());
            if leg_length >= max_leg_interval {
                extra += self.add_waypoints_to_leg(&prev_wp, &wp, plan, i+extra, leg_length);
            }
            prev_wp = wp;
        }
        // Try for the final leg
        let leg_length = prev_wp.get_loc().distance_to(to.get_loc());
        if leg_length >= max_leg_interval {
            self.add_waypoints_to_leg(&prev_wp, to, plan, plan.len(), leg_length);
        }
    }

    fn add_waypoints_to_leg(
        &self,
        prev_wp: &Waypoint,
        to: &Waypoint,
        plan: &mut Vec<Waypoint>,
        i: usize,
        leg_length: f64,
    ) -> usize {
        let additional_points = leg_length / self.max_leg_distance;
        let new_leg_count = if self.add_waypoint_bias && (additional_points.fract() > 0.2) {
            additional_points.ceil()
        } else {
            additional_points.floor()
        } as usize;
        let interval = leg_length / new_leg_count as f64;

        let mut last_wp = prev_wp.clone();

        let extra_points = new_leg_count - 1;
        for a_pos in 0..extra_points {
            let heading = last_wp.get_loc().bearing_to_deg(to.get_loc());
            let x_loc = last_wp.get_loc().coordinate_at(interval, heading);
            let wp = Waypoint::Simple {
                loc: x_loc,
                elevation: Cell::new(0),
                locked: false,
            };
            let save_wp = wp.clone();
            plan.insert(i + a_pos, wp);
            last_wp = save_wp;
        }
        extra_points
    }

    fn get_navaids_near(
        &self,
        locations: &RwLock<Vec<Arc<Navaid>>>,
        point: &Coordinate,
        range: f64,
    ) -> Vec<Arc<Navaid>> {
        let mut near_locations: Vec<Arc<Navaid>> = Vec::new();

        if let Some(filter) = RangeFilter::new(*point.get_latitude(), *point.get_longitude(), range)
        {
            let guard = locations.read().unwrap();
            let locations = guard.deref();
            let near_navaids: Vec<&Arc<Navaid>> = locations
                .iter()
                .filter(move |loc| filter.filter(&***loc))
                .collect();

            for navaid in near_navaids {
                near_locations.push(navaid.clone());
            }
        }
        near_locations
    }

    fn get_fixes_near(
        &self,
        locations: &RwLock<Vec<Arc<Fix>>>,
        point: &Coordinate,
        range: f64,
    ) -> Vec<Arc<Fix>> {
        let mut near_locations: Vec<Arc<Fix>> = Vec::new();

        if let Some(filter) = RangeFilter::new(*point.get_latitude(), *point.get_longitude(), range)
        {
            let guard = locations.read().unwrap();
            let locations = guard.deref();
            let near_fixes: Vec<&Arc<Fix>> = locations
                .iter()
                .filter(move |loc| filter.filter(&***loc))
                .collect();

            for fix in near_fixes {
                near_locations.push(fix.clone());
            }
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

    pub fn recalc_plan_elevations(&self, plan: &mut Plan) {

        let aircraft = plan.get_aircraft().clone();
        let altitude = plan.get_plan_altitude();

        for sector in plan.get_sectors_mut() {

            if sector.get_start().is_none() || sector.get_end().is_none() {
                continue;
            }
            let start_wp =  &sector.get_start().unwrap();
            let end_wp = &sector.get_end().unwrap();

            // Remove the previous top of climb and beginning of descent
            let waypoints = sector.get_waypoints_mut();
            waypoints.retain(|wp| !matches!(wp, Waypoint::Toc { .. } | Waypoint::Bod { .. }));


            let max_alt = calc_max_altitude(
                &aircraft,
                altitude,
                &start_wp,
                &end_wp,
                waypoints,
            );

            add_toc(
                &aircraft,
                &start_wp,
                &end_wp,
                waypoints,
                max_alt,
            );

            add_bod(
                &aircraft,
                &start_wp,
                &end_wp,
                waypoints,
                max_alt,
            );

            set_elevations(
                &aircraft,
                &start_wp,
                &end_wp,
                waypoints,
                max_alt,
            );
        }
    }
}

pub fn add_bod(
    aircraft: &Option<Arc<Aircraft>>,
    from: &Waypoint,
    to: &Waypoint,
    waypoints: &mut Vec<Waypoint>,
    max_alt: i32,
) {
    let mut done = false;
    let alt_to_bod = max_alt - to.get_elevation();
    let time_to_bod = alt_to_bod as f64
        / aircraft
            .as_ref()
            .map(|a| *a.get_sink_rate() as f64)
            .unwrap_or(500.0)
        / 60.0;
    let dist_to_bod = aircraft
        .as_ref()
        .map(|a| *a.get_sink_speed() as f64)
        .unwrap_or(100.0)
        * time_to_bod;

    let mut distance_remaining = dist_to_bod;
    let mut next_wp: &Waypoint = to;

    let mut insertion_spot = None;
    let mut bod = None;

    for i in (0..waypoints.len()).rev() {
        let wp = &waypoints[i];
        let leg_length = wp.get_loc().distance_to(next_wp.get_loc());

        if leg_length >= distance_remaining {
            let heading = wp.get_loc().bearing_to_deg(next_wp.get_loc());
            let bod_loc = wp
                .get_loc()
                .coordinate_at(leg_length - distance_remaining, heading);
            bod = Some(Waypoint::Bod {
                loc: bod_loc,
                elevation: Cell::new(max_alt),
                locked: false,
            });
            insertion_spot = Some(i + 1);
            done = true;
            break;
        }

        distance_remaining -= leg_length;
        next_wp = wp;
    }

    if !done {
        let leg_length = from.get_loc().distance_to(&next_wp.get_loc().clone());

        if leg_length >= distance_remaining {
            let heading = from.get_loc().bearing_to_deg(next_wp.get_loc());
            let bod_loc = from
                .get_loc()
                .coordinate_at(leg_length - distance_remaining, heading);
            bod = Some(Waypoint::Bod {
                loc: bod_loc,
                elevation: Cell::new(max_alt),
                locked: false,
            });
            insertion_spot = Some(0);
        }
    }
    if let Some(i) = insertion_spot {
        if let Some(wp) = bod {
            waypoints.insert(i, wp);
        }
    }
}

pub fn add_toc(
    aircraft: &Option<Arc<Aircraft>>,
    from: &Waypoint,
    to: &Waypoint,
    waypoints: &mut Vec<Waypoint>,
    max_alt: i32,
) {
    let mut done = false;
    let alt_to_toc = max_alt - from.get_elevation();

    let time_to_toc = alt_to_toc as f64
        / aircraft
            .as_ref()
            .map(|a| *a.get_climb_rate() as f64)
            .unwrap_or(1000.0)
        / 60.0;
    let dist_to_toc = aircraft
        .as_ref()
        .map(|a| *a.get_climb_speed() as f64)
        .unwrap_or(120.0)
        * time_to_toc;

    let mut distance_remaining = dist_to_toc;
    let mut prev_wp = from;
    let mut insertion_spot = None;
    let mut toc = None;

    for i in 0..waypoints.len() {
        let wp = &waypoints[i];
        let leg_length = prev_wp.get_loc().distance_to(wp.get_loc());

        if leg_length >= distance_remaining {
            let heading = prev_wp.get_loc().bearing_to_deg(wp.get_loc());
            let toc_loc = prev_wp.get_loc().coordinate_at(distance_remaining, heading);
            toc = Some(Waypoint::Toc {
                loc: toc_loc,
                elevation: Cell::new(max_alt),
                locked: false,
            });
            insertion_spot = Some(i);
            done = true;
            break;
        }

        distance_remaining -= leg_length;
        prev_wp = wp;
    }

    if !done {
        let leg_length = prev_wp.get_loc().distance_to(to.get_loc());

        if leg_length >= distance_remaining {
            let heading = prev_wp.get_loc().bearing_to_deg(to.get_loc());
            let toc_loc = prev_wp.get_loc().coordinate_at(distance_remaining, heading);
            let wp = Waypoint::Toc {
                loc: toc_loc,
                elevation: Cell::new(max_alt),
                locked: false,
            };
            waypoints.push(wp);
        }
    }
    if let Some(i) = insertion_spot {
        if let Some(wp) = toc {
            waypoints.insert(i, wp);
        }
    }
}

fn calc_max_altitude(
    aircraft: &Option<Arc<Aircraft>>,
    max_altitude: i32,
    from: &Waypoint,
    to: &Waypoint,
    waypoints: &Vec<Waypoint>,
) -> i32 {
    let mut dist = 0.0;
    let mut prev_wp = from;

    for wp in waypoints {
        let leg_length = prev_wp.get_loc().distance_to(wp.get_loc());
        dist += leg_length;
        prev_wp = wp;
    }

    let leg_length = prev_wp.get_loc().distance_to(to.get_loc());
    dist += leg_length;

    let mut alt = max_altitude;

    while calc_climb_sink_distance(&aircraft, to, from, alt) > dist {
        alt -= 500;
    }

    alt
}

pub fn set_elevations(
    aircraft: &Option<Arc<Aircraft>>,
    from: &Waypoint,
    _to: &Waypoint,
    waypoints: &Vec<Waypoint>,
    max_alt: i32,
) {
    let mut alt = from.get_elevation();
    let mut ascent = true;
    let mut descent = false;

    let mut prev_wp = from;

    for wp in waypoints {
        match wp {
            Waypoint::Toc { .. } => {
                ascent = false;
                alt = max_alt;
            }
            Waypoint::Bod { .. } => {
                ascent = false;
                descent = true;
                alt = max_alt;
            }
            _ => {
                if ascent {
                    let distance = prev_wp.get_loc().distance_to(wp.get_loc());
                    let leg_time = distance
                        / aircraft.as_ref()
                            .map(|a| *a.get_climb_speed() as f64)
                            .unwrap_or(120.0);
                    alt += (leg_time
                        * aircraft.as_ref()
                            .map(|a| *a.get_climb_rate() as f64)
                            .unwrap_or(1000.0)
                        * 60.0) as i32;

                    if alt > max_alt {
                        alt = max_alt;
                        ascent = false;
                    }

                    wp.set_elevation(&alt);
                } else if descent {
                    let distance = prev_wp.get_loc().distance_to(wp.get_loc());
                    let leg_time = distance
                        / aircraft.as_ref()
                            .map(|a| *a.get_sink_speed() as f64)
                            .unwrap_or(80.0);
                    alt -= (leg_time
                        * aircraft.as_ref()
                            .map(|a| *a.get_sink_rate() as f64)
                            .unwrap_or(700.0)
                        * 60.0) as i32;
                    wp.set_elevation(&alt);
                } else {
                    wp.set_elevation(&alt);
                }
            }
        }
        prev_wp = wp;
    }
}

fn calc_climb_sink_distance(aircraft: &Option<Arc<Aircraft>>, from: &Waypoint, to: &Waypoint, altitude: i32) -> f64 {
    let alt_to_toc = (altitude - from.get_elevation()) as f64;
    let time_to_toc = alt_to_toc
        / aircraft.as_ref()
            .map(|a| *a.get_climb_rate() as f64)
            .unwrap_or(1000.0)
        / 60.0;
    let dist_to_toc = aircraft.as_ref()
        .map(|a| *a.get_climb_speed() as f64)
        .unwrap_or(120.0)
        * time_to_toc;

    let alt_to_bod = (altitude - to.get_elevation()) as f64;
    let time_to_bod = alt_to_bod
        / aircraft.as_ref()
            .map(|a| *a.get_sink_rate() as f64)
            .unwrap_or(700.0)
        / 60.0;
    let dist_to_bod = aircraft.as_ref()
        .map(|a| *a.get_sink_speed() as f64)
        .unwrap_or(80.0)
        * time_to_bod;

    dist_to_toc + dist_to_bod
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use crate::model::sector::Sector;

    use crate::model::test_utils::tests::make_airport_at;
    use crate::preference::USE_GPS;

    use super::Planner;

    #[test]
    fn test_with_gps() {
        let planner = Planner {
            max_leg_distance: 100.0,
            min_leg_distance: 25.0,
            max_deviation: 10.0,
            vor_only: false,
            vor_preferred: true,
            add_gps_waypoints: true,
            add_waypoint_bias: false,
            plan_type: USE_GPS.to_string(),
            navaids: &Arc::new(RwLock::new(Vec::new())),
            fixes: &Arc::new(RwLock::new(Vec::new())),
        };

        let ap1 = make_airport_at("YSSY", -34.0, 151.0);
        let ap2 = make_airport_at("YPER", -32.1, 120.5);

        let mut sector = Sector::new();
        sector.set_start(Some(ap1));
        sector.set_end(Some(ap2));
        let plan = planner.make_plan(&mut sector);


        for wp in &plan {
            println!(
                "WP - {}, {}",
                wp.get_lat_as_string(),
                wp.get_long_as_string()
            )
        }
        assert_eq!(plan.len(), 14);
    }
}

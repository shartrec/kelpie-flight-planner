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
use crate::util::location_filter::{AndFilter, DeviationFilter, Filter, InverseRangeFilter, RangeFilter, VorFilter};

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
            vor_preferred: pref.get::<bool>(VOR_PREFFERED).unwrap_or(true),
            add_gps_waypoints: pref.get::<bool>(ADD_WAYPOINTS).unwrap_or(false),
            add_waypoint_bias: pref.get::<bool>(ADD_WAYPOINT_BIAS).unwrap_or(true),
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
        self.get_navaid_nearest(to.get_loc(), ARRIVAL_BEACON_RANGE). map(|arrival_beacon| {
            Waypoint::Navaid {
                navaid: arrival_beacon.clone(),
                elevation: Cell::new(0),
                locked: false,
            }
        })
    }

    fn add_navaids_between(&self, from: &Waypoint, to: &Waypoint, plan: &mut Vec<Waypoint>) {
        let distance = from.get_loc().distance_to(to.get_loc());

        if distance < self.max_leg_distance {
            return;
        }

        let midpoint = Coordinate::midpoint(from.get_loc(), to.get_loc());

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

        let midpoint = Coordinate::midpoint(from.get_loc(), to.get_loc());

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


        let df = DeviationFilter::new(from.clone(), to.clone(), heading_from, heading_to, self.max_deviation);
        let rf = RangeFilter::new(midpoint.clone(), range);
        let irf1 = InverseRangeFilter::new(from.clone(), self.min_leg_distance);
        let irf2 = InverseRangeFilter::new(to.clone(), self.min_leg_distance);

        // Add the filters, putting the most discriminating ones first
        let mut filter = AndFilter::new();
        filter.add(Box::new(rf));
        filter.add(Box::new(df));
        filter.add(Box::new(irf1));
        filter.add(Box::new(irf2));
        if self.vor_only {
            let vf = VorFilter::new();
            filter.add(Box::new(vf));
        }

        self.get_nearest_navaids_with_filter(midpoint, Box::new(filter))
    }

    fn get_navaid_nearest(&self, coord: &Coordinate, max_range: f64) -> Option<Arc<Navaid>> {

        let mut filter = AndFilter::new();
        if self.vor_only {
            let vf = VorFilter::new();
            filter.add(Box::new(vf));
        }
        let rf = RangeFilter::new(coord.clone(), max_range);
        filter.add(Box::new(rf));

        self.get_nearest_navaids_with_filter(coord, Box::new(filter))
    }

    fn get_nearest_navaids_with_filter(&self, midpoint: &Coordinate, filter: Box<dyn Filter>) -> Option<Arc<Navaid>> {
        let binding = self.navaids
            .read()
            .unwrap();
        let near_aids = binding
            .iter()
            .filter(|loc| filter.filter(&***loc));

        let mut best_loc: Option<Arc<Navaid>> = None;
        let mut best_vor: Option<Arc<Navaid>> = None;
        let mut nearest = 100000.0;
        let mut nearest_vor = 100000.0;
        for navaid in near_aids {
            if self.vor_preferred && navaid.get_type() == NavaidType::Vor {
                if midpoint.distance_to(navaid.get_loc()) < nearest_vor {
                    best_vor = Some(navaid.clone());
                    nearest_vor = midpoint.distance_to(navaid.get_loc());
                }
            }
            if midpoint.distance_to(navaid.get_loc()) < nearest {
                best_loc = Some(navaid.clone());
                nearest = midpoint.distance_to(navaid.get_loc());
            }
        }

        if best_vor.is_some() {
            best_loc = best_vor;
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

        let df = DeviationFilter::new(from.clone(), to.clone(), heading_from, heading_to, self.max_deviation);
        let rf = RangeFilter::new(midpoint.clone(), range);
        let irf1 = InverseRangeFilter::new(from.clone(), self.min_leg_distance);
        let irf2 = InverseRangeFilter::new(to.clone(), self.min_leg_distance);

        let mut filter = AndFilter::new();
        filter.add(Box::new(rf));
        filter.add(Box::new(df));
        filter.add(Box::new(irf1));
        filter.add(Box::new(irf2));

        self.get_fix_nearest_with_filter(midpoint, Box::new(filter))
    }

    #[allow(dead_code)]
    fn get_fix_nearest(&self, coord: &Coordinate, max_range: f64) -> Option<Arc<Fix>> {

        let filter = RangeFilter::new(coord.clone(), max_range);

        self.get_fix_nearest_with_filter(coord, Box::new(filter))
    }

    fn get_fix_nearest_with_filter(&self, coord: &Coordinate, f: Box<dyn Filter>) -> Option<Arc<Fix>> {
        let binding = self.fixes
            .read()
            .unwrap();
        let near_aids = binding
            .iter()
            .filter(|loc| f.filter(&***loc));

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
            let wp = plan[i + extra].clone();
            let leg_length = prev_wp.get_loc().distance_to(wp.get_loc());
            if leg_length >= max_leg_interval {
                extra += self.add_waypoints_to_leg(&prev_wp, &wp, plan, i + extra, leg_length);
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


    pub fn recalc_plan_elevations(&self, plan: &mut Plan) {
        let aircraft = plan.get_aircraft().clone();
        let altitude = plan.get_plan_altitude();

        for sector in plan.get_sectors_mut() {
            if sector.borrow().get_start().is_none() || sector.borrow().get_end().is_none() {
                continue;
            }
            let start_wp = &sector.borrow().get_start().unwrap();
            let end_wp = &sector.borrow().get_end().unwrap();

            // Remove the previous top of climb and beginning of descent
            let mut ref_mut = sector.borrow_mut();
            let waypoints = ref_mut.get_waypoints_mut();
            waypoints.retain(|wp| !matches!(wp, Waypoint::Toc { .. } | Waypoint::Bod { .. }));


            let max_alt = calc_max_altitude(
                &aircraft,
                altitude,
                start_wp,
                end_wp,
                waypoints,
            );

            add_toc(
                &aircraft,
                start_wp,
                end_wp,
                waypoints,
                max_alt,
            );

            add_bod(
                &aircraft,
                start_wp,
                end_wp,
                waypoints,
                max_alt,
            );

            set_elevations(
                &aircraft,
                start_wp,
                end_wp,
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

    for (i, wp) in waypoints.iter().enumerate() {
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
    waypoints: &[Waypoint],
) -> i32 {
    let mut dist = 0.0;
    let mut prev_wp = from;

    waypoints.iter().for_each(|wp| {
        let leg_length = prev_wp.get_loc().distance_to(wp.get_loc());
        dist += leg_length;
        prev_wp = wp;
    });

    let leg_length = prev_wp.get_loc().distance_to(to.get_loc());
    dist += leg_length;

    let mut alt = max_altitude;

    while calc_climb_sink_distance(aircraft, to, from, alt) > dist {
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

    let (climb_rate, climb_speed, sink_rate, sink_speed) = aircraft.as_ref()
        .map(|a| (
            *a.get_climb_rate() as f64, *a.get_climb_speed() as f64,
            *a.get_sink_rate() as f64, *a.get_sink_speed() as f64,
        ))
        .unwrap_or((1000.0, 120.0, 700.0, 80.0));

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
                    let leg_time = distance / climb_speed;
                    alt += (leg_time * climb_rate * 60.0) as i32;

                    if alt > max_alt {
                        alt = max_alt;
                        ascent = false;
                    }
                    wp.set_elevation(&alt);
                } else if descent {
                    let distance = prev_wp.get_loc().distance_to(wp.get_loc());
                    let leg_time = distance / sink_speed;
                    alt -= (leg_time * sink_rate * 60.0) as i32;
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
    let (climb_rate, climb_speed, sink_rate, sink_speed) = aircraft.as_ref()
        .map(|a| (
            *a.get_climb_rate() as f64, *a.get_climb_speed() as f64,
            *a.get_sink_rate() as f64, *a.get_sink_speed() as f64,
        ))
        .unwrap_or((1000.0, 120.0, 700.0, 80.0));

    let alt_to_toc = (altitude - from.get_elevation()) as f64;
    let time_to_toc = alt_to_toc / climb_rate / 60.0;
    let dist_to_toc = time_to_toc * climb_speed;

    let alt_to_bod = (altitude - to.get_elevation()) as f64;
    let time_to_bod = alt_to_bod / sink_rate / 60.0;
    let dist_to_bod = time_to_bod * sink_speed;

    dist_to_toc + dist_to_bod
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::{Arc, RwLock};
    use crate::earth::coordinate::Coordinate;
    use crate::model::fix::Fix;
    use crate::model::navaid::{Navaid, NavaidType};
    use crate::model::sector::Sector;
    use crate::model::test_utils::tests::make_airport_at;
    use crate::model::waypoint::Waypoint;
    use crate::preference::{USE_FIXES, USE_GPS, USE_RADIO_BEACONS};

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
        let plan = planner.make_plan(&sector);


        for wp in &plan {
            println!(
                "WP - {}, {}",
                wp.get_lat_as_string(),
                wp.get_long_as_string()
            )
        }
        assert_eq!(plan.len(), 14);
    }

    #[test]
    fn make_plan_with_radio_beacons() {
        // Add navaids to the earth module
        let navaid1 = Arc::new(Navaid::new("NAVAID1".to_string(), NavaidType::Vor, -33.0, 140.0, 10, 10., 10, "10".to_string(), "BIN".to_string()));
        let navaid2 = Arc::new(Navaid::new("NAVAID2".to_string(), NavaidType::Vor, -33.5, 130.0, 10, 10., 10, "10".to_string(), "GLB".to_string()));
        let navaid3 = Arc::new(Navaid::new("NAVAID3".to_string(), NavaidType::Vor, -33.8, 125.0, 10, 10., 10, "10".to_string(), "WAL".to_string()));
        let navaids = &Arc::new(RwLock::new(vec![navaid1.clone(), navaid2.clone(), navaid3.clone()]));

        let planner = Planner {
            max_leg_distance: 100.0,
            min_leg_distance: 25.0,
            max_deviation: 10.0,
            vor_only: false,
            vor_preferred: true,
            add_gps_waypoints: false,
            add_waypoint_bias: false,
            plan_type: USE_RADIO_BEACONS.to_string(),
            navaids,
            fixes: &Arc::new(RwLock::new(Vec::new())),
        };

        let ap1 = make_airport_at("YSSY", -34.0, 151.0);
        let ap2 = make_airport_at("YPER", -32.1, 120.5);

        let mut sector = Sector::new();
        sector.set_start(Some(ap1));
        sector.set_end(Some(ap2));
        let plan = planner.make_plan(&sector);

        assert!(!plan.is_empty());
    }

    #[test]
    fn make_plan_with_fixes() {


        // Add fixes to the earth module
        let fix1 = Arc::new(Fix::new("FIX1".to_string(), -33.0, 140.0));
        let fix2 = Arc::new(Fix::new("FIX2".to_string(), -33.5, 130.0));
        let fix3 = Arc::new(Fix::new("FIX3".to_string(), -33.8, 125.0));
        let fixes = &Arc::new(RwLock::new(vec![fix1.clone(), fix2.clone(), fix3.clone()]));

        let planner = Planner {
            max_leg_distance: 100.0,
            min_leg_distance: 25.0,
            max_deviation: 10.0,
            vor_only: false,
            vor_preferred: true,
            add_gps_waypoints: false,
            add_waypoint_bias: false,
            plan_type: USE_FIXES.to_string(),
            navaids: &Arc::new(RwLock::new(Vec::new())),
            fixes,
        };

        let ap1 = make_airport_at("YSSY", -34.0, 151.0);
        let ap2 = make_airport_at("YPER", -32.1, 120.5);


        let mut sector = Sector::new();
        sector.set_start(Some(ap1));
        sector.set_end(Some(ap2));
        let plan = planner.make_plan(&sector);

        assert!(!plan.is_empty());
    }

    #[test]
    fn make_plan_with_no_waypoints() {
        let planner = Planner {
            max_leg_distance: 100.0,
            min_leg_distance: 25.0,
            max_deviation: 10.0,
            vor_only: false,
            vor_preferred: true,
            add_gps_waypoints: false,
            add_waypoint_bias: false,
            plan_type: USE_RADIO_BEACONS.to_string(),
            navaids: &Arc::new(RwLock::new(Vec::new())),
            fixes: &Arc::new(RwLock::new(Vec::new())),
        };

        let ap1 = make_airport_at("YSSY", -34.0, 151.0);
        let ap2 = make_airport_at("YPER", -32.1, 120.5);

        let mut sector = Sector::new();
        sector.set_start(Some(ap1));
        sector.set_end(Some(ap2));
        let plan = planner.make_plan(&sector);

        assert_eq!(plan.len(), 0);
    }

    #[test]
    fn make_plan_with_locked_waypoints() {
        let planner = Planner {
            max_leg_distance: 100.0,
            min_leg_distance: 25.0,
            max_deviation: 10.0,
            vor_only: false,
            vor_preferred: true,
            add_gps_waypoints: false,
            add_waypoint_bias: false,
            plan_type: USE_RADIO_BEACONS.to_string(),
            navaids: &Arc::new(RwLock::new(Vec::new())),
            fixes: &Arc::new(RwLock::new(Vec::new())),
        };

        let ap1 = make_airport_at("YSSY", -34.0, 151.0);
        let ap2 = make_airport_at("YPER", -32.1, 120.5);
        let wp = Waypoint::Simple {
            loc: Coordinate::new(-33.0, 135.0),
            elevation: Cell::new(0),
            locked: true,
        };

        let mut sector = Sector::new();
        sector.set_start(Some(ap1));
        sector.set_end(Some(ap2));
        sector.add_waypoint(wp.clone());
        let plan = planner.make_plan(&sector);

        assert!(plan.contains(&wp));
    }
}

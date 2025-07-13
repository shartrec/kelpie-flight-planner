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
use crate::earth::coordinate::Coordinate;
use crate::model::plan::Plan;
use crate::preference::UNITS;
use crate::util::distance_format::DistanceFormat;
use crate::util::hour_format::HourFormat;
use std::cell::Cell;
use std::sync::Arc;

use super::airport::Airport;
use super::waypoint::Waypoint;

// #[derive(Clone)]
pub struct Sector {
    airport_start: Option<Waypoint>,
    airport_end: Option<Waypoint>,
    waypoints: Vec<Waypoint>,
    dirty: bool,
}

impl Sector {
    pub fn new() -> Self {
        Sector {
            airport_start: None,
            airport_end: None,
            waypoints: Vec::with_capacity(10),
            dirty: false,
        }
    }

    pub fn set_start(&mut self, start: Option<Arc<Airport>>) {
        self.airport_start = start.map(|a| {
            Waypoint::Airport {
                airport: a.clone(),
                locked: true,
            }
        });
        self.dirty = true;
    }

    pub fn set_end(&mut self, end: Option<Arc<Airport>>) {
        self.airport_end = end.map(|a| {
            Waypoint::Airport {
                airport: a.clone(),
                locked: true,
            }
        });
        self.dirty = true;
    }

    pub fn insert_waypoint(&mut self, index: usize, waypoint: Waypoint) {
        if index <= self.waypoints.len() {
            self.waypoints.insert(index, waypoint);
        }
        self.dirty = true;
    }

    pub fn add_waypoint(&mut self, waypoint: Waypoint) {
        self.waypoints.push(waypoint);
        self.dirty = true;
    }

    pub fn add_waypoint_optimised(&mut self, waypoint: Waypoint) {
        if self.waypoints.is_empty() {
            self.waypoints.push(waypoint);
        } else {
            // Find the nearest waypoint in the sector to this waypoint
            let mut min_distance = f64::MAX;

            let mut close_wp_index : i32 = 0;
            let mut close_wp = None;

            if let Some(airport) = &self.airport_start {
                let dist = airport.get_loc().distance_to(waypoint.get_loc());
                if dist < min_distance {
                    min_distance = dist;
                    close_wp = Some(airport);
                    close_wp_index = -1;
                }
            }
            if let Some(airport) = &self.airport_end {
                let dist = airport.get_loc().distance_to(waypoint.get_loc());
                if dist < min_distance {
                    min_distance = dist;
                    close_wp = Some(airport);
                    close_wp_index = -2;
                }
            }

            // iterate over waypoints to get closest
            for (i, wp) in self.waypoints.iter().enumerate() {
                let dist = wp.get_loc().distance_to(waypoint.get_loc());
                if dist < min_distance {
                    min_distance = dist;
                    close_wp = Some(wp);
                    close_wp_index = i as i32;
                }
            }

            // If closest is an airport then we just put it at beginning or end
            if close_wp_index == -1 {
                self.waypoints.insert(0, waypoint);
            } else if close_wp_index == -2 {
                self.waypoints.push(waypoint);
            } else if let Some(close_wp) = close_wp {

                // We need to find the shortest path when we add the new waypoint either before or after the closest waypoint
                // Get the distance from the prior waypoint to the new one
                // get the prior waypoint
                let prior_wp =
                    if close_wp_index == 0 {
                        self.get_start().unwrap_or_else(|| Waypoint::Simple {
                            loc: Coordinate::new(0.0, 0.0),
                            elevation: Cell::new(0),
                            locked: false,
                        })
                    } else {
                        self.waypoints.as_slice()[close_wp_index as usize - 1].clone()
                    };
                // get the following waypoint
                let next_wp =
                    if close_wp_index == (self.get_waypoint_count() - 1) as i32 {
                        self.get_end().unwrap_or_else(|| Waypoint::Simple {
                            loc: Coordinate::new(0.0, 0.0),
                            elevation: Cell::new(0),
                            locked: false,
                        })
                    } else {
                        self.waypoints.as_slice()[close_wp_index as usize + 1].clone()
                    };

                // now calculate the paths P_C_W_N and P_W_C_N
                let dist_pcwn = prior_wp.get_loc().distance_to(close_wp.get_loc())
                    + close_wp.get_loc().distance_to(waypoint.get_loc())
                    + waypoint.get_loc().distance_to(next_wp.get_loc());
                let dist_pwcn = prior_wp.get_loc().distance_to(waypoint.get_loc())
                    + waypoint.get_loc().distance_to(close_wp.get_loc())
                    + close_wp.get_loc().distance_to(next_wp.get_loc());

                // Place waypoint so that the path is shortest
                if dist_pcwn > dist_pwcn {
                    self.waypoints.insert(close_wp_index as usize, waypoint);
                } else {
                    self.waypoints.insert(close_wp_index as usize + 1, waypoint);
                }
            }
        }
        self.dirty = true;
    }

    pub fn add_all_waypoint(&mut self, waypoints: Vec<Waypoint>) {
        self.waypoints.clear();
        for wp in waypoints {
            self.waypoints.push(wp);
        }
        self.dirty = true;
    }

    pub fn remove_waypoint(&mut self, index: usize) -> Option<Waypoint> {
        if index < self.waypoints.len() {
            self.dirty = true;
            Some(self.waypoints.remove(index))
        } else {
            None
        }
    }

    pub fn remove_all_waypoints(&mut self) {
        self.dirty = true;
        self.waypoints.clear();
    }

    pub fn move_waypoint_up(&mut self, index: usize) {
        if index > 0 && index < self.waypoints.len() {
            self.waypoints.swap(index - 1, index);
        }
        self.dirty = true;
    }

    pub fn move_waypoint_down(&mut self, index: usize) {
        if index < self.waypoints.len() - 1 {
            self.waypoints.swap(index, index + 1);
        }
        self.dirty = true;
    }

    pub fn is_empty(&self) -> bool {
        !(self.airport_start.is_some()
            || self.airport_end.is_some()
            || self.waypoints.len() > 0)
    }

    pub fn get_name(&self) -> String {
        let n1 = match &self.airport_start {
            Some(w) => w.get_id(),
            None => "",
        };
        let n2 = match &self.airport_end {
            Some(w) => w.get_id(),
            None => "",
        };
        format!("{} --> {}", n1, n2)
    }

    pub fn get_start(&self) -> Option<Waypoint> {
        self.airport_start.clone()
    }

    pub fn get_end(&self) -> Option<Waypoint> {
        self.airport_end.clone()
    }

    pub fn get_waypoints_mut(&mut self) -> &mut Vec<Waypoint> {
        &mut self.waypoints
    }

    pub fn get_waypoints(&self) -> &Vec<Waypoint> {
        &self.waypoints
    }

    pub fn get_waypoint_count(&self) -> usize {
        self.waypoints.len()
    }

    pub fn get_waypoint(&self, pos: usize) -> Option<Waypoint> {
        Some(self.waypoints[pos].clone())
    }

    pub fn get_duration_as_string(&self, plan: &Plan) -> String {
        let time_format = HourFormat::new();
        let time = &self.get_duration(plan);
        time_format.format(time)
    }

    pub fn get_duration(&self, plan: &Plan) -> f64 {
        let d = self.waypoints
            .iter()
            .map(move |wp| plan.get_time_to(wp))
            .reduce(|acc, t| acc + t)
            .unwrap_or(0.0);
        let d1 = if let Some(end) = &self.airport_end {
            plan.get_time_to(end)
        } else {
            0.0
        };
        d + d1
    }
    pub fn get_distance_as_string(&self, plan: &Plan) -> String {
        let pref = crate::preference::manager();
        let units = pref.get::<String>(UNITS).unwrap_or("Nm".to_string());
        let distance_format = DistanceFormat::new(&units);
        let distance = &self.get_distance(plan);
        distance_format.format(distance)
    }
    pub fn get_distance(&self, plan: &Plan) -> f64 {
        let d = self.waypoints
            .iter()
            .map(move |wp| plan.get_leg_distance_to(wp))
            .reduce(|acc, t| acc + t)
            .unwrap_or(0.0);
        let d1 = if let Some(end) = &self.airport_end {
            plan.get_leg_distance_to(end)
        } else {
            0.0
        };
        d + d1
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
}

impl Default for Sector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::earth::coordinate::Coordinate;
    use crate::model::test_utils::tests::make_airport;
    use crate::model::waypoint::Waypoint;

    use super::Sector;

    #[test]
    fn test_set_start() {
        let mut s = Sector::new();
        s.set_start(Some(make_airport("YSSY")));
        assert_eq!(s.get_start().unwrap().get_id(), "YSSY");
    }

    #[test]
    fn test_set_end() {
        let mut s = Sector::new();
        s.set_end(Some(make_airport("YMLB")));
        assert_eq!(s.get_end().unwrap().get_id(), "YMLB");
    }

    #[test]
    fn test_waypoints() {
        let mut s = Sector::new();
        s.set_start(Some(make_airport("YSSY")));
        s.set_end(Some(make_airport("YMLB")));
        let w1 = Waypoint::Simple {
            loc: Coordinate::new(13.0, 111.0),
            elevation: Cell::new(10),
            locked: false,
        };
        let w2 = Waypoint::Simple {
            loc: Coordinate::new(23.0, 121.0),
            elevation: Cell::new(20),
            locked: false,
        };
        let w3 = Waypoint::Simple {
            loc: Coordinate::new(33.0, 131.0),
            elevation: Cell::new(30),
            locked: false,
        };
        let w4 = Waypoint::Simple {
            loc: Coordinate::new(43.0, 141.0),
            elevation: Cell::new(40),
            locked: false,
        };

        s.add_waypoint(w1.clone());
        s.add_waypoint(w2.clone());
        s.add_waypoint(w3.clone());
        s.insert_waypoint(1, w4.clone());

        let wps = &s.waypoints;
        assert_eq!(wps.len(), 4);
        assert_eq!(wps.get(0).unwrap().get_loc(), w1.get_loc());
        assert_eq!(wps.get(1).unwrap().get_loc(), w4.get_loc());
        assert_eq!(wps.get(2).unwrap().get_loc(), w2.get_loc());
        assert_eq!(wps.get(3).unwrap().get_loc(), w3.get_loc());

        s.remove_waypoint(2);
        let wps = &s.waypoints;
        assert_eq!(wps.len(), 3);
        assert_eq!(wps.get(0).unwrap().get_loc(), w1.get_loc());
        assert_eq!(wps.get(1).unwrap().get_loc(), w4.get_loc());
        assert_eq!(wps.get(2).unwrap().get_loc(), w3.get_loc());
    }
}

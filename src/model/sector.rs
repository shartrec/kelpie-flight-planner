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

use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::model::plan::Plan;
use crate::preference::UNITS;
use crate::util::distance_format::DistanceFormat;
use crate::util::hour_format::HourFormat;

use super::airport::Airport;
use super::waypoint::Waypoint;

#[derive(Clone)]
pub struct Sector {
    airport_start: Option<Waypoint>,
    airport_end: Option<Waypoint>,
    waypoints: Rc<RwLock<Vec<Waypoint>>>,
    dirty: bool,
}

impl Sector {
    pub fn new() -> Self {
        Sector {
            airport_start: None,
            airport_end: None,
            waypoints: Rc::new(RwLock::new(Vec::with_capacity(10))),
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
        if let Ok(mut vec) = self.waypoints.write() {
            if index <= vec.len() {
                vec.insert(index, waypoint);
            }
        }
        self.dirty = true;
    }

    pub fn add_waypoint(&mut self, waypoint: Waypoint) {
        if let Ok(mut vec) = self.waypoints.write() {
            vec.push(waypoint);
        }
        self.dirty = true;
    }

    pub fn add_waypoint_optimised(&mut self, waypoint: Waypoint) {
        if let Ok(mut vec) = self.waypoints.write() {
            if vec.len() == 0 {
                vec.push(waypoint);
            } else {
                // Find the nearest waypoint in the sector to this waypoint
                let mut min_distance = f64::MAX;
                let mut close_wp_index = 0;
                let mut close_wp = None;
                for i in 0..vec.len() {
                    let w = &vec.deref().as_slice()[i];
                    let dist = w.get_loc().distance_to(waypoint.get_loc());
                    if dist < min_distance {
                        min_distance = dist;
                        close_wp = Some(w);
                        close_wp_index = i;
                    }
                }
                if let Some(wp) = close_wp {
                    // Now if the distance from the prior wp to the one to be inserted
                    // is less that from the prior wp to the closest then insert before
                    // else insert after
                    let dist_before_cl = if close_wp_index == 0 {
                            match self.get_start() {
                                Some(airport) => airport.get_loc().distance_to(wp.get_loc()),
                                None => 0.0
                            }
                        } else {
                            let w = &vec.deref().as_slice()[close_wp_index - 1];
                            w.get_loc().distance_to(wp.get_loc())
                    };
                    let dist_before_wp = if close_wp_index == 0 {
                            match self.get_start() {
                                Some(airport) => airport.get_loc().distance_to(waypoint.get_loc()),
                                None => 0.0
                            }
                        } else {
                            let w = &vec.deref().as_slice()[close_wp_index - 1];
                            w.get_loc().distance_to(waypoint.get_loc())
                    };
                    if dist_before_wp < dist_before_cl {
                        vec.insert(close_wp_index, waypoint);
                    } else {
                        vec.insert(close_wp_index + 1, waypoint);
                    }
                } else {
                    vec.push(waypoint);
                }
            }
        }
        self.dirty = true;
    }

    pub fn add_all_waypoint(&mut self, waypoints: Vec<Waypoint>) {
        if let Ok(mut vec) = self.waypoints.write() {
            vec.clear();
            for wp in waypoints {
                vec.push(wp);
            }
        }
        self.dirty = true;
    }

    pub fn remove_waypoint(&mut self, index: usize) -> Option<Waypoint> {
        if let Ok(mut vec) = self.waypoints.write() {
            if index < vec.len() {
                self.dirty = true;
                Some(vec.remove(index))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn move_waypoint_up(&mut self, index: usize) {
        if let Ok(mut vec) = self.waypoints.write() {
            if index > 0 && index < vec.len() {
                vec.swap(index - 1, index);
            }
        }
        self.dirty = true;
    }

    pub fn move_waypoint_down(&mut self, index: usize) {
        if let Ok(mut vec) = self.waypoints.write() {
            if index < vec.len() - 1{
                vec.swap(index, index + 1);
            }
        }
        self.dirty = true;
    }

    pub fn is_empty(&self) -> bool {
        ! (self.airport_start.is_some()
            || self.airport_end.is_some()
            || self.waypoints.read().unwrap().len() > 0)
    }

    pub fn get_name(&self) -> String {
        let n1 = match &self.airport_start {
            Some(w) => w.get_id(),
            None => "",
        };
        let n2 =match &self.airport_end {
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

    pub fn get_waypoints(&self) -> &Rc<RwLock<Vec<Waypoint>>> {
        &self.waypoints
    }

    pub fn get_waypoint_count(&self) -> usize {
        self.waypoints.read().unwrap().len()
    }

    pub fn get_waypoint(&self, pos: usize) -> Option<Waypoint> {
        Some(self.waypoints.read().unwrap()[pos].clone())
    }

    pub fn get_duration_as_string(&self, plan: &Plan) -> String {
        let time_format = HourFormat::new();
        let time = &self.get_duration(plan);
        time_format.format(time)
    }

    pub fn get_duration(&self, plan: &Plan) -> f64 {
        match self.waypoints.read() {
            Ok(waypoints) => waypoints
                .iter()
                .map(move |wp| plan.get_time_to(wp))
                .reduce(|acc, t| acc + t)
                .unwrap_or(0.0),
            _ => 0.0,
        }
    }
    pub fn get_distance_as_string(&self, plan: &Plan) -> String {
        let pref = crate::preference::manager();
        let units = pref.get::<String>(UNITS).unwrap_or("Nm".to_string());
        let distance_format = DistanceFormat::new(&units);
        let distance = &self.get_distance(plan);
        distance_format.format(distance)
    }
    pub fn get_distance(&self, plan: &Plan) -> f64 {
        match self.waypoints.read() {
            Ok(waypoints) => waypoints
                .iter()
                .map(move |wp| plan.get_leg_distance_to(wp))
                .reduce(|acc, t| acc + t)
                .unwrap_or(0.0),
            _ => 0.0,
        }
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

        let wps = s.waypoints.read().unwrap();
        assert_eq!(wps.len(), 4);
        assert_eq!(wps.get(0).unwrap().get_loc(), w1.get_loc());
        assert_eq!(wps.get(1).unwrap().get_loc(), w4.get_loc());
        assert_eq!(wps.get(2).unwrap().get_loc(), w2.get_loc());
        assert_eq!(wps.get(3).unwrap().get_loc(), w3.get_loc());
        drop(wps);

        s.remove_waypoint(2);
        let wps = s.waypoints.read().unwrap();
        assert_eq!(wps.len(), 3);
        assert_eq!(wps.get(0).unwrap().get_loc(), w1.get_loc());
        assert_eq!(wps.get(1).unwrap().get_loc(), w4.get_loc());
        assert_eq!(wps.get(2).unwrap().get_loc(), w3.get_loc());
    }
}

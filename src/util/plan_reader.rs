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
use std::fs::File;
use std::path::Path;

use gtk::subclass::prelude::ObjectSubclassIsExt;
use xmltree::Element;

use crate::earth::coordinate::Coordinate;
use crate::earth::get_earth_model;
use crate::hangar::hangar::get_hangar;
use crate::model::plan::Plan;
use crate::model::sector::Sector;
use crate::model::waypoint::Waypoint;

pub fn read_plan(file_path: &Path) -> Result<Plan, String> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Err(String::from("Error reading file")),
    };
    let doc = match Element::parse(file) {
        Ok(doc) => doc,
        Err(_) => return Err(String::from("Error parsing XML")),
    };

    if doc.name != "plan" {
        return Err(String::from("Invalid XML format"));
    }

    let mut plan = Plan::new();

    if let Some(aircraft_name) = doc.attributes.get("aircraft") {
        // Assuming getAircraftManager() and getAircraftByName() functions
        let aircraft = get_hangar().imp().get(aircraft_name);
        plan.set_aircraft(&aircraft);
    }

    let sector_list = doc.children;
    for sector_element in sector_list {
        let mut sector = Sector::new();

        if let Some(from) = sector_element.as_element().unwrap().get_child("from-airport") {
            let start = get_earth_model().get_airport_by_id(from.attributes.get("id").unwrap());
            sector.set_start(start);
        }

        if let Some(to) = sector_element.as_element().unwrap().get_child("to-airport") {
            let end = get_earth_model().get_airport_by_id(to.attributes.get("id").unwrap());
            sector.set_end(end);
        }

        let waypoints = &sector_element.as_element().unwrap().children;
        for waypoint_element in waypoints {
            let e = waypoint_element.as_element().unwrap();
            if e.name != "waypoint" {
                continue;
            }

            let waypoint_type = e.attributes.get("type").unwrap().as_str();

            let wp = match waypoint_type {
                "NAVAID" => {
                    let navaid = get_earth_model()
                        .get_navaid_by_id_and_name(e.attributes.get("id").unwrap(), e.attributes.get("name").unwrap());
                    navaid.map(|n| {
                        Waypoint::Navaid { navaid: n, elevation: Cell::new(0), locked: false }
                    })
                }
                "AIRPORT" => {
                    let airport = get_earth_model().get_airport_by_id(e.attributes.get("id").unwrap());
                    airport.map(|a| {
                        Waypoint::Airport { airport: a, locked: false }
                    })
                }
                "FIX" => {
                    let navaid = get_earth_model()
                        .get_fix_by_id(e.attributes.get("id").unwrap());
                    navaid.map(|f| {
                        Waypoint::Fix { fix: f, elevation: Cell::new(0), locked: false }
                    })
                }
                "TOC" => {
                    let lat = e.attributes.get("latitude").unwrap().parse::<f64>().unwrap_or(0.0);
                    let long = e.attributes.get("longitude").unwrap().parse::<f64>().unwrap_or(0.0);
                    Some(Waypoint::Toc { loc: Coordinate::new(lat, long), elevation: Cell::new(0), locked: false })
                }
                "BOD" => {
                    let lat = e.attributes.get("latitude").unwrap().parse::<f64>().unwrap_or(0.0);
                    let long = e.attributes.get("longitude").unwrap().parse::<f64>().unwrap_or(0.0);
                    Some(Waypoint::Bod { loc: Coordinate::new(lat, long), elevation: Cell::new(0), locked: false })
                }
                "GPS" => {
                    let lat = e.attributes.get("latitude").unwrap().parse::<f64>().unwrap_or(0.0);
                    let long = e.attributes.get("longitude").unwrap().parse::<f64>().unwrap_or(0.0);
                    Some(Waypoint::Simple { loc: Coordinate::new(lat, long), elevation: Cell::new(0), locked: false })
                }
                _ => {
                    None
                }
            };
            if let Some(wp) = wp {
                let elev = e.attributes.get("elevation").unwrap().parse::<i32>().unwrap_or(0);
                wp.set_elevation(&elev);
                sector.add_waypoint(wp);
            }
        }
        plan.add_sector(sector);
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::util::plan_reader::read_plan;

    #[test]
    fn test_read_plan() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/YSSY-YBBN.fpl");

        let result = read_plan(&path);
        match result {
            Ok(_p) => println!("We have a plan"),

            Err(e) => { panic!("Failed to read plan {}", e) }
        };
    }
}
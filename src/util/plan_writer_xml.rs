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

use std::fs::File;
use std::ops::Deref;
use std::path::Path;

use xmltree::{Element, EmitterConfig, XMLNode};
use crate::model::location::Location;

use crate::model::plan::Plan;
use crate::model::waypoint::Waypoint;

pub fn write_plan(plan: &Plan, file_path: &Path) -> Result<(), String> {
    let out = match File::create(file_path) {
        Ok(file) => file,
        Err(_) => return Err(String::from("Error reading file")),
    };
    let mut plan_element = Element::new("plan");
    plan_element.attributes.insert("name".to_string(), plan.get_name());
    if let Some(aircraft) = plan.get_aircraft().deref() {
        plan_element.attributes.insert("aircraft".to_string(), aircraft.get_name().to_string());
    }
    for sector in   plan.get_sectors().deref() {
        let mut sector_element = Element::new("sector");

        let binding = sector.borrow();
        let s = binding.deref();
        if let Some(start) = s.get_start() {
            let mut from = Element::new("from-airport");
            from.attributes.insert("id".to_string(), start.get_id().to_string());
            sector_element.children.push(XMLNode::Element(from));
        }

        for wp in s.get_waypoints()
            .read()
            .expect("Can't get read lock on sectors")
            .deref() {
            let mut wp_element = Element::new("waypoint");
            wp_element.attributes.insert("name".to_string(), wp.get_name().to_string());
            wp_element.attributes.insert("type".to_string(), wp.get_type_name().to_string());
            match wp {
                Waypoint::Navaid { navaid: n, elevation: _b, locked: _c } => {
                    wp_element.attributes.insert("id".to_string(), n.get_id().to_string());
                }
                Waypoint::Airport{airport: a, locked: _b} => {
                    wp_element.attributes.insert("id".to_string(), a.get_id().to_string());
                }
                Waypoint::Fix { fix: f,elevation: _b,locked: _c} => {
                    wp_element.attributes.insert("id".to_string(), f.get_id().to_string());
                }
                _ => {
                    wp_element.attributes.insert("latitude".to_string(), format!("{:.4}", wp.get_lat()));
                    wp_element.attributes.insert("longitude".to_string(), format!("{:.4}", wp.get_long()));
                }
            }
            wp_element.attributes.insert("elevation".to_string(), format!("{}", wp.get_elevation()));
            sector_element.children.push(XMLNode::Element(wp_element));
        }

        if let Some(end) = s.get_end() {
            let mut to = Element::new("to-airport");
            to.attributes.insert("id".to_string(), end.get_id().to_string());
            sector_element.children.push(XMLNode::Element(to));
        }

        plan_element.children.push(XMLNode::Element(sector_element));
    }

    let config = EmitterConfig::new()
        .perform_indent(true);

    match plan_element.write_with_config(out, config) {
        Ok(_) => { Ok(())}
        Err(e) => {Err(e.to_string())}
    }
}

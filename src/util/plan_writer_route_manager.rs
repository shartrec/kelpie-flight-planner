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

pub fn export_plan_fg(plan: &Plan, file_path: &Path) -> Result<(), String> {
    let out = match File::create(file_path) {
        Ok(file) => file,
        Err(_) => return Err(String::from("Error reading file")),
    };
    let mut plan_element = Element::new("PropertyList");
    let mut element = Element::new("version");
    element.attributes.insert("type".to_string(), "int".to_string());
    element.children.push(XMLNode::Text("2".to_string()));
    plan_element.children.push(XMLNode::Element(element));

    let mut element = Element::new("estimated-duration-minutes");
    element.attributes.insert("type".to_string(), "int".to_string());
    let dur = format!("{:.0}", plan.get_duration() * 60.0);
    element.children.push(XMLNode::Text(dur));
    plan_element.children.push(XMLNode::Element(element));

    for sector in plan.get_sectors().deref() {
        if let Some(start) = sector.borrow().get_start() {
            if let Some(from) = make_airport(&start, true) {
                plan_element.children.push(XMLNode::Element(from));
            }
        }
        if let Some(end) = sector.borrow().get_end() {
            if let Some(to) = make_airport(&end, true) {
                plan_element.children.push(XMLNode::Element(to));
            }
        }
        if let Some(cruise) = make_cruise(plan) {
            plan_element.children.push(XMLNode::Element(cruise));
        }


        let mut route = Element::new("route"); //$NON-NLS-1$
        for (wp_ordinal, wp) in sector.borrow().get_waypoints()
            .iter().enumerate() {
            let wpt = make_waypoint(wp, wp_ordinal as i32, plan);
            route.children.push(XMLNode::Element(wpt));
        }
        plan_element.children.push(XMLNode::Element(route));
    }

    let config = EmitterConfig::new()
        .perform_indent(true);

    match plan_element.write_with_config(out, config) {
        Ok(_) => { Ok(()) }
        Err(e) => { Err(e.to_string()) }
    }
}

fn make_cruise(plan: &Plan) -> Option<Element> {
    let mut cruise = Element::new("cruise");
    let mut alt = Element::new("altitude-ft");
    alt.attributes.insert("type".to_string(), "int".to_string());
    alt.children.push(XMLNode::Text(format!("{:.0}", plan.get_max_altitude().unwrap_or(7000))));
    cruise.children.push(XMLNode::Element(alt));

    if let Some(aircraft) = plan.get_aircraft() {
        let mut spd = Element::new("knots");
        spd.attributes.insert("type".to_string(), "int".to_string());
        spd.children.push(XMLNode::Text(format!("{:.0}", aircraft.get_cruise_speed())));
        cruise.children.push(XMLNode::Element(spd));
    }

    Some(cruise)
}


fn make_airport(waypoint: &Waypoint, start: bool) -> Option<Element> {
    // Pick a runway, any runway
    let airport = waypoint.get_airport();
    let runway = airport.get_longest_runway();
    if let Some(runway) = runway {
        let mut wp = Element::new(if start { "departure" } else { "destination" });

        let mut ap = Element::new("airport");
        ap.attributes.insert("type".to_string(), "string".to_string());
        ap.children.push(XMLNode::Text(airport.get_id().to_string()));
        wp.children.push(XMLNode::Element(ap));

        let mut rw = Element::new("runway");
        rw.attributes.insert("type".to_string(), "string".to_string());
        rw.children.push(XMLNode::Text(runway.number().to_string()));
        wp.children.push(XMLNode::Element(rw));

        Some(wp)
    } else {
        None
    }
}

fn make_waypoint(waypoint: &Waypoint, wp_ordinal: i32, plan: &Plan) -> Element {
    let mut wp = Element::new("wp");

    wp.attributes.insert("n".to_string(), wp_ordinal.to_string());

    let mut typ = Element::new("type");
    typ.attributes.insert("type".to_string(), "string".to_string());
    typ.children.push(XMLNode::Text("navaid".to_string()));
    wp.children.push(XMLNode::Element(typ));

    let mut ident = Element::new("ident");
    ident.attributes.insert("type".to_string(), "string".to_string());
    ident.children.push(XMLNode::Text(waypoint.get_id().to_string()));
    wp.children.push(XMLNode::Element(ident));

    let mut restrict = Element::new("alt-restrict");
    restrict.attributes.insert("type".to_string(), "string".to_string());
    restrict.children.push(XMLNode::Text("at".to_string()));
    wp.children.push(XMLNode::Element(restrict));

    let mut elev = Element::new("altitude-ft");
    elev.attributes.insert("type".to_string(), "double".to_string());
    elev.children.push(XMLNode::Text(format!("{:.0}", waypoint.get_elevation())));
    wp.children.push(XMLNode::Element(elev));

    let mut spd = Element::new("speed");
    spd.attributes.insert("type".to_string(), "double".to_string());
    spd.children.push(XMLNode::Text(format!("{:.0}", plan.get_speed_to(waypoint))));
    wp.children.push(XMLNode::Element(spd));

    let mut lat = Element::new("lat");
    lat.attributes.insert("type".to_string(), "double".to_string());
    lat.children.push(XMLNode::Text(format!("{:.8}", waypoint.get_lat())));
    wp.children.push(XMLNode::Element(lat));

    let mut lon = Element::new("lon");
    lon.attributes.insert("type".to_string(), "double".to_string());
    lon.children.push(XMLNode::Text(format!("{:.8}", waypoint.get_long())));
    wp.children.push(XMLNode::Element(lon));

    wp
}
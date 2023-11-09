/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */

use std::fs::File;
use std::io::BufWriter;
use std::ops::Deref;
use xml::EventWriter;

use xmltree::{Element, EmitterConfig, XMLNode};

use crate::model::plan::Plan;
use crate::model::waypoint::Waypoint;
use crate::model::waypoint::Waypoint::Navaid;

pub fn write_plan(plan: &Plan, out: &mut File) -> xml::writer::Result<()> {
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
                Waypoint::Navaid { navaid: a, elevation: b, locked: c } => {
                    wp_element.attributes.insert("id".to_string(), wp.get_id().to_string());
                }
                Waypoint::Airport{airport: a, locked: b} => {
                    wp_element.attributes.insert("id".to_string(), wp.get_id().to_string());
                }
                Waypoint::Fix { fix: a,elevation: b,locked: c} => {
                    wp_element.attributes.insert("id".to_string(), wp.get_id().to_string());
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

    plan_element.write_with_config(out, config)
}

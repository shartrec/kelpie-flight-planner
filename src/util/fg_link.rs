/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
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

use std::error::Error;

use serde::Deserialize;

use crate::earth::coordinate::Coordinate;
use crate::preference::{FGFS_LINK_ENABLED, FGFS_LINK_HOST, FGFS_LINK_PORT};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct FGProperty {
    path: String,
    name: String,
    value: f64,
    #[serde(rename = "type")]
    prop_type: String,
    index: f64,
    n_children: f64,
}

#[allow(dead_code)]
impl FGProperty {
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn value(&self) -> f64 {
        self.value
    }
    pub fn prop_type(&self) -> &str {
        &self.prop_type
    }
    pub fn index(&self) -> f64 {
        self.index
    }
    pub fn n_children(&self) -> f64 {
        self.n_children
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AircraftPositionInfo {
    position: Coordinate,
    heading: f64,
}

impl AircraftPositionInfo {

    pub(crate) fn get_position(&self) -> &Coordinate {
        &self.position
    }

    pub(crate) fn get_heading(&self) -> f64 {
        self.heading
    }
}

pub fn get_aircraft_position() -> Option<AircraftPositionInfo> {
    let pref = crate::preference::manager();
    if pref.get::<bool>(FGFS_LINK_ENABLED).unwrap_or(false) {
        let host = pref.get::<String>(FGFS_LINK_HOST).unwrap_or("127.0.0.1".to_string());
        let port = pref.get::<String>(FGFS_LINK_PORT).unwrap_or("5100".to_string());

        let base_url = format!("http://{}:{}/json", host, port);

        let longitude = fetch_property(&format!("{}/position/longitude-deg", base_url));
        let latitude = fetch_property(&format!("{}/position/latitude-deg", base_url));
        let heading = fetch_property(&format!("{}/orientation/heading-deg", base_url));
        if longitude.is_ok() && latitude.is_ok() && heading.is_ok() {
            let position = Coordinate::new(latitude.unwrap_or(0.0), longitude.unwrap_or(0.0));
            let heading = heading.unwrap_or(0.0);
            return Some(AircraftPositionInfo{position, heading});
        }
    }
    None
}

fn fetch_property(url: &str) -> Result<f64, Box<dyn Error>> {
    let response = ureq::get(url).call()?;
    let property: FGProperty = response.into_json()?;
    Ok(property.value)
}


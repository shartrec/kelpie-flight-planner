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
use std::any::Any;
use crate::earth::coordinate::Coordinate;

use super::location::Location;

#[derive(Clone, PartialEq)]
pub struct Navaid {
    pub(crate) id: String,
    type_: NavaidType,
    coordinate: Coordinate,
    name: String,
    elevation: i32,
    freq: f64,
    range: i32,
    mag_variation: String,
}

impl Navaid {
    //noinspection RsExternalLinter
    pub fn new(
        id: String,
        type_: NavaidType,
        latitude: f64,
        longitude: f64,
        elevation: i32,
        freq: f64,
        range: i32,
        mag_variation: String,
        name: String,
    ) -> Self {
        Self {
            id,
            type_,
            coordinate: Coordinate::new(latitude, longitude),
            name,
            elevation,
            freq,
            range,
            mag_variation,
        }
    }

    pub fn get_type(&self) -> NavaidType {
        self.type_.clone()
    }

    pub fn get_freq(&self) -> &f64 {
        &self.freq
    }

    pub fn get_range(&self) -> i32 {
        self.range
    }

    pub fn get_mag_variation(&self) -> String {
        self.mag_variation.clone()
    }
}

impl Location for Navaid {
    fn get_elevation(&self) -> &i32 {
        &self.elevation
    }
    fn get_id(&self) -> &str {
        self.id.as_str()
    }
    fn get_lat(&self) -> f64 {
        self.coordinate.get_latitude()
    }
    fn get_lat_as_string(&self) -> String {
        self.coordinate.get_latitude_as_string()
    }
    fn get_long(&self) -> f64 {
        self.coordinate.get_longitude()
    }
    fn get_long_as_string(&self) -> String {
        self.coordinate.get_longitude_as_string()
    }
    fn get_loc(&self) -> &Coordinate {
        &self.coordinate
    }
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, PartialEq)]
pub enum NavaidType {
    Vor,
    Ndb,
    Dme,
}

impl NavaidType {
    pub fn type_for(navaid_type: &str) -> Option<NavaidType> {
        if navaid_type == "0" {
            Some(NavaidType::Dme)
        } else if navaid_type == "2" {
            Some(NavaidType::Ndb)
        } else if navaid_type == "3" {
            Some(NavaidType::Vor)
        } else {
            None
        }
    }
}

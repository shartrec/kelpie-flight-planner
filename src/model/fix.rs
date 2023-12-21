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

use super::location::Location;

#[derive(Clone, PartialEq)]
pub struct Fix {
    id: String,
    coordinate: Coordinate,
}

impl Fix {
    pub fn new(id: String, latitude: f64, longitude: f64) -> Self {
        Self {
            id,
            coordinate: Coordinate::new(latitude, longitude),
        }
    }
}

impl Location for Fix {
    fn get_elevation(&self) -> &i32 {
        &0
    }

    fn get_id(&self) -> &str {
        self.id.as_str()
    }

    fn get_lat(&self) -> &f64 {
        self.coordinate.get_latitude()
    }

    fn get_lat_as_string(&self) -> String {
        self.coordinate.get_latitude_as_string().clone()
    }

    fn get_long(&self) -> &f64 {
        self.coordinate.get_longitude()
    }

    fn get_long_as_string(&self) -> String {
        self.coordinate.get_longitude_as_string().clone()
    }

    fn get_loc(&self) -> &Coordinate {
        &self.coordinate
    }

    fn get_name(&self) -> &str {
        self.get_id()
    }
}

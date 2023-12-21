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

#[derive(Clone)]
pub struct Aircraft {
    name: String,
    cruise_speed: i32,
    cruise_altitude: i32,
    climb_speed: i32,
    climb_rate: i32,
    sink_speed: i32,
    sink_rate: i32,
    is_default: bool,
}

impl Aircraft {
    //noinspection RsExternalLinter
    pub fn new(
        name: String,
        cruise_speed: i32,
        cruise_altitude: i32,
        climb_speed: i32,
        climb_rate: i32,
        sink_speed: i32,
        sink_rate: i32,
        is_default: bool,
    ) -> Self {
        Aircraft {
            name,
            cruise_speed,
            cruise_altitude,
            climb_speed,
            climb_rate,
            sink_speed,
            sink_rate,
            is_default,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_cruise_speed(&self) -> &i32 {
        &self.cruise_speed
    }

    pub fn get_cruise_altitude(&self) -> &i32 {
        &self.cruise_altitude
    }

    pub fn get_climb_speed(&self) -> &i32 {
        &self.climb_speed
    }

    pub fn get_climb_rate(&self) -> &i32 {
        &self.climb_rate
    }

    pub fn get_sink_speed(&self) -> &i32 {
        &self.sink_speed
    }

    pub fn get_sink_rate(&self) -> &i32 {
        &self.sink_rate
    }

    pub fn is_default(&self) -> &bool {
        &self.is_default
    }

}

impl Default for Aircraft {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            cruise_speed: 140,
            cruise_altitude: 7000,
            climb_speed: 120,
            climb_rate: 500,
            sink_speed: 100,
            sink_rate: 500,
            is_default: false,
        }
    }
}

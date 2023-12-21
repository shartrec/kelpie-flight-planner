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

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::Arc;

    use crate::model::airport::{Airport, AirportType};

    pub fn make_airport(id: &str) -> Arc<Airport> {
        Arc::new(Airport::new(
            id.to_string(),
            1.0,
            1.0,
            1,
            Some(AirportType::Airport),
            true,
            false,
            "Sydney".to_string(),
            10000,
        ))
    }

    #[cfg(test)]
    pub fn make_airport_at(id: &str, lat: f64, long: f64) -> Arc<Airport> {
        Arc::new(Airport::new(
            id.to_string(),
            lat,
            long,
            1,
            Some(AirportType::Airport),
            true,
            false,
            "Sydney".to_string(),
            10000,
        ))
    }
}

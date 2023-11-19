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
use crate::preference::{UNITS_KM, UNITS_MI, UNITS_NM};

pub struct DistanceFormat {
    conversion_factor: f64,
    distance_unit: String,
}

impl DistanceFormat {
    pub fn new(unit: &str) -> Self {
        Self {
            conversion_factor: match unit {
                UNITS_NM => 1.0,
                UNITS_MI => 6076.0 / 5280.00,
                UNITS_KM => 1.609 * 6076.0 / 5280.,
                _ => 1.0,
            },
            distance_unit: unit.to_string(),
        }
    }

    pub fn format(&self, distance: &f64) -> String {
        let converted_distance = distance * self.conversion_factor;
        format!("{:.1}{}", converted_distance, self.distance_unit)
    }
}

#[cfg(test)]
mod tests {
    use super::DistanceFormat;

    #[test]
    fn test_fmt_distance() {
        assert_eq!(DistanceFormat::new("Nm").format(&35.0), "35.0Nm");
        assert_eq!(DistanceFormat::new("Mi").format(&34.0), "39.1Mi");
        assert_eq!(DistanceFormat::new("Km").format(&34.0), "63.0Km");
    }
}

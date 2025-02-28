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

use geo::{Bearing, Destination, Distance, Geodesic, InterpolatePoint, Point};
use crate::util::lat_long_format::LatLongFormat;


/// A coordinate on the Earth's surface.
/// This struct is a wrapper around the geo::Point struct.
/// We use the geo crate to perform geodesic calculations, such as distance and bearing between two coordinates.
///
/// In aviation distances are measured in nautical miles and bearings are measured in degrees, with runways
/// defined in feet. We convert between these units as needed.
///
/// Using this struct allows us to keep all these conversions in one place.
///
#[derive(Debug, Clone, PartialEq)]
pub struct Coordinate {
    point: Point,
}

impl Coordinate {
    const METERS_TO_NAUTICAL_MILES: f64 = 0.000539957;
    const NAUTICAL_MILES_TO_METERS: f64 = 1852.0;

    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            point: Point::new(longitude, latitude),
        }
    }

    pub fn from(point: Point) -> Self {
        Self {
            point,
        }
    }

    pub fn bearing_to(&self, to: &Coordinate) -> f64 {
        Geodesic::bearing(self.point.clone(), to.point.clone()).to_radians()
    }

    pub fn bearing_to_deg(&self, to: &Coordinate) -> f64 {
        Geodesic::bearing(self.point.clone(), to.point.clone())
    }

    pub fn coordinate_at(&self, distance: f64, heading: f64) -> Coordinate {
        let d = distance * Self::NAUTICAL_MILES_TO_METERS;
        let p = Geodesic::destination(self.point.clone(), heading, d);
        Coordinate::from(p)
    }

    pub fn distance_to(&self, to: &Coordinate) -> f64 {
        Geodesic::distance(self.point.clone(), to.point.clone()) * Self::METERS_TO_NAUTICAL_MILES
    }

    pub fn get_latitude(&self) -> f64 {
        self.point.y()
    }

    pub fn get_latitude_as_string(&self) -> String {
        let formatter = LatLongFormat::lat_format();
        formatter.format(self.point.y())
    }

    pub fn get_longitude(&self) -> f64 {
        self.point.x()
    }

    pub fn get_longitude_as_string(&self) -> String {
        let formatter = LatLongFormat::long_format();
        formatter.format(self.point.x())
    }

    pub fn midpoint(from: &Coordinate, to: &Coordinate) -> Coordinate {
        let p = Geodesic::point_at_ratio_between(from.point.clone(), to.point.clone(), 0.5);
        Coordinate::from(p)
    }

}

#[cfg(test)]
mod tests {
    use super::Coordinate;

    #[test]
    fn test_construct() {
        let result = Coordinate::new(-34.0, 151.0);
        assert_eq!(result.get_latitude(), -34.0);
        assert_eq!(result.get_longitude(), 151.0);
    }

    #[test]
    fn test_distance_to() {
        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = Coordinate::new(-34.0, 151.0);
        assert_eq!(c1.distance_to(&c2), 0.0);
        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = Coordinate::new(-34.0, 150.0);
        assert_eq!(c1.distance_to(&c2).round(), 50.0);
        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = Coordinate::new(-35.0, 151.0);
        assert_eq!(c1.distance_to(&c2).round(), 60.0);
        let c1 = Coordinate::new(-34.45, 150.50);
        let c2 = Coordinate::new(-34.18, 150.86);
        assert_eq!(c1.distance_to(&c2).round(), 24.0);
        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = Coordinate::new(35.0, -151.0);
        assert_eq!(c1.distance_to(&c2).round(), 5257.0);
        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = Coordinate::new(0.0, 0.0);
        assert_eq!(c1.distance_to(&c2).round(), 8200.0);
    }
    #[test]
    fn test_distance_to_edge_cases() {
        // Same coordinates
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(0.0, 0.0);
        assert_eq!(c1.distance_to(&c2), 0.0);

        // Equator to North Pole
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(90.0, 0.0);
        assert_eq!(c1.distance_to(&c2).round(), 5401.0);

        // Equator to South Pole
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(-90.0, 0.0);
        assert_eq!(c1.distance_to(&c2).round(), 5401.0);

        // Prime Meridian to 90 degrees East
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(0.0, 90.0);
        assert_eq!(c1.distance_to(&c2).round(), 5410.0);

        // Prime Meridian to 90 degrees West
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(0.0, -90.0);
        assert_eq!(c1.distance_to(&c2).round(), 5410.0);
    }
    #[test]
    fn test_bearing_to_deg() {
        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = Coordinate::new(-35.0, 151.0);
        assert_eq!(c1.bearing_to_deg(&c2).round(), 180.0);
        let c1 = Coordinate::new(34.0, 151.0);
        let c2 = Coordinate::new(35.0, 151.0);
        assert_eq!(c1.bearing_to_deg(&c2).round(), 0.0);
        let c1 = Coordinate::new(34.0, 151.0);
        let c2 = Coordinate::new(34.0, 152.0);
        assert_eq!(c1.bearing_to_deg(&c2).round(), 90.0);
    }

    #[test]
    fn test_bearing_to_edge_cases() {
        // Equator to North Pole
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(90.0, 0.0);
        assert_eq!(c1.bearing_to_deg(&c2).round(), 0.0);

        // Equator to South Pole
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(-90.0, 0.0);
        assert_eq!(c1.bearing_to_deg(&c2).round(), 180.0);

        // Prime Meridian to 90 degrees East
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(0.0, 90.0);
        assert_eq!(c1.bearing_to_deg(&c2).round(), 90.0);

        // Prime Meridian to 90 degrees West
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(0.0, -90.0);
        assert_eq!(c1.bearing_to_deg(&c2).round(), 270.0);
    }

    #[test]
    fn test_coordinate_at() {
        let c1 = Coordinate::new(0.0, 151.0);
        let c2 = c1.coordinate_at(120.0, 60.0);
        assert!(assert_between(c2.get_latitude(), 0.99, 1.01));
        assert!(assert_between(c2.get_longitude(), 152.72, 152.74));

        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = c1.coordinate_at(120.0, 120.0);
        assert_between(c2.get_latitude(), -34.99, -34.97);
        assert_between(c2.get_longitude(), 153.10, 153.12);

        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = c1.coordinate_at(100000.0, 120.0);
        assert_between(c2.get_latitude(), 43.0, 44.0);
        assert_between(c2.get_longitude(), 28.0, 29.0);
    }
    #[test]
    fn test_coordinate_at_edge_cases() {
        // Distance to North Pole
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = c1.coordinate_at(5406.0, 0.0);
        assert!(assert_between(c2.get_latitude(), 89.90, 90.01));

        // Distance to South Pole
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = c1.coordinate_at(5406.0, 180.0);
        assert!(assert_between(c2.get_latitude(), -90.01, -89.90));

        // Distance along the equator
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = c1.coordinate_at(5406.0, 90.0);
        assert!(assert_between(c2.get_latitude(), -0.01, 0.01));
        assert!(assert_between(c2.get_longitude(), 89.90, 90.01));

        // Distance along the prime meridian
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = c1.coordinate_at(5406.0, 0.0);
        assert!(assert_between(c2.get_latitude(), 89.90, 90.01));
    }

    fn assert_between(variable: f64, bottom: f64, top: f64) -> bool {
        let result = variable >= bottom && variable <= top;
        if !result {
            assert!(
                result,
                "Variable {} not between {} and {}",
                variable, bottom, top
            );
        }
        result
    }
}

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

use std::f64::consts::PI;

use crate::util::lat_long_format::LatLongFormat;

#[derive(Debug, Clone, PartialEq)]
pub struct Coordinate {
    latitude: f64,
    longitude: f64,
}

impl Coordinate {
    const EARTH_RADIUS: f64 = 3441.85; // Nautical miles

    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    pub fn bearing_to(&self, l: &Coordinate) -> f64 {
        let lat1 = self.latitude.to_radians();
        let lat2 = l.latitude.to_radians();
        let lon1 = self.longitude.to_radians();
        let lon2 = l.longitude.to_radians();

        let d_lon = lon2 - lon1;

        let y = d_lon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * d_lon.cos();

        let mut heading = y.atan2(x);

        if heading < 0.0 {
            heading += 2.0 * PI;
        }
        heading
    }

    pub fn bearing_to_deg(&self, l: &Coordinate) -> f64 {
        self.bearing_to(l).to_degrees()
    }

    pub fn coordinate_at(&self, distance: f64, heading: f64) -> Coordinate {
        let d = distance / Self::EARTH_RADIUS;
        let lat1 = self.latitude.to_radians();
        let lon1 = self.longitude.to_radians();
        let tc = heading.to_radians();

        let sin_lat1 = lat1.sin();
        let cos_lat1 = lat1.cos();
        let sin_d = d.sin();
        let cos_d = d.cos();
        let sin_tc = tc.sin();
        let cos_tc = tc.cos();

        let lat = (sin_lat1 * cos_d + cos_lat1 * sin_d * cos_tc).asin();
        let d_lon = (sin_tc * sin_d * cos_lat1).atan2(cos_d - sin_lat1 * lat.sin());

        let lon = (lon1 + d_lon + PI) % (2.0 * PI) - PI;

        Coordinate::new(lat.to_degrees(), lon.to_degrees())
    }

    pub fn distance_to(&self, l: &Coordinate) -> f64 {
        let lat1 = self.latitude.to_radians();
        let lat2 = l.latitude.to_radians();
        let d_lat = lat2 - lat1;
        let d_lon = (l.longitude - self.longitude).to_radians();

        let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        Self::EARTH_RADIUS * c
    }
    pub fn get_latitude(&self) -> &f64 {
        &self.latitude
    }

    pub fn get_latitude_as_string(&self) -> String {
        let formatter = LatLongFormat::lat_format();
        formatter.format(self.latitude)
    }

    pub fn get_longitude(&self) -> &f64 {
        &self.longitude
    }

    pub fn get_longitude_as_string(&self) -> String {
        let formatter = LatLongFormat::long_format();
        formatter.format(self.longitude)
    }
}

#[cfg(test)]
mod tests {
    use super::Coordinate;

    #[test]
    fn test_construct() {
        let result = Coordinate::new(-34.0, 151.0);
        assert_eq!(*result.get_latitude(), -34.0);
        assert_eq!(*result.get_longitude(), 151.0);
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
        assert_eq!(c1.distance_to(&c2).round(), 5272.0);
        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = Coordinate::new(0.0, 0.0);
        assert_eq!(c1.distance_to(&c2).round(), 8198.0);
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
        assert_eq!(c1.distance_to(&c2).round(), 5406.0);

        // Equator to South Pole
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(-90.0, 0.0);
        assert_eq!(c1.distance_to(&c2).round(), 5406.0);

        // Prime Meridian to 90 degrees East
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(0.0, 90.0);
        assert_eq!(c1.distance_to(&c2).round(), 5406.0);

        // Prime Meridian to 90 degrees West
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(0.0, -90.0);
        assert_eq!(c1.distance_to(&c2).round(), 5406.0);
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
        // Same coordinates
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = Coordinate::new(0.0, 0.0);
        assert_eq!(c1.bearing_to_deg(&c2).round(), 0.0);

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
        assert!(assert_between(c2.latitude, 0.99, 1.01));
        assert!(assert_between(c2.longitude, 152.72, 152.74));

        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = c1.coordinate_at(120.0, 120.0);
        assert_between(c2.latitude, -34.99, -34.97);
        assert_between(c2.longitude, 153.10, 153.12);

        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = c1.coordinate_at(100000.0, 120.0);
        assert_between(c2.latitude, 43.0, 44.0);
        assert_between(c2.longitude, 28.0, 29.0);
    }
    #[test]
    fn test_coordinate_at_edge_cases() {
        // Zero distance
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = c1.coordinate_at(0.0, 0.0);
        assert_eq!(c1, c2);

        // Distance to North Pole
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = c1.coordinate_at(5406.0, 0.0);
        assert!(assert_between(c2.latitude, 89.99, 90.01));
        assert!(assert_between(c2.longitude, -0.01, 0.01));

        // Distance to South Pole
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = c1.coordinate_at(5406.0, 180.0);
        assert!(assert_between(c2.latitude, -90.01, -89.99));
        assert!(assert_between(c2.longitude, -0.01, 0.01));

        // Distance along the equator
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = c1.coordinate_at(5406.0, 90.0);
        assert!(assert_between(c2.latitude, -0.01, 0.01));
        assert!(assert_between(c2.longitude, 89.99, 90.01));

        // Distance along the prime meridian
        let c1 = Coordinate::new(0.0, 0.0);
        let c2 = c1.coordinate_at(5406.0, 0.0);
        assert!(assert_between(c2.latitude, 89.99, 90.01));
        assert!(assert_between(c2.longitude, -0.01, 0.01));
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

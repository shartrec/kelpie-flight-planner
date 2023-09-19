use std::f64::consts::PI;

use crate::util::lat_long_format::LatLongFormat;

#[derive(Debug, Clone, PartialEq)]
pub struct Coordinate {
    latitude: f64,
    longitude: f64,
}

impl Coordinate {
    const EARTH_RADIUS: f64 = 3441.85;

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

        let d_lon = lon1 - lon2;
        let d_lat = lat1 - lat2;

        let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
        let d = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        let x = (lat2.sin() - lat1.sin() * d.cos()) / (d.sin() * lat1.cos());
        let mut heading = x.acos();

        if (lon2 - lon1).sin() < 0.0 {
            heading = 2.0 * PI - heading;
        }

        heading
    }

    pub fn bearing_to_deg(&self, l: &Coordinate) -> f64 {
        self.bearing_to(l).to_degrees()
    }

    pub fn coordinate_at(&self, distance: f64, heading: f64) -> Coordinate {
        let d = distance as f64 / Self::EARTH_RADIUS;
        let lat1 = self.latitude.to_radians();
        let lon1 = self.longitude.to_radians();
        let tc = heading.to_radians();
        let lat = (lat1.sin() * d.cos() + lat1.cos() * d.sin() * tc.cos()).asin();
        let d_lon = (tc.sin() * d.sin() * lat1.cos()).atan2(d.cos() - lat1.sin() * lat.sin());

        let lon = (lon1 + d_lon + PI) % (2.0 * PI) - PI;

        Coordinate::new(lat.to_degrees(), lon.to_degrees())
    }

    pub fn distance_to(&self, l: &Coordinate) -> f64 {
        let lat1 = self.latitude.to_radians();
        let lat2 = l.latitude.to_radians();
        let lon1 = self.longitude.to_radians();
        let lon2 = l.longitude.to_radians();
        let d_lon = lon1 - lon2;
        let d_lat = lat1 - lat2;

        let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
        let d = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        (Self::EARTH_RADIUS * d.abs())
    }

    pub fn get_latitude(&self) -> f64 {
        self.latitude
    }

    pub fn get_latitude_as_string(&self) -> String {
        let formatter = LatLongFormat::lat_format();
        formatter.format(self.latitude)
    }

    pub fn get_longitude(&self) -> f64 {
        self.longitude
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
        assert_eq!(c1.distance_to(&c2), 50.0);
        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = Coordinate::new(-35.0, 151.0);
        assert_eq!(c1.distance_to(&c2), 60.0);
        let c1 = Coordinate::new(-34.45, 150.50);
        let c2 = Coordinate::new(-34.18, 150.86);
        assert_eq!(c1.distance_to(&c2), 24.0);
        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = Coordinate::new(35.0, -151.0);
        assert_eq!(c1.distance_to(&c2), 5272.0);
        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = Coordinate::new(0.0, 0.0);
        assert_eq!(c1.distance_to(&c2), 8198.0);
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
        assert_eq!(c1.bearing_to_deg(&c2), 90.0);
    }

    #[test]
    fn test_coordinate_at() {
        let c1 = Coordinate::new(0.0, 151.0);
        let c2 = c1.coordinate_at(120.0, 60.0);
        assert!(is_between(c2.latitude, 0.99, 1.01));
        assert!(is_between(c2.longitude, 152.72, 152.74));

        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = c1.coordinate_at(120.0, 120.0);
        assert!(is_between(c2.latitude, -34.99, -34.97));
        assert!(is_between(c2.longitude, 153.10, 153.12));

        let c1 = Coordinate::new(-34.0, 151.0);
        let c2 = c1.coordinate_at(100000.0, 120.0);
        assert!(is_between(c2.latitude, 43.0, 44.0));
        assert!(is_between(c2.longitude, 28.0, 29.0));
    }

    fn is_between(variable: f64, bottom: f64, top: f64) -> bool {
        let result = variable >= bottom && variable <= top;
        if !result {
            println!("Variable {} not between {} and {}", variable, bottom, top);
        }
        result
    }
}

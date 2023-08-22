use super::location::Location;
use crate::earth::coordinate::Coordinate;

#[derive(Debug, Clone)]
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
    fn get_elevation(&self) -> f64 {
        0.0
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_lat(&self) -> f64 {
        self.coordinate.get_latitude()
    }

    fn get_lat_as_string(&self) -> String {
        self.coordinate.get_latitude_as_string().clone()
    }

    fn get_long(&self) -> f64 {
        self.coordinate.get_longitude()
    }

    fn get_long_as_string(&self) -> String {
        self.coordinate.get_longitude_as_string().clone()
    }

    fn get_loc(&self) -> Coordinate {
        self.coordinate.clone()
    }

    fn get_name(&self) -> String {
        "".to_string()
    }
}

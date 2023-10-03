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
        &self.id.as_str()
    }

    fn get_lat(&self) -> &f64 {
        &self.coordinate.get_latitude()
    }

    fn get_lat_as_string(&self) -> String {
        self.coordinate.get_latitude_as_string().clone()
    }

    fn get_long(&self) -> &f64 {
        &self.coordinate.get_longitude()
    }

    fn get_long_as_string(&self) -> String {
        self.coordinate.get_longitude_as_string().clone()
    }

    fn get_loc(&self) -> &Coordinate {
        &self.coordinate
    }

    fn get_name(&self) -> &str {&self.get_id()}
}

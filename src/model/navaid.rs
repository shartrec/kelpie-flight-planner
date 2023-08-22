use super::location::Location;
use crate::earth::coordinate::Coordinate;

#[derive(Clone)]
pub struct Navaid {
    id: String,
    type_: NavaidType,
    coordinate: Coordinate,
    name: String,
    elevation: f64,
    freq: f64,
    range: f64,
    mag_variation: String,
}

impl Navaid {
    pub fn new(
        id: String,
        type_: NavaidType,
        latitude: f64,
        longitude: f64,
        elevation: f64,
        freq: f64,
        range: f64,
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

    pub fn get_freq(&self) -> f64 {
        self.freq
    }

    pub fn get_range(&self) -> f64 {
        self.range
    }

    pub fn get_mag_variation(&self) -> String {
        self.mag_variation.clone()
    }
}

impl Location for Navaid {
    fn get_elevation(&self) -> f64 {
        self.elevation
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
        self.name.clone()
    }
}

#[derive(Clone, PartialEq)]
pub enum NavaidType {
    VOR,
    NDB,
    DME,
}

impl NavaidType {
    pub fn type_for(navaid_type: &str) -> Option<NavaidType> {
        if navaid_type == "0" {
            Some(NavaidType::DME)
        } else if navaid_type == "2" {
            Some(NavaidType::NDB)
        } else if navaid_type == "3" {
            Some(NavaidType::VOR)
        } else {
            None
        }
    }
}

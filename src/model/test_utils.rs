
use super::airport::{Airport, AirportType};

pub fn make_airport(id: &str) -> super::airport::Airport {
    Airport::new(
        id.to_string(),
        1.0,
        1.0,
        1,
        Some(AirportType::AIRPORT),
        true,
        false,
        "Sydney".to_string(),
        10000,
    )
}
pub fn make_airport_at(id: &str, lat: f64, long: f64) -> super::airport::Airport {
    Airport::new(
        id.to_string(),
        lat,
        long,
        1,
        Some(AirportType::AIRPORT),
        true,
        false,
        "Sydney".to_string(),
        10000,
    )
}

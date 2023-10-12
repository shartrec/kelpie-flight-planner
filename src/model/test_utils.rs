use std::sync::Arc;

use super::airport::{Airport, AirportType};

#[cfg(test)]
pub fn make_airport(id: &str) -> Arc<super::airport::Airport> {
    Arc::new(Airport::new(
        id.to_string(),
        1.0,
        1.0,
        1,
        Some(AirportType::AIRPORT),
        true,
        false,
        "Sydney".to_string(),
        10000,
    ))
}

#[cfg(test)]
pub fn make_airport_at(id: &str, lat: f64, long: f64) -> Arc<super::airport::Airport> {
    Arc::new(Airport::new(
        id.to_string(),
        lat,
        long,
        1,
        Some(AirportType::AIRPORT),
        true,
        false,
        "Sydney".to_string(),
        10000,
    ))
}

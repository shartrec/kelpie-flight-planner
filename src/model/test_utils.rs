
use super::airport::{Airport, AirportType};

pub fn make_airport(id: &str) -> super::airport::Airport {
    Airport::new(
        id.to_string(),
        1.0,
        1.0,
        1.0,
        Some(AirportType::AIRPORT),
        true,
        false,
        "Sydney".to_string(),
        10000,
    )
}

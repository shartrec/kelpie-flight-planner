#[cfg(test)]
pub(crate) mod tests {
    use std::sync::Arc;

    use crate::model::airport::{Airport, AirportType};

    pub fn make_airport(id: &str) -> Arc<Airport> {
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
    pub fn make_airport_at(id: &str, lat: f64, long: f64) -> Arc<Airport> {
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
}

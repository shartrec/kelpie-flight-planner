/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
pub struct DistanceFormat {
    conversion_factor: f64,
    distance_unit: String,
}

impl DistanceFormat {
    pub fn new(unit: &str) -> Self {
        Self {
            conversion_factor: match unit {
                "Nm" => 1.0,
                "Mi" => 6076.0 / 5280.00,
                "Km" => 1.609 * 6076.0 / 5280.,
                _ => 1.0,
            },
            distance_unit: unit.to_string(),
        }
    }

    pub fn format(&self, distance: &f64) -> String {
        let converted_distance = distance * self.conversion_factor;
        format!("{:.1}{}", converted_distance, self.distance_unit)
    }
}

#[cfg(test)]
mod tests {
    use super::DistanceFormat;

    #[test]
    fn test_fmt_distance() {
        assert_eq!(DistanceFormat::new("Nm").format(&35.0), "35.0Nm");
        assert_eq!(DistanceFormat::new("Mi").format(&34.0), "39.1Mi");
        assert_eq!(DistanceFormat::new("Km").format(&34.0), "63.0Km");
    }
}

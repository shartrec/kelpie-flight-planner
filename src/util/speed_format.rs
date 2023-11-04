use crate::preference::{UNITS_KM, UNITS_MI, UNITS_NM};

pub struct SpeedFormat {
    conversion_factor: f64,
    speed_unit: String,
}

impl SpeedFormat {
    pub fn new(unit: &str) -> Self {
        let conversion_factor: f64;
        let speed_unit: &str;

        if unit == UNITS_NM {
            conversion_factor = 1.0;
            speed_unit = "Kts";
        } else if unit == UNITS_MI {
            conversion_factor = 6076.0 / 5280.0;
            speed_unit = "Mph";
        } else if unit == UNITS_KM {
            conversion_factor = 1.609 * 6076.0 / 5280.0;
            speed_unit = "Kph";
        } else {
            panic!("Invalid unit");
        }

        SpeedFormat {
            conversion_factor,
            speed_unit: speed_unit.to_string(),
        }
    }

    pub fn format(&self, speed: &f64) -> String {
        let converted_speed = speed * self.conversion_factor;

        format!("{:.0}{}", converted_speed, self.speed_unit)
    }
}

#[cfg(test)]
mod tests {
    use super::SpeedFormat;

    #[test]
    fn test_fmt_speed() {
        assert_eq!(SpeedFormat::new("Nm").format(&34.5), "34Kts");
        assert_eq!(SpeedFormat::new("Nm").format(&34.0), "34Kts");
        assert_eq!(SpeedFormat::new("Nm").format(&34.9), "35Kts");
        assert_eq!(SpeedFormat::new("Mi").format(&34.5), "40Mph");
        assert_eq!(SpeedFormat::new("Km").format(&34.5), "64Kph");
    }
}

use std::fmt;

#[derive(Debug, Clone)]
pub struct Runway {
    ils_opposite_freq: f64,
    ils_freq: f64,
    number: String,
    lat: f64,
    long: f64,
    heading: f64,
    length: i32,
    width: i32,
    centre_line_lights: bool,
    surface: String,
    markings: String,
    edge_lights: String,
}

impl Runway {
    fn new(
        number: String,
        lat: f64,
        long: f64,
        length: i32,
        width: i32,
        heading: f64,
        centre_line_lights: bool,
        surface: String,
        markings: String,
        edge_lights: String,
    ) -> Self {
        Runway {
            ils_opposite_freq: 0.0,
            ils_freq: 0.0,
            number,
            lat,
            long,
            heading,
            length,
            width,
            centre_line_lights,
            surface,
            markings,
            edge_lights,
        }
    }

    pub fn get_ils_freq(&self) -> f64 {
        self.ils_freq
    }

    pub fn get_ils_opposite_freq(&self) -> f64 {
        self.ils_opposite_freq
    }

    pub fn get_centre_line_lights(&self) -> bool {
        self.centre_line_lights
    }

    pub fn get_edge_lights(&self) -> &str {
        &self.edge_lights
    }

    pub fn get_heading(&self) -> f64 {
        self.heading
    }

    pub fn get_lat(&self) -> f64 {
        self.lat
    }

    pub fn get_length(&self) -> i32 {
        self.length
    }

    pub fn get_long(&self) -> f64 {
        self.long
    }

    pub fn get_markings(&self) -> &str {
        &self.markings
    }

    pub fn get_number(&self) -> &str {
        &self.number
    }

    pub fn get_number_pair(&self) -> String {
        format!("{}/{}", self.number, self.get_opposite_number())
    }

    pub fn get_opposite_number(&self) -> String {
        Self::get_opp_num(&self.number)
    }

    fn get_opp_num(_number: &str) -> String {
        match _number {
            "N" => "S".to_string(),
            "S" => "N".to_string(),
            "E" => "W".to_string(),
            "W" => "E".to_string(),
            _ => {
                let heading_part = match _number.find(|c: char| !c.is_digit(10)) {
                    Some(idx) => &_number[..idx],
                    None => _number,
                };

                let mut extra_part = "";
                if _number.ends_with("R") {
                    extra_part = "L";
                } else if _number.ends_with("L") {
                    extra_part = "R";
                } else if _number.ends_with("C") {
                    extra_part = "C";
                }

                let x = heading_part.parse::<i32>().unwrap_or(0);
                let opposite_heading = if x <= 18 { x + 18 } else { x - 18 };
                format!("{:02}{}", opposite_heading, extra_part)
            }
        }
    }

    pub fn get_surface(&self) -> &str {
        &self.surface
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }
}

#[cfg(test)]
mod tests {
    use super::Runway;

    #[test]
    fn test_opposite() {
        let runway = Runway::new(
            "12R".to_string(),
            37.7749,
            -122.4194,
            8000,
            150,
            120.0,
            true,
            "A".to_string(),
            "V".to_string(),
            "H".to_string(),
        );

        assert_eq!(runway.get_opposite_number(), "30L");
        assert_eq!(Runway::get_opp_num("27"), "09");
        assert_eq!(Runway::get_opp_num("27C"), "09C");
        assert_eq!(Runway::get_opp_num("27L"), "09R");
        assert_eq!(Runway::get_opp_num("09L"), "27R");
        assert_eq!(Runway::get_opp_num("18L"), "36R");
        assert_eq!(Runway::get_opp_num("36L"), "18R");
    }
}

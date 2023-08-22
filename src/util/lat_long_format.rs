use std::str::FromStr;

pub struct LatLongFormat {
	pos_token: char,
	neg_token: char,
	max_degree: f64,
}

impl LatLongFormat {
    pub fn lat_format() -> Self {
        LatLongFormat{pos_token : 'N', neg_token:'S', max_degree:90.0}
    }

    pub fn long_format() -> Self {
        LatLongFormat{pos_token : 'E', neg_token:'W', max_degree:180.0}
    }

    fn dec_to_degree(&self, buff: &mut String, d: f64, bearing: &str) {
        let mut deg = d.floor();
        let minsec = (d - deg) * 60.0;
        let mut min = minsec.floor();
        let mut sec = (minsec - min) * 60.0;

        if (60.0 - sec) < 0.005 {
            sec = 0.0;
            min += 1.0;
        }

        if (60.0 - min) < 0.005 {
            min = 0.0;
            deg += 1.0;
        }

        buff.push_str(&format!("{:02}\u{00b0}{:02}\"{:02}\'{}", deg, min, sec, bearing));
    }

    fn format_absolute(&self, number: f64, bearing: char) -> String {
        let mut buffer = String::new();
        self.dec_to_degree(&mut buffer, number, String::from(bearing).as_str());
        buffer
    }

    pub fn format(&self, number: f64) -> String {
    	self.format_absolute(number, if number > 0.0 {self.pos_token} else {self.pos_token})
    }

    pub fn parse(&self, source: &str) -> Result<f64, &str> {
        let mut sign = 1.0;
        let mut deg = 0.0;
        let mut min = 0.0;
        let mut sec = 0.0;

        let work = source.trim();
        let last_char = work.chars().last().unwrap_or(' ');

        if last_char == self.pos_token {
                sign = 1.0;
        } else if last_char == self.neg_token {
                sign = -1.0;
        }

        let tokenizer = work
            .split(|c: char| {
                c.is_whitespace() || c == '.' || c == '\u{00b0}' || c == '"' || c == '\''
            })
            .filter(|token| !token.is_empty());

        let tokens: Vec<&str> = tokenizer.collect();

        if let Some(deg_tok) = tokens.get(0) {
            deg = match f64::from_str(deg_tok) {
                Ok(num) => num,
                Err(_) => return Err("Invalid coordinate format"),
            };
            if deg > 180.0 {
                return Err("Out of range");
            }
        }

        if let Some(min_tok) = tokens.get(1) {
            min = match f64::from_str(min_tok) {
                Ok(num) => num,
                Err(_) => return Err("Invalid coordinate format"),
            };
            if min > 60.0 {
                return Err("Out of Range");
            }
        }

        if let Some(sec_tok) = tokens.get(2) {
            sec = match f64::from_str(sec_tok) {
                Ok(num) => num,
                Err(_) => return Err("Invalid coordinate format"),
            };
            if sec > 60.0 {
                return Err("Out of Range");
            }
        }

        Ok((deg + min / 60.0 + sec / 3600.0) * sign)
    }
}


#[cfg(test)]
mod tests {
    use super::LatLongFormat;

    #[test]
    fn test_fmt_lat() {
        let formatter = LatLongFormat::lat_format();
        assert_eq!(formatter.format(34.5), "34\u{00b0}30\"00\'N");
    }
    #[test]
    fn test_fmt_long() {
        let formatter = LatLongFormat::long_format();
        assert_eq!(formatter.format(34.5), "34\u{00b0}30\"00\'E");
    }
    #[test]
    fn test_parse_lat() {
        let formatter = LatLongFormat::lat_format();
        assert_eq!(formatter.parse("34\u{00b0}30\"00\'N").unwrap(), 34.5);
        assert_eq!(formatter.parse("34\u{00b0}30\"00\'S").unwrap(), -34.5);
    }
    #[test]
    fn test_parse_long() {
        let formatter = LatLongFormat::long_format();
        assert_eq!(formatter.parse("34\u{00b0}30\"00\'E").unwrap(), 34.5);
        assert_eq!(formatter.parse("34\u{00b0}30\"00\'W").unwrap(), -34.5);
    }
    #[test]
    fn test_parse_error() -> Result<(), String> {
        let formatter = LatLongFormat::lat_format();
        match formatter.parse("234\u{00b0}30\"00\'E") {
        	Ok(_) => Err(String::from("Invalid format should not parse")),
        	Err(_) => Ok(()),
        }
    }
}

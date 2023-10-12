pub struct HourFormat {}

impl HourFormat {
    pub fn new() -> Self {
        HourFormat {}
    }

    pub fn format(&self, time: &f64) -> String {
        // The degree portion is just the integer portion of the value
        let mut hours = time.floor();
        let minsec = time.fract() * 60.0;
        let mut min = minsec.floor();
        let sec = min.fract() * 60.0;
        // Correct for rounding errors
        if sec > 30.0 {
            min += 1.0;
        }
        if 60.0 - min < 0.005 {
            min = 0.0;
            hours += 1.0;
        }
        format!("{:02.0}:{:02.0}", hours, min)
    }
}

#[cfg(test)]
mod tests {
    use super::HourFormat;

    #[test]
    fn test_fmt_time_as_hours() {
        let format = HourFormat::new();
        assert_eq!(format.format(&5.5), "05:30");
        assert_eq!(format.format(&2.15), "02:08");
        assert_eq!(format.format(&15.922), "15:55");
        assert_eq!(format.format(&16.005), "16:00");
        assert_eq!(format.format(&1.0), "01:00");
    }
}

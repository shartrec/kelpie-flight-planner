/*
 * Copyright (c) 2003-2003-2023. Trevor Campbell and others.
 *
 * This file is part of Kelpie Flight Planner.
 *
 * Kelpie Flight Planner is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * Kelpie Flight Planner is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Kelpie Flight Planner; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 * Contributors:
 *      Trevor Campbell
 *
 */

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

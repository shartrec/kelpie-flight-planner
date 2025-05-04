/*
 * Copyright (c) 2003-2025. Trevor Campbell and others.
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
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};

pub(crate) fn subsolar_point(datetime: DateTime<Utc>) -> (f64, f64) {
    let timestamp = datetime.timestamp() as f64;

    // Days since J2000 epoch (Jan 1, 2000, 12:00 UTC)
    let days_since_j2000 = (timestamp - 946728000.0) / 86400.0;

    // Mean longitude of the Sun (deg)
    let mean_long = (280.460 + 0.9856474 * days_since_j2000) % 360.0;

    // Mean anomaly of the Sun (deg)
    let mean_anom = (357.528 + 0.9856003 * days_since_j2000) % 360.0;
    let mean_anom_rad = mean_anom.to_radians();

    // Ecliptic longitude (deg)
    let eclip_long = (mean_long + 1.915 * mean_anom_rad.sin() + 0.020 * (2.0 * mean_anom_rad).sin()) % 360.0;
    let eclip_long_rad = eclip_long.to_radians();

    // Obliquity of the ecliptic
    let obliquity = 23.439_f64.to_radians();

    // Subsolar latitude = solar declination
    let decl = (eclip_long_rad.sin() * obliquity.sin()).asin().to_degrees(); // [-23.44, +23.44]

    // Subsolar longitude
    let eot = equation_of_time(datetime.ordinal() as u32);
    let time = datetime.time() + Duration::seconds(eot as i64 * 60);
    let minutes_from_noon = 720.0 - (time.hour() * 60 + time.minute()) as f64;
    let utc_offset_deg = (minutes_from_noon / 1440.0) * 360.0;


    let subsolar_lon = utc_offset_deg % 360.0;
    // Wrap longitude to [-180, +180] for consistency
    let lon = if subsolar_lon > 180.0 {
        subsolar_lon - 360.0
    } else {
        subsolar_lon
    };

    (decl, lon)
}

fn equation_of_time(day_of_year: u32) -> f64 {
    let b_deg = 360.0 / 365.0 * (day_of_year as f64 - 81.0);
    let b_rad = b_deg.to_radians();

    // Approximate equation of time (in minutes)
    let eot = 9.87 * (2.0 * b_rad).sin()
        - 7.53 * b_rad.cos()
        - 1.5  * b_rad.sin();

    eot
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_subsolar_point() {
        let datetime = Utc.with_ymd_and_hms(2023, 6, 21, 12, 0, 45);
        let (lat, lon) = subsolar_point(datetime.unwrap());
        assert_eq!(lat.round(), 23.0);
        assert_eq!(lon.round(), 0.0);
    }

    #[test]
    // Data from https://gml.noaa.gov/grad/solcalc/
    fn test_subsolar_point_non_noon_equinox() {
        let datetime = Utc.with_ymd_and_hms(2025, 9, 22, 18, 19, 0);
        let (lat, lon) = subsolar_point(datetime.unwrap());
        assert_eq!(lat.round(), 0.0);
        assert_eq!(lon.round(), -97.0);
    }
    #[test]
    // Data from https://gml.noaa.gov/grad/solcalc/
    fn test_subsolar_point_20() {
        let datetime = Utc.with_ymd_and_hms(2025, 5, 4, 0, 56, 51);
        let (lat, lon) = subsolar_point(datetime.unwrap());
        println!("lat: {}, lon: {}", lat, lon);
        assert_eq!(lat.round(), 16.0);
        assert_eq!(lon.round(), 165.0);
    }
    #[test]
    // Data from https://gml.noaa.gov/grad/solcalc/
    fn test_subsolar_point_max_eot() {
        let datetime = Utc.with_ymd_and_hms(2025, 11, 3, 0, 0, 0);
        let (lat, lon) = subsolar_point(datetime.unwrap());
        println!("lat: {}, lon: {}", lat, lon);
        assert_eq!(lat.round(), -15.0);
        assert_eq!(lon.round(), 176.0);
    }
    #[test]
    // Data from https://gml.noaa.gov/grad/solcalc/
    fn test_subsolar_point_max_eot2() {
        let datetime = Utc.with_ymd_and_hms(2025, 11, 3, 15, 34, 0);
        let (lat, lon) = subsolar_point(datetime.unwrap());
        println!("lat: {}, lon: {}", lat, lon);
        assert_eq!(lat.round(), -15.0);
        assert_eq!(lon.round(), -57.0);
    }
    #[test]
    // Data from https://gml.noaa.gov/grad/solcalc/
    fn test_subsolar_point_min_eot() {
        let datetime = Utc.with_ymd_and_hms(2025, 2, 11, 0, 0, 0);
        let (lat, lon) = subsolar_point(datetime.unwrap());
        println!("lat: {}, lon: {}", lat, lon);
        assert_eq!(lat.round(), -14.0);
        assert_eq!(lon.round(), -177.0);
    }
    #[test]
    // Data from https://gml.noaa.gov/grad/solcalc/
    fn test_subsolar_point_min_eot2() {
        let datetime = Utc.with_ymd_and_hms(2025, 2, 11, 15, 34, 0);
        let (lat, lon) = subsolar_point(datetime.unwrap());
        println!("lat: {}, lon: {}", lat, lon);
        assert_eq!(lat.round(), -14.0);
        assert_eq!(lon.round(), -50.0);
    }
}
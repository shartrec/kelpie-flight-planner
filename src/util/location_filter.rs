/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */

// Range filer for determining if a coordinate is within the specified distance of another

use regex::{Regex, RegexBuilder};

use crate::earth::coordinate::Coordinate;
use crate::model::location::Location;

pub trait Filter {
    fn filter(&self, location: &dyn Location) -> bool;
}


pub struct RangeFilter {
    this: Coordinate,
    range: f64,
    rough_lat_sep: f64,
    rough_long_sep: f64,
}

impl RangeFilter {
    pub fn new(lat: f64, lon: f64, range: f64) -> Option<Self> {
        // We do a little optimization here rather than calculating
        // all distances accurately; we make a quick rough calculation to exclude many coordinates
        let rough_lat_sep = range / 60.0;
        let x = lat.to_radians().cos();
        let rough_long_sep = if x < 0.01 { 181.0 } else { range / (60.0 * x) };

        Some(Self { this: Coordinate::new(lat, lon), range, rough_lat_sep, rough_long_sep })
    }
}
impl Filter for RangeFilter {
    // returns true if the coordinate passes the filter
    fn filter(&self, location: &dyn Location) -> bool {
        let other = location.get_loc();
        if ((self.this.get_latitude() - other.get_latitude()).abs() < self.rough_lat_sep) &
            ((self.this.get_longitude() - other.get_longitude()).abs() < self.rough_long_sep)  {
            self.this.distance_to(other) < self.range
        } else {
            false
        }
    }
}

pub struct NameIdFilter {
    term: String,
    regex: Regex,
}

impl NameIdFilter {
    pub fn new(term: &str) -> Option<Self> {
        match RegexBuilder::new(term).case_insensitive(true).build() {
            Ok(regex) => {
                Some(Self { term: term.to_string(), regex: regex })
            }
            Err(_) => {
                None
            }
        }
    }
}

impl Filter for NameIdFilter {
    fn filter(&self, location: &dyn Location) -> bool {
        location.get_id().eq_ignore_ascii_case(&self.term) || self.regex.is_match(location.get_name())
    }
}

pub struct IdFilter {
    term: String,
}

impl IdFilter {
    pub fn new(term: &str) -> Option<Self> {
        Some(Self { term: term.to_string()})
    }
}

impl Filter for IdFilter {
    fn filter(&self, location: &dyn Location) -> bool {
        location.get_id().eq_ignore_ascii_case(&self.term)
    }
}

pub struct CombinedFilter {
    filters: Vec<Box<dyn Filter>>,
}

impl CombinedFilter {
    pub fn new() -> Self {
        Self {
            filters: Vec::new()
        }
    }
    
    pub fn add(&mut self, filter: Box<dyn Filter>) {
        self.filters.push(filter);
    }
}

impl Filter for CombinedFilter {
    fn filter(&self, location: &dyn Location) -> bool {
        for f in &self.filters {
            if !f.filter(location) {
                return false;
            }
        }
        true
    }
}
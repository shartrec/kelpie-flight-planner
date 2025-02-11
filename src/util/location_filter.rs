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

use adw::prelude::Cast;
use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::CustomFilter;
use regex_lite::{Regex, RegexBuilder};

use crate::earth::coordinate::Coordinate;
use crate::model::airport_object::AirportObject;
use crate::model::fix_object::FixObject;
use crate::model::location::Location;
use crate::model::navaid::{Navaid, NavaidType};
use crate::model::navaid_object::NavaidObject;

pub fn new_airport_filter(filter: Box<dyn Filter>) -> CustomFilter {
    CustomFilter::new(move |obj| {
        let airport_object = obj.clone()
            .downcast::<AirportObject>()
            .expect("The item has to be an `Airport`.");

        let airport = airport_object.imp().airport();
        let airport: &dyn Location = &*airport;
        filter.filter(airport)
    })
}

pub fn set_airport_filter(custom_filter: &CustomFilter, filter: Box<dyn Filter>) {
    custom_filter.set_filter_func(move |obj| {
        let airport_object = obj.clone()
            .downcast::<AirportObject>()
            .expect("The item has to be an `Airport`.");

        let airport = airport_object.imp().airport();
        let airport: &dyn Location = &*airport;
        filter.filter(airport)
    })
}

pub fn new_navaid_filter(filter: Box<dyn Filter>) -> CustomFilter {
    CustomFilter::new(move |obj| {
        let navaid_object = obj.clone()
            .downcast::<NavaidObject>()
            .expect("The item has to be an `Navaid`.");

        let navaid = navaid_object.imp().navaid();
        let navaid: &dyn Location = &*navaid;
        filter.filter(navaid)
    })
}

pub fn set_navaid_filter(custom_filter: &CustomFilter, filter: Box<dyn Filter>) {
    custom_filter.set_filter_func(move |obj| {
        let navaid_object = obj.clone()
            .downcast::<NavaidObject>()
            .expect("The item has to be an `Navaid`.");

        let navaid = navaid_object.imp().navaid();
        let navaid: &dyn Location = &*navaid;
        filter.filter(navaid)
    })
}

pub fn new_fix_filter(filter: Box<dyn Filter>) -> CustomFilter {
    CustomFilter::new(move |obj| {
        let fix_object = obj.clone()
            .downcast::<FixObject>()
            .expect("The item has to be an `Fix`.");

        let fix = fix_object.imp().fix();
        let fix: &dyn Location = &*fix;
        filter.filter(fix)
    })
}

pub fn set_fix_filter(custom_filter: &CustomFilter, filter: Box<dyn Filter>) {
    custom_filter.set_filter_func(move |obj| {
        let fix_object = obj.clone()
            .downcast::<FixObject>()
            .expect("The item has to be an `Fix`.");

        let fix = fix_object.imp().fix();
        let fix: &dyn Location = &*fix;
        filter.filter(fix)
    })
}

pub trait Filter {
    fn filter(&self, location: &dyn Location) -> bool;
}

pub struct NameIdFilter {
    term: String,
    regex: Regex,
}

impl NameIdFilter {
    pub fn new(term: &str) -> Option<Self> {
        let regex = RegexBuilder::new(term).case_insensitive(true).build().ok()?;
        Some(Self {
            term: term.to_string(),
            regex,
        })
    }
}

impl Filter for NameIdFilter {
    fn filter(&self, location: &dyn Location) -> bool {
        location.get_id().eq_ignore_ascii_case(&self.term)
            || self.regex.is_match(location.get_name())
    }
}

pub struct IdFilter {
    term: String,
}

impl IdFilter {
    pub fn new(term: &str) -> Option<Self> {
        Some(Self {
            term: term.to_string(),
        })
    }
}

impl Filter for IdFilter {
    fn filter(&self, location: &dyn Location) -> bool {
        location.get_id().eq_ignore_ascii_case(&self.term)
    }
}
// Range filer for determining if a coordinate is within the specified distance of another
pub struct RangeFilter {
    this: Coordinate,
    range: f64,
    rough_lat_sep: f64,
    rough_long_sep: f64,
}

impl RangeFilter {
    pub fn new(this: Coordinate, range: f64) -> Self {
        // We do a little optimization here rather than calculating
        // all distances accurately; we make a quick rough calculation to exclude many coordinates
        let rough_lat_sep = range / 60.0;
        let x = this.get_latitude().to_radians().cos();
        let rough_long_sep = if x < 0.01 { 181.0 } else { range / (60.0 * x) };

        Self {
            this,
            range,
            rough_lat_sep,
            rough_long_sep,
        }
    }
}

impl Filter for RangeFilter {
    // returns true if the coordinate passes the filter
    fn filter(&self, location: &dyn Location) -> bool {
        let other = location.get_loc();
        if ((self.this.get_latitude() - other.get_latitude()).abs() < self.rough_lat_sep)
            & ((self.this.get_longitude() - other.get_longitude()).abs() < self.rough_long_sep)
        {
            self.this.distance_to(other) < self.range
        } else {
            false
        }
    }
}
pub struct InverseRangeFilter {
    this: Coordinate,
    range: f64,
}

impl InverseRangeFilter {
    pub fn new(this: Coordinate, range: f64) -> Self {
        Self {
            this,
            range,
        }
    }
}

impl Filter for InverseRangeFilter {
    // returns true if the coordinate passes the filter
    fn filter(&self, location: &dyn Location) -> bool {
        let other = location.get_loc();
        self.this.distance_to(other) >= self.range
    }
}

pub struct DeviationFilter {
    from: Coordinate,
    to: Coordinate,
    max_deviation: f64,
    heading_from: f64,
    heading_to: f64,
}

impl DeviationFilter {
    pub fn new(from: Coordinate, to: Coordinate, heading_from: f64, heading_to: f64,max_deviation: f64) -> Self {
        Self {
            from,
            to,
            max_deviation,
            heading_from: heading_from,
            heading_to: heading_to,
        }
    }

    fn get_deviation(&self, heading_from: f64, bearing_to_deg: f64) -> f64 {
        let mut raw_deviation = (bearing_to_deg - heading_from).abs();
        if raw_deviation > 180.0 {
            raw_deviation = 360.0 - raw_deviation;
        }
        raw_deviation
    }
}

impl Filter for DeviationFilter {
    // returns true if the coordinate passes the filter
    fn filter(&self, location: &dyn Location) -> bool {
        let deviation_to =
            self.get_deviation(self.heading_from, self.from.bearing_to_deg(location.get_loc()));
        let deviation_from =
            self.get_deviation(self.heading_to, location.get_loc().bearing_to_deg(&self.to));

        deviation_to < self.max_deviation && deviation_from < self.max_deviation
    }
}


pub struct VorFilter {}

impl VorFilter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Filter for VorFilter {
    fn filter(&self, location: &dyn Location) -> bool {
        if let Some(navaid) = location.as_any().downcast_ref::<Navaid>() {
            navaid.get_type() == NavaidType::Vor
        } else {
            false
        }
    }
}

pub struct NilFilter {}

impl NilFilter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Filter for NilFilter {
    fn filter(&self, _location: &dyn Location) -> bool {
        false
    }
}

// Filter that combines two filters with an AND operation
pub struct AndFilter {
    filters: Vec<Box<dyn Filter>>,
}

impl AndFilter {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn add(&mut self, filter: Box<dyn Filter>) {
        self.filters.push(filter);
    }
}

impl Filter for AndFilter {
    fn filter(&self, location: &dyn Location) -> bool {
        self.filters.iter().all(|f| f.filter(location))
    }
}

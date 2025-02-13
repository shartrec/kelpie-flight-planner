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

use std::{fmt, fs};
use std::any::Any;
use std::io::BufReader;
use std::sync::{Arc, RwLock};

use flate2::read;
use log::{error, warn};

use crate::earth::{FEET_PER_DEGREE, get_earth_model};
use crate::earth::coordinate::Coordinate;
use crate::util::airport_parser::AirportParserFG850;

use super::location::Location;

#[derive(Clone)]
pub struct Airport {
    id: String,
    coordinate: Coordinate,
    elevation: i32,
    control_tower: bool,
    runways: Arc<RwLock<Vec<Runway>>>,
    show_default_buildings: bool,
    taxiways: Arc<RwLock<Vec<Taxiway>>>,
    max_runway_length: i64,
    airport_type: Option<AirportType>,
    name: String,
}

impl Airport {
    //noinspection RsExternalLinter
    pub fn new(
        id: String,
        latitude: f64,
        longitude: f64,
        elevation: i32,
        airport_type: Option<AirportType>,
        control_tower: bool,
        show_default_buildings: bool,
        name: String,
        max_runway_length: i64,
    ) -> Self {
        Self {
            id,
            coordinate: Coordinate::new(latitude, longitude),
            elevation,
            control_tower,
            runways: Arc::new(RwLock::new(Vec::new())),
            show_default_buildings,
            taxiways: Arc::new(RwLock::new(Vec::new())),
            max_runway_length,
            airport_type,
            name,
        }
    }

    pub fn add_runway(&self, runway: Runway) {
        self.runways
            .write()
            .expect("Can't get airport lock")
            .push(runway);
    }

    pub fn add_taxiway(&self, taxiway: Taxiway) {
        self.taxiways
            .write()
            .expect("Can't get airport lock")
            .push(taxiway);
    }

    pub fn get_control_tower(&self) -> bool {
        self.control_tower
    }

    pub fn get_runway_count(&self) -> usize {
        self.get_runways()
            .read()
            .expect("Can't get airport lock")
            .len()
    }

    pub fn get_runways(&self) -> &Arc<RwLock<Vec<Runway>>> {
        let mut loaded = false;
        if !self
            .runways
            .read()
            .expect("Can't get airport lock")
            .is_empty()
        {
            loaded = true;
        }
        if !loaded {
            self.load_runways_and_taxiways()
        }
        &self.runways
    }

    pub fn get_longest_runway(&self) -> Option<Runway> {
        let binding = self.get_runways()
            .read()
            .expect("Can't get airport lock");
        let runway = binding
            .iter()
            .max_by_key(|runway| runway.get_length());
        runway.cloned()
    }

    pub fn get_show_default_buildings(&self) -> bool {
        self.show_default_buildings
    }

    pub fn get_taxiways(&self) -> &Arc<RwLock<Vec<Taxiway>>> {
        // We check runways here as all airports have a runway, but in FlightGear maybe no Taxiways defined
        let mut loaded = false;
        if !self
            .runways
            .read()
            .expect("Can't get airport lock")
            .is_empty()
        {
            loaded = true;
        }
        if !loaded {
            self.load_runways_and_taxiways()
        }
        &self.taxiways
    }

    pub fn get_type(&self) -> Option<AirportType> {
        self.airport_type.clone()
    }

    pub fn load_runways_and_taxiways(&self) {
        let pref = crate::preference::manager();

        let runway_offsets = get_earth_model().get_runway_offsets();

        let file = match pref.get::<String>(crate::preference::AIRPORTS_PATH) {
            Some(p) => fs::File::open(p),
            None => {
                error!("Path to airports file not found");
                return;
            }
        };
        match file {
            Ok(input) => {
                let parser = AirportParserFG850::new();
                let decoder = read::GzDecoder::new(input);
                let mut reader = BufReader::new(decoder);
                let result = parser.load_runways(self, runway_offsets, &mut reader);
                if let Err(e) = result {
                    warn!("{}", e)
                }
            }
            Err(_e) => warn!("Unable to open airport data"),
        }
    }

    pub fn get_max_runway_length(&self) -> i64 {
        self.max_runway_length
    }

    pub fn set_max_runway_length(&mut self, runway_length: i64) {
        self.max_runway_length = runway_length;
    }

    pub fn calc_airport_extent(&self) -> [f64; 4] {
        // Start of by assuming just the airport with no runways or taxiways
        let mut min_lat = *self.get_lat();
        let mut max_lat = *self.get_lat();
        let mut min_long = *self.get_long();
        let mut max_long = *self.get_long();

        // for each runway get its extent
        for runway in self.runways.read().expect("Can't get airport lock").iter() {
            let extent = self.calc_extent(
                runway.lat,
                runway.long,
                runway.heading,
                runway.length,
                runway.width,
            );
            if extent[0] < min_lat {
                min_lat = extent[0];
            }
            if extent[1] > max_lat {
                max_lat = extent[1];
            }
            if extent[2] < min_long {
                min_long = extent[2];
            }
            if extent[3] > max_long {
                max_long = extent[3];
            }
        }

        // for each taxiway get its extent
        for taxiway in self.taxiways.read().expect("Can't get airport lock").iter() {
            let nodes = taxiway.get_nodes();
            for node in nodes {
                if node.get_lat() < min_lat {
                    min_lat = node.get_lat();
                }
                if node.get_lat() > max_lat {
                    max_lat = node.get_lat();
                }
                if node.get_long() < min_long {
                    min_long = node.get_long();
                }
                if node.get_long() > max_long {
                    max_long = node.get_long();
                }
            }
        }

        [min_lat, max_lat, min_long, max_long]
    }

    ///
    ///     Calculate the extent of a runway or taxiway
    ///     @param runway
    ///     return
    ///
    fn calc_extent(&self, lat: f64, lon: f64, heading: f64, length: i32, width: i32) -> [f64; 4] {
        let mut extent: [f64; 4] = [0.0; 4];

        let heading_radians = heading.to_radians();

        // Corner offset component contributed by the runway length
        let feet_east_l = (length as f64 / 2.0) * heading_radians.sin();
        let feet_north_l = (length as f64 / 2.0) * heading_radians.cos();

        // Corner offset component contributed by the runway width
        let feet_east_w = (width as f64 / 2.0) * heading_radians.cos();
        let feet_north_w = (width as f64 / 2.0) * heading_radians.sin();

        // Get the corner offsets (Corners A,B,C,D)
        let a_lat = feet_north_l + feet_north_w;
        let a_long = feet_east_l - feet_east_w;

        let b_lat = feet_north_l - feet_north_w;
        let b_long = feet_east_l + feet_east_w;

        let c_lat = -feet_north_l - feet_north_w;
        let c_long = -feet_east_l + feet_east_w;

        let d_lat = -feet_north_l + feet_north_w;
        let d_long = -feet_east_l - feet_east_w;

        // Calculate the min and max lat and long
        // This is not obvious because any of the values could be negative
        let max_lat = a_lat.max(b_lat).max(c_lat).max(d_lat);
        let min_lat = a_lat.min(b_lat).min(c_lat).min(d_lat);
        let max_long = a_long.max(b_long).max(c_long).max(d_long);
        let min_long = a_long.min(b_long).min(c_long).min(d_long);

        // Convert these back to degrees lat and long and offset from the center
        extent[0] = lat + (min_lat / FEET_PER_DEGREE as f64);
        extent[1] = lat + (max_lat / FEET_PER_DEGREE as f64);
        extent[2] = lon + (min_long / (FEET_PER_DEGREE as f64 * lat.to_radians().cos()));
        extent[3] = lon + (max_long / (FEET_PER_DEGREE as f64 * lat.to_radians().cos()));

        extent
    }

    pub fn get_ils(&self, runway_id: &str) -> Option<f64> {
        let ils = get_earth_model()
            .get_ils()
            .read()
            .expect("can't get ils lock");
        match ils.get(&self.id) {
            Some(list) => {
                let mut freq = None;
                for tuple in list {
                    if tuple.0 == runway_id {
                        freq = Some(tuple.1);
                        break;
                    }
                }
                freq
            }
            None => None,
        }
    }

    pub fn set_coordinate(&mut self, latitude: f64, longitude: f64,) {
        self.coordinate = Coordinate::new(latitude, longitude);
    }
}

impl Location for Airport {
    fn get_elevation(&self) -> &i32 {
        &self.elevation
    }

    fn get_id(&self) -> &str {
        self.id.as_str()
    }

    fn get_lat(&self) -> &f64 {
        self.coordinate.get_latitude()
    }

    fn get_lat_as_string(&self) -> String {
        self.coordinate.get_latitude_as_string().clone()
    }

    fn get_long(&self) -> &f64 {
        self.coordinate.get_longitude()
    }

    fn get_long_as_string(&self) -> String {
        self.coordinate.get_longitude_as_string().clone()
    }

    fn get_loc(&self) -> &Coordinate {
        &self.coordinate
    }

    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PartialEq for Airport {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Default for Airport {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            coordinate: Coordinate::new(0.0, 0.0),
            elevation: 0,
            control_tower: false,
            runways: Arc::new(RwLock::new(Vec::new())),
            show_default_buildings: false,
            taxiways: Arc::new(RwLock::new(Vec::new())),
            max_runway_length: 0,
            airport_type: None,
            name: "".to_string(),
        }
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct Runway {
    number: String,
    runway_type: Option<RunwayType>,
    lat: f64,
    long: f64,
    heading: f64,
    length: i32,
    width: i32,
    centre_line_lights: bool,
    surface: String,
    edge_lights: String,
}

impl Runway {
    //noinspection RsExternalLinter
    pub fn new(
        number: String,
        runway_type: Option<RunwayType>,
        lat: f64,
        long: f64,
        length: i32,
        width: i32,
        heading: f64,
        centre_line_lights: bool,
        surface: String,
        edge_lights: String,
    ) -> Self {
        Self {
            number,
            runway_type,
            lat,
            long,
            heading,
            length,
            width,
            centre_line_lights,
            surface,
            edge_lights,
        }
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

    pub fn get_surface(&self) -> &str {
        &self.surface
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_number(&self) -> &str {
        &self.number
    }

    pub(crate) fn get_opposite_number(&self) -> String {
        match self.number.as_str() {
            "N" => "S".to_string(),
            "S" => "N".to_string(),
            "E" => "W".to_string(),
            "W" => "E".to_string(),
            _ => {
                let (heading_part, extra_part) = if self.number.ends_with('R') {
                    (self.number.trim_end_matches('R'), "L")
                } else if self.number.ends_with('L') {
                    (self.number.trim_end_matches('L'), "R")
                } else if self.number.ends_with('C') {
                    (self.number.trim_end_matches('C'), "C")
                } else {
                    (self.number.as_str(), "")
                };

                let x = heading_part.parse::<i32>().unwrap_or(0);
                format!("{:02}{}", if x < 18 { x + 18 } else { x - 18 }, extra_part)
            }
        }
    }

    pub fn get_number_pair(&self) -> String {
        format!("{}/{}", self.get_number(), self.get_opposite_number())
    }
    pub fn get_runway_type(&self) -> Option<RunwayType> {
        self.runway_type.clone()
    }
}

impl fmt::Display for Runway {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Taxiway {} - Length: {}, Width: {}, Surface: {}",
            self.number, self.length, self.width, self.surface
        )
    }
}

#[derive(Default, Clone, Debug)]
pub struct Taxiway {
    nodes: Vec<LayoutNode>,
}

impl Taxiway {
    pub fn new(nodes: Vec<LayoutNode>) -> Self {
        Self { nodes }
    }

    pub fn get_nodes(&self) -> &Vec<LayoutNode> {
        &self.nodes
    }
}

#[derive(Default, Clone, Debug)]
pub struct LayoutNode {
    _type: String,
    lat: f64,
    long: f64,
    bezier_lat: f64,
    bezier_long: f64,
}

impl LayoutNode {
    pub fn new(_type: String, lat: f64, long: f64, bezier_lat: f64, bezier_long: f64) -> Self {
        Self {
            _type,
            lat,
            long,
            bezier_lat,
            bezier_long,
        }
    }

    pub fn get_type(&self) -> &str {
        &self._type
    }
    pub fn get_lat(&self) -> f64 {
        self.lat
    }

    pub fn get_long(&self) -> f64 {
        self.long
    }
    pub fn get_bezier_lat(&self) -> f64 {
        self.bezier_lat
    }

    pub fn get_bezier_long(&self) -> f64 {
        self.bezier_long
    }
}

#[derive(Clone, PartialEq)]
pub enum AirportType {
    Airport,
    SeaBase,
    Heliport,
}

impl AirportType {
    pub fn type_for(airport_type: &str) -> Option<AirportType> {
        if airport_type == "1" {
            Some(AirportType::Airport)
        } else if airport_type == "16" {
            Some(AirportType::SeaBase)
        } else if airport_type == "17" {
            Some(AirportType::Heliport)
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum RunwayType {
    Runway,
    WaterRunway,
    Helipad,
}

impl RunwayType {
    pub fn type_for(runway_type: &str) -> Option<RunwayType> {
        if runway_type == "100" {
            Some(RunwayType::Runway)
        } else if runway_type == "101" {
            Some(RunwayType::WaterRunway)
        } else if runway_type == "102" {
            Some(RunwayType::Helipad)
        } else {
            None
        }
    }
}

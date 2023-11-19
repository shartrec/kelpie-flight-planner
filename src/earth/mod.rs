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

use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::sync::{Arc, RwLock};
use flate2::read;

use lazy_static::lazy_static;
use log::info;

use crate::event;
use crate::event::Event;
use crate::model::airport::Airport;
use crate::model::fix::Fix;
use crate::model::location::Location;
use crate::model::navaid::Navaid;
use crate::util::airport_parser::AirportParserFG850;
use crate::util::fix_parser::FixParserFG;
use crate::util::navaid_parser::NavaidParserFG;

pub mod coordinate;
pub mod geomagnetism;
pub(crate) mod shapefile;
pub(crate) mod spherical_projector;

pub const FEET_PER_DEGREE: i32 = 6076 * 60;

lazy_static! {
    static ref EARTH: Earth = Earth {
        airports: Arc::new(RwLock::new(Vec::new())),
        navaids: Arc::new(RwLock::new(Vec::new())),
        fixes: Arc::new(RwLock::new(Vec::new())),
        runway_offsets: Arc::new(RwLock::new(HashMap::new())),
        ils: Arc::new(RwLock::new(HashMap::new())),
    };
}

pub struct Earth {
    airports: Arc<RwLock<Vec<Arc<Airport>>>>,
    navaids: Arc<RwLock<Vec<Arc<Navaid>>>>,
    fixes: Arc<RwLock<Vec<Arc<Fix>>>>,
    runway_offsets: Arc<RwLock<HashMap<String, usize>>>,
    ils: Arc<RwLock<HashMap<String, Vec<(String, f64)>>>>,
}

impl Earth {
    pub fn get_airports(&self) -> &Arc<RwLock<Vec<Arc<Airport>>>> {
        &self.airports
    }

    pub fn get_airport_by_id(&self, id: &str) -> Option<Arc<Airport>> {
        self.airports
            .read()
            .unwrap()
            .iter()
            .find(|airport| airport.get_id() == id)
            .cloned()
    }

    pub fn set_airports(&self, airports: Vec<Arc<Airport>>) {
        self.airports.write().unwrap().clear();
        self.airports.write().unwrap().extend(airports);
    }

    pub fn get_navaid_by_id_and_name(&self, id: &str, name: &str) -> Option<Arc<Navaid>> {
        self.navaids
            .read()
            .unwrap()
            .iter()
            .find(|navaid| navaid.get_id() == id && navaid.get_name().eq_ignore_ascii_case(name))
            .cloned()
    }

    pub fn get_navaids(&self) -> &Arc<RwLock<Vec<Arc<Navaid>>>> {
        &self.navaids
    }

    pub fn set_navaids(&self, navaids: Vec<Arc<Navaid>>) {
        self.navaids.write().unwrap().clear();
        self.navaids.write().unwrap().extend(navaids);
    }

    pub fn get_ils(&self) -> &Arc<RwLock<HashMap<String, Vec<(String, f64)>>>> {
        &self.ils
    }

    pub fn set_ils(&self, navaids: HashMap<String, Vec<(String, f64)>>) {
        self.ils.write().unwrap().clear();
        self.ils.write().unwrap().extend(navaids);
    }

    pub fn get_fix_by_id(&self, id: &str) -> Option<Arc<Fix>> {
        self.fixes
            .read()
            .unwrap()
            .iter()
            .find(|fix| fix.get_id() == id)
            .cloned()
    }

    pub fn get_fixes(&self) -> &Arc<RwLock<Vec<Arc<Fix>>>> {
        &self.fixes
    }

    pub fn set_fixes(&self, fixes: Vec<Arc<Fix>>) {
        self.fixes.write().unwrap().clear();
        self.fixes.write().unwrap().extend(fixes);
    }

    pub fn set_runway_offsets(&self, runway_offsets: HashMap<String, usize>) {
        self.runway_offsets.write().unwrap().clear();
        self.runway_offsets.write().unwrap().extend(runway_offsets);
    }

    pub fn get_runway_offsets(&self) -> &Arc<RwLock<HashMap<String, usize>>> {
        &self.runway_offsets
    }
}

pub fn get_earth_model() -> &'static Earth {
    &EARTH
}

pub fn initialise() {
    let timer = std::time::Instant::now();
    let pref = crate::preference::manager();
    match pref.get::<String>(crate::preference::AIRPORTS_PATH) {
        Some(p) => load_airports(&p),
        None => (),
    }
    info!("Airports loaded in {:?}", timer.elapsed());
    match pref.get::<String>(crate::preference::NAVAIDS_PATH) {
        Some(p) => load_navaids(&p),
        None => (),
    }
    info!("Navaids loaded in {:?}", timer.elapsed());
    match pref.get::<String>(crate::preference::FIXES_PATH) {
        Some(p) => load_fixes(&p),
        None => (),
    }
    info!("Fixes loaded in {:?}", timer.elapsed());
}

fn load_airports(path: &str) {
    let mut airports: Vec<Arc<Airport>> = Vec::new();
    let mut runway_offsets = HashMap::with_capacity(25000);
    let file = fs::File::open(path);
    match file {
        Ok(input) => {
            let decoder = read::GzDecoder::new(input);
            let mut reader = BufReader::new(decoder);

            let mut parser = AirportParserFG850::new();
            match parser.load_airports(&mut airports, &mut runway_offsets, &mut reader) {
                Ok(()) => (),
                Err(msg) => panic! {"{}", msg},
            }
        }
        Err(_e) => panic!("Unable to open test airport data"),
    }
    get_earth_model().set_airports(airports);
    get_earth_model().set_runway_offsets(runway_offsets);
    event::manager().notify_listeners(Event::AirportsLoaded);
}

fn load_navaids(path: &str) {
    let mut navaids: Vec<Arc<Navaid>> = Vec::new();
    let mut ils: HashMap<String, Vec<(String, f64)>> = HashMap::new();
    let file = fs::File::open(path);
    match file {
        Ok(input) => {
            let mut parser = NavaidParserFG {};
            let decoder = read::GzDecoder::new(input);
            let mut reader = BufReader::new(decoder);
            match parser.load_navaids(&mut navaids, &mut ils, &mut reader) {
                Ok(()) => (),
                Err(msg) => panic! {"{}", msg},
            }
        }
        Err(_) => panic!("Unable to open test navaid data"),
    }
    get_earth_model().set_navaids(navaids);
    get_earth_model().set_ils(ils);
    event::manager().notify_listeners(Event::NavaidsLoaded);
}

fn load_fixes(path: &str) {
    let mut fixes: Vec<Arc<Fix>> = Vec::new();
    let file = fs::File::open(path);
    match file {
        Ok(input) => {
            let mut parser = FixParserFG {};
            let decoder = read::GzDecoder::new(input);
            let mut reader = BufReader::new(decoder);
            match parser.load_fixes(&mut fixes, &mut reader) {
                Ok(()) => (),
                Err(msg) => panic! {"{}", msg},
            }
        }
        Err(_) => panic!("Unable to open test fix data"),
    }
    get_earth_model().set_fixes(fixes);
    event::manager().notify_listeners(Event::FixesLoaded);
}

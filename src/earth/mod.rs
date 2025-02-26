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
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Error};
use std::sync::{Arc, LazyLock, RwLock};

use flate2::read;
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

pub mod geomagnetism;
pub(crate) mod shapefile;
pub(crate) mod spherical_projector;
pub(crate) mod airport_list_model;
pub(crate) mod navaid_list_model;
pub(crate) mod fix_list_model;

pub const FEET_PER_DEGREE: i32 = 6076 * 60;


static EARTH: LazyLock<Earth> = LazyLock::new(|| Earth {
    airports: RwLock::new(Vec::new()),
    navaids: RwLock::new(Vec::new()),
    fixes: RwLock::new(Vec::new()),
    runway_offsets: RwLock::new(HashMap::new()),
    ils: RwLock::new(HashMap::new()),
});

pub struct Earth {
    airports: RwLock<Vec<Arc<Airport>>>,
    navaids: RwLock<Vec<Arc<Navaid>>>,
    fixes: RwLock<Vec<Arc<Fix>>>,
    runway_offsets: RwLock<HashMap<String, usize>>,
    ils: RwLock<HashMap<String, Vec<(String, f64)>>>,
}

impl Earth {
    pub fn get_airports(&self) -> &RwLock<Vec<Arc<Airport>>> {
        &self.airports
    }

    pub fn get_airport_by_id(&self, id: &str) -> Option<Arc<Airport>> {
        self.airports
            .read()
            .expect("Unable to get lock on Airports")
            .iter()
            .find(|airport| airport.get_id() == id)
            .cloned()
    }

    pub fn set_airports(&self, airports: Vec<Arc<Airport>>) {
        let mut aps = self.airports.write().expect("Unable to get lock on Airports");
        aps.clear();
        aps.extend(airports);
    }

    pub fn get_navaid_by_id_and_name(&self, id: &str, name: &str) -> Option<Arc<Navaid>> {
        self.navaids
            .read()
            .expect("Unable to get lock on Navaids")
            .iter()
            .find(|navaid| navaid.get_id() == id && navaid.get_name().eq_ignore_ascii_case(name))
            .cloned()
    }

    pub fn get_navaids(&self) -> &RwLock<Vec<Arc<Navaid>>> {
        &self.navaids
    }

    pub fn set_navaids(&self, navaids: Vec<Arc<Navaid>>) {
        let mut navs = self.navaids.write().expect("Unable to get lock on Navaids");
        navs.clear();
        navs.extend(navaids);
    }

    pub fn get_ils(&self) -> &RwLock<HashMap<String, Vec<(String, f64)>>> {
        &self.ils
    }

    pub fn set_ils(&self, navaids: HashMap<String, Vec<(String, f64)>>) {
        let mut ils = self.ils.write().expect("Unable to get lock on Ils");
        ils.clear();
        ils.extend(navaids);
    }

    pub fn get_fix_by_id(&self, id: &str) -> Option<Arc<Fix>> {
        self.fixes
            .read()
            .expect("Unable to get lock on Ils")
            .iter()
            .find(|fix| fix.get_id() == id)
            .cloned()
    }

    pub fn get_fixes(&self) -> &RwLock<Vec<Arc<Fix>>> {
        &self.fixes
    }

    pub fn set_fixes(&self, fixes: Vec<Arc<Fix>>) {
        let mut fxs = self.fixes.write().expect("Unable to get lock on fixes");
        fxs.clear();
        fxs.extend(fixes);
    }

    pub fn set_runway_offsets(&self, runway_offsets: HashMap<String, usize>) {
        let mut rns = self.runway_offsets.write().expect("Unable to get lock on runways");
        rns.clear();
        rns.extend(runway_offsets);
    }

    pub fn get_runway_offsets(&self) -> &RwLock<HashMap<String, usize>> {
        &self.runway_offsets
    }
}

pub fn get_earth_model() -> &'static Earth {
    &EARTH
}

pub fn initialise() -> Result<(), Error> {
    let timer = std::time::Instant::now();
    let pref = crate::preference::manager();
    match pref.get::<String>(crate::preference::AIRPORTS_PATH) {
        Some(p) => load_airports(&p)?,
        None => return Err(Error::new(std::io::ErrorKind::NotFound, "Flightgear Airport path not set")),
    }
    info!("{} Airports loaded in {:?}", get_earth_model().get_airports().read().expect("Unable to get lock on Airports").len(), timer.elapsed());
    let timer = std::time::Instant::now();
    match pref.get::<String>(crate::preference::NAVAIDS_PATH) {
        Some(p) => load_navaids(&p)?,
        None => return Err(Error::new(std::io::ErrorKind::NotFound, "Flightgear Navaid path not set")),
    }
    info!("{} Navaids loaded in {:?}", get_earth_model().get_navaids().read().expect("Unable to get lock on Navaids").len(), timer.elapsed());
    let timer = std::time::Instant::now();
    match pref.get::<String>(crate::preference::FIXES_PATH) {
        Some(p) => load_fixes(&p)?,
        None => return Err(Error::new(std::io::ErrorKind::NotFound, "Flightgear Fix path not set")),
    }
    info!("{} Fixes loaded in {:?}", get_earth_model().get_fixes().read().expect("Unable to get lock on Fixes").len(), timer.elapsed());
    Ok(())
}

fn load_airports(path: &str) -> Result<(), Error> {
    event::manager().notify_listeners(Event::StatusChange(format!("Loading Airports from : {}", path)));
    let mut airports: Vec<Arc<Airport>> = Vec::new();
    let mut runway_offsets = HashMap::with_capacity(25000);
    let file = fs::File::open(path);
    let result = match file {
        Ok(input) => {
            let decoder = read::GzDecoder::new(input);
            let mut reader = BufReader::new(decoder);
            let mut parser = AirportParserFG850::new();
            match parser.load_airports(&mut airports, &mut runway_offsets, &mut reader) {
                Ok(()) => {
                    get_earth_model().set_airports(airports);
                    get_earth_model().set_runway_offsets(runway_offsets);
                    event::manager().notify_listeners(Event::AirportsLoaded);
                    Ok(())
                }
                Err(msg) => Err(msg),
            }
        }
        Err(e) => Err(e),
    };
    event::manager().notify_listeners(Event::StatusChange("".to_string()));
    result
}

fn load_navaids(path: &str) -> Result<(), Error> {
    event::manager().notify_listeners(Event::StatusChange(format!("Loading Nav aids from : {}", path)));
    let mut navaids: Vec<Arc<Navaid>> = Vec::new();
    let mut ils: HashMap<String, Vec<(String, f64)>> = HashMap::new();
    let file = fs::File::open(path);
    let result = match file {
        Ok(input) => {
            let mut parser = NavaidParserFG {};
            let decoder = read::GzDecoder::new(input);
            let mut reader = BufReader::new(decoder);
            match parser.load_navaids(&mut navaids, &mut ils, &mut reader) {
                Ok(()) => {
                    get_earth_model().set_navaids(navaids);
                    get_earth_model().set_ils(ils);
                    event::manager().notify_listeners(Event::NavaidsLoaded);
                    Ok(())
                }
                Err(msg) => Err(msg),
            }
        }
        Err(e) => Err(e),
    };
    event::manager().notify_listeners(Event::StatusChange("".to_string()));
    result
}

fn load_fixes(path: &str) -> Result<(), Error> {
    event::manager().notify_listeners(Event::StatusChange(format!("Loading Fixes from : {}", path)));
    let mut fixes: Vec<Arc<Fix>> = Vec::new();
    let file = fs::File::open(path);
    let result = match file {
        Ok(input) => {
            let mut parser = FixParserFG {};
            let decoder = read::GzDecoder::new(input);
            let mut reader = BufReader::new(decoder);
            match parser.load_fixes(&mut fixes, &mut reader) {
                Ok(()) => {
                    get_earth_model().set_fixes(fixes);
                    event::manager().notify_listeners(Event::FixesLoaded);
                    Ok(())
                }
                Err(msg) => Err(msg),
            }
        }
        Err(e) => Err(e),
    };
    event::manager().notify_listeners(Event::StatusChange("".to_string()));
    result
}



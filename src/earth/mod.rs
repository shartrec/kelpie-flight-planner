use std::cmp::Ord;
use std::fs;
use std::io::BufReader;
use std::sync::{Arc, RwLock};

use gtk::glib::clone::Downgrade;
use gtk::glib::Sender;
use lazy_static::lazy_static;
use log::info;
use crate::event::Event;

use crate::model::airport::Airport;
use crate::model::fix::Fix;
use crate::model::location::Location;
use crate::model::navaid::Navaid;
use crate::util::airport_parser::AirportParserFG850;
use crate::util::fix_parser::FixParserFG;
use crate::util::navaid_parser::NavaidParserFG;

pub mod coordinate;

pub const FEET_PER_DEGREE: i32 = 6076 * 60;

lazy_static! {
    static ref EARTH: Earth = Earth {
        airports: Arc::new(RwLock::new(Vec::new())),
        navaids: Arc::new(RwLock::new(Vec::new())),
        fixes: Arc::new(RwLock::new(Vec::new())),
    };
}

pub struct Earth {
    airports: Arc<RwLock<Vec<Box<Airport>>>>,
    navaids: Arc<RwLock<Vec<Box<Navaid>>>>,
    fixes: Arc<RwLock<Vec<Box<Fix>>>>,
}

impl Earth {
    pub fn get_airports(&self) -> &Arc<RwLock<Vec<Box<Airport>>>> {
        &self.airports
    }

    pub fn get_airport_by_id(&self, id: &str) -> Option<Box<Airport>> {
        self.airports
            .read()
            .unwrap()
            .iter()
            .find(|airport| airport.get_id() == id)
            .cloned()
    }

    pub fn set_airports(&self, airports: Vec<Box<Airport>>) {
        self.airports.write().unwrap().clear();
        self.airports.write().unwrap().extend(airports);
    }

    pub fn get_navaid_by_id_and_name(&self, id: &str, name: &str) -> Option<Box<Navaid>> {
        self.navaids
            .read()
            .unwrap()
            .iter()
            .find(|navaid| navaid.get_id() == id && navaid.get_name().eq_ignore_ascii_case(name))
            .cloned()
    }

    pub fn get_navaids(&self) -> &Arc<RwLock<Vec<Box<Navaid>>>> {
        &self.navaids
    }

    pub fn set_navaids(&self, navaids: Vec<Box<Navaid>>) {
        self.navaids.write().unwrap().clear();
        self.navaids.write().unwrap().extend(navaids);
    }

    pub fn get_fix_by_id(&self, id: &str) -> Option<Box<Fix>> {
        self.fixes
            .read()
            .unwrap()
            .iter()
            .find(|fix| fix.get_id() == id)
            .cloned()
    }

    pub fn get_fixes(&self) -> &Arc<RwLock<Vec<Box<Fix>>>> {
        &self.fixes
    }

    pub fn set_fixes(&self, fixes: Vec<Box<Fix>>) {
        self.fixes.write().unwrap().clear();
        self.fixes.write().unwrap().extend(fixes);
    }

}

pub fn get_earth_model() -> &'static Earth {
    &EARTH
}

pub fn initialise(transmitter: Sender<Event>) {
    let timer = std::time::Instant::now();
    let pref = crate::preference::manager();
    match pref.get::<String>(crate::preference::AIRPORTS_PATH) {
        Some(p) => load_airports(&p, &transmitter),
        None => (),
    }
    info!("Airports loaded in {:?}", timer.elapsed());
    match pref.get::<String>(crate::preference::NAVAIDS_PATH) {
        Some(p) => load_navaids(&p, &transmitter),
        None => (),
    }
    info!("Navaids loaded in {:?}", timer.elapsed());
    match pref.get::<String>(crate::preference::FIXES_PATH) {
        Some(p) => load_fixes(&p, &transmitter),
        None => (),
    }
    info!("Fixes loaded in {:?}", timer.elapsed());
}

fn load_airports(path: &str, transmitter: &Sender<Event>) {
    let mut airports: Vec<Box<Airport>> = Vec::new();
    let file = fs::File::open(path);
    match file {
        Ok(f) => {
            let mut parser = AirportParserFG850::new();
            let mut reader = BufReader::new(f);
            match parser.load_airports(&mut airports, &mut reader) {
                Ok(()) => (),
                Err(msg) => panic! {"{}", msg},
            }
        }
        Err(_e) => panic!("Unable to open test airport data"),
    }
    get_earth_model().set_airports(airports);
    let _ = transmitter.send(Event::AirportsLoaded);
}

fn load_navaids(path: &str, transmitter: &Sender<Event>) {
    let mut navaids: Vec<Box<Navaid>>= Vec::new();
    let file = fs::File::open(path);
    match file {
        Ok(f) => {
            let mut parser = NavaidParserFG {};
            let mut reader = BufReader::new(f);
            match parser.load_navaids(&mut navaids, &mut reader) {
                Ok(()) => (),
                Err(msg) => panic! {"{}", msg},
            }
        }
        Err(e) => panic!("Unable to open test navaid data"),
    }
    get_earth_model().set_navaids(navaids);
    let _ = transmitter.send(Event::NavaidsLoaded);
}

fn load_fixes(path: &str, transmitter: &Sender<Event>) {
    let mut fixes: Vec<Box<Fix>> = Vec::new();
    let file = fs::File::open(path);
    match file {
        Ok(f) => {
            let mut parser = FixParserFG {};
            let mut reader = BufReader::new(f);
            match parser.load_fixes(&mut fixes, &mut reader) {
                Ok(()) => (),
                Err(msg) => panic! {"{}", msg},
            }
        }
        Err(e) => panic!("Unable to open test fix data"),
    }
    get_earth_model().set_fixes(fixes);
    let _ = transmitter.send(Event::FixesLoaded);
}

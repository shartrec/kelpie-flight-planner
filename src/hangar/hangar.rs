/*
 * Copyright (c) 2003-2023. Trevor Campbell and others.
 */
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use app_dirs::*;
use lazy_static::lazy_static;
use log::{error, warn};
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};
use yaml_rust::yaml::Hash;

use crate::model::aircraft::Aircraft;
use crate::preference::APP_INFO;

// This is where all the planes live.
lazy_static! {
    static ref HANGAR: Hangar = Hangar {
        aircraft: load_hangar(),
    };
}

static DEFAULT_AICRAFT: &str = "---
- climb-rate: 5000
  climb-speed: 250
  cruise-altitude: 35000
  cruise-speed: 490
  is-default: false
  name: 777-200B
  sink-rate: 3000
  sink-speed: 280
- climb-rate: 2000
  climb-speed: 300
  cruise-altitude: 36000
  cruise-speed: 450
  is-default: false
  name: Boeing 737
  sink-rate: 1000
  sink-speed: 200
- climb-rate: 1000
  climb-speed: 110
  cruise-altitude: 7000
  cruise-speed: 140
  is-default: true
  name: Cessna C-172 - High wing
  sink-rate: 500
  sink-speed: 100
";

pub struct Hangar {
    aircraft: Arc<RwLock<Vec<Aircraft>>>,
}

impl Hangar {
    pub fn get_default_aircraft(&self) -> Option<Aircraft> {
        let aircraft = self
            .aircraft
            .read()
            .expect("Unable to get a lock on the aircraft hangar");
        for a in aircraft.iter() {
            if *a.is_default() {
                return Some(a.clone());
            }
        }
        None
    }

    pub fn get_all(&self) -> Arc<RwLock<Vec<Aircraft>>> {
        self.aircraft.clone()
    }

    pub fn get_hangar() -> &'static Hangar {
        &HANGAR
    }
}

const KEY_NAME: &'static str = "name";
const KEY_CRUISE_SPEED: &'static str = "cruise-speed";
const KEY_CRUISE_ALTITUDE: &'static str = "cruise-altitude";
const KEY_CLIMB_SPEED: &'static str = "climb-speed";
const KEY_CLIMB_RATE: &'static str = "climb-rate";
const KEY_SINK_SPEED: &'static str = "sink-speed";
const KEY_SINK_RATE: &'static str = "sink-rate";
const KEY_IS_DEFAULT: &'static str = "is-default";

// Load aircraft from yaml file
pub fn load_hangar() -> Arc<RwLock<Vec<Aircraft>>> {
    let path = get_hangar_path();

    let mut contents = String::new();
    match File::open(path) {
        Ok(mut file) => {
            file.read_to_string(&mut contents)
                .expect("Unable to read file");
        }
        Err(_) => {
            contents = DEFAULT_AICRAFT.to_string();
        }
    }
    let mut hangar = Vec::new();

    let docs = YamlLoader::load_from_str(&contents).unwrap();
    for doc in docs {
        if let Some(all) = doc.as_vec() {
            for each in all {
                if let Some(map) = each.as_hash() {
                    let aircraft = Aircraft::new(
                        get_string(map, KEY_NAME),
                        get_i32(map, KEY_CRUISE_SPEED),
                        get_i32(map, KEY_CRUISE_ALTITUDE),
                        get_i32(map, KEY_CLIMB_SPEED),
                        get_i32(map, KEY_CLIMB_RATE),
                        get_i32(map, KEY_SINK_SPEED),
                        get_i32(map, KEY_SINK_RATE),
                        get_bool(map, KEY_IS_DEFAULT),
                    );
                    hangar.push(aircraft);
                }
            }
        }
    }
    Arc::new(RwLock::new(hangar))
}

fn get_bool(map: &Hash, key: &str) -> bool {
    map.get(&Yaml::String(key.to_string()))
        .unwrap_or(&Yaml::Boolean(false))
        .as_bool()
        .unwrap_or(false)
}

fn get_i32(map: &Hash, key: &str) -> i32 {
    map.get(&Yaml::String(key.to_string()))
        .unwrap_or(&Yaml::Integer(0))
        .as_i64()
        .unwrap_or(0) as i32
}

fn get_string(map: &Hash, key: &str) -> String {
    map.get(&Yaml::String(key.to_string()))
        .unwrap_or(&Yaml::String("".to_string()))
        .as_str()
        .unwrap_or("")
        .to_string()
}

#[allow(dead_code)]
pub fn save_hangar() {
    let path = get_hangar_path();

    let hangar = Hangar::get_hangar().get_all();
    let all = hangar.read().expect("Unable to get read lock on hangar");

    let mut vec = Vec::new();

    // if let Some(mut vec) = vec.as_vec() {
    for a in all.iter() {
        let mut inner_map = Hash::new();
        put_string(&mut inner_map, KEY_NAME, a.get_name());
        put_i32(&mut inner_map, KEY_CRUISE_SPEED, a.get_cruise_speed());
        put_i32(&mut inner_map, KEY_CRUISE_ALTITUDE, a.get_cruise_altitude());
        put_i32(&mut inner_map, KEY_CLIMB_SPEED, a.get_climb_speed());
        put_i32(&mut inner_map, KEY_CLIMB_RATE, a.get_climb_rate());
        put_i32(&mut inner_map, KEY_SINK_SPEED, a.get_sink_speed());
        put_i32(&mut inner_map, KEY_SINK_RATE, a.get_sink_rate());
        put_bool(&mut inner_map, KEY_IS_DEFAULT, a.is_default());

        let map = Yaml::Hash(inner_map);

        vec.push(map);
    }
    let doc = Yaml::Array(vec);

    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(&doc).unwrap();

    match File::create(path) {
        Ok(mut f) => match f.write_all(out_str.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                warn!("Unable to save aircraft config : {}", err);
            }
        },
        Err(err) => {
            error!("Unable to save aircraft config : {}", err);
        }
    }
}

#[allow(dead_code)]
fn put_bool(map: &mut Hash, key: &str, v: &bool) {
    map.insert(Yaml::String(key.to_string()), Yaml::Boolean(v.clone()));
}

#[allow(dead_code)]
fn put_i32(map: &mut Hash, key: &str, v: &i32) {
    map.insert(Yaml::String(key.to_string()), Yaml::Integer(*v as i64));
}

#[allow(dead_code)]
fn put_string(map: &mut Hash, key: &str, v: &str) {
    map.insert(Yaml::String(key.to_string()), Yaml::String(v.to_string()));
}

pub fn get_hangar_path() -> PathBuf {
    get_app_dir(
        app_dirs::AppDataType::UserConfig,
        &APP_INFO,
        "aircraft.yaml",
    )
    .expect("Unable to build path for airport config")
}

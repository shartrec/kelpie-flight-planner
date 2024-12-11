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
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;

use dirs_next::config_dir;
use gtk::{gio, glib, subclass::prelude::*};
use lazy_static::lazy_static;
use log::{error, warn};
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};
use yaml_rust::yaml::Hash;

use crate::model::aircraft::Aircraft;
use crate::preference::APP_INFO;

// This is where all the planes live.
lazy_static! {
    static ref HANGAR: Hangar = Hangar::new();
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
// To use the Hangar as a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct Hangar(ObjectSubclass<imp::Hangar>) @implements gio::ListModel;
}

impl Hangar {
    pub fn new() -> Hangar {
        glib::Object::new()
    }
}

impl Default for Hangar {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use std::collections::BTreeMap;
    use std::sync::{Arc, RwLock};

    use gtk::{gio, glib, StringObject};
    use gtk::prelude::{ListModelExt, StaticType};
    use gtk::subclass::prelude::{ListModelImpl, ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt};

    use crate::hangar::hangar::{load_hangar, save_hangar};
    use crate::model::aircraft::Aircraft;

    #[derive(Default)]
    pub struct Hangar {
        pub aircraft: Arc<RwLock<BTreeMap<String, Arc<Aircraft>>>>,
    }

    impl Hangar {
        pub fn get_default_aircraft(&self) -> Option<Arc<Aircraft>> {
            let aircraft = self
                .aircraft
                .read()
                .expect("Unable to get a lock on the aircraft hangar");
            for (_name, a) in aircraft.iter() {
                if *a.is_default() {
                    return Some(a.clone());
                }
            }
            None
        }

        pub fn get_all(&self) -> &Arc<RwLock<BTreeMap<String, Arc<Aircraft>>>> {
            &self.aircraft
        }

        pub fn get(&self, name: &str) -> Option<Arc<Aircraft>> {
            self.aircraft.read().expect("Can't get aircraft lock").get(name).cloned()
        }

        pub fn put(&self, aircraft: Aircraft) {
            let name = aircraft.get_name().to_string().clone();
            let old_p = self.get_item_position(&name);
            let mut binding = self.aircraft.write().expect("Can't get aircraft lock");
            binding.insert(aircraft.get_name().to_string(), Arc::new(aircraft));
            drop(binding);
            self.save();
            if let Some(pos) = old_p {
                self.obj().items_changed(pos, 0, 0);
            } else if let Some(pos) = self.get_item_position(&name) {
                self.obj().items_changed(pos, 0, 1);
            }
        }

        pub fn remove(&self, name: &str) {
            let p = self.get_item_position(name);
            let mut binding = self.aircraft.write().expect("Can't get aircraft lock");
            binding.remove(name);
            drop(binding);
            self.save();
            if let Some(pos) = p {
                self.obj().items_changed(pos, 1, 0);
            }
        }

        pub fn set_default(&self, name: &str) {
            // We need to unset the old default and set the new default.
            if let Some(a) = self.get_default_aircraft() {
                let prior_default = Aircraft::new(
                    a.get_name().to_string(),
                    *a.get_cruise_speed(),
                    *a.get_cruise_altitude(),
                    *a.get_climb_speed(),
                    *a.get_climb_rate(),
                    *a.get_sink_speed(),
                    *a.get_sink_rate(),
                    false,
                );
                self.put(prior_default);
                if let Some(pos) = self.get_item_position(a.get_name()) {
                    self.obj().items_changed(pos, 1, 1);
                }
            }
            if let Some(a) = self.get(name) {
                let new_default = Aircraft::new(
                    a.get_name().to_string(),
                    *a.get_cruise_speed(),
                    *a.get_cruise_altitude(),
                    *a.get_climb_speed(),
                    *a.get_climb_rate(),
                    *a.get_sink_speed(),
                    *a.get_sink_rate(),
                    true,
                );
                self.put(new_default);
                if let Some(pos) = self.get_item_position(name) {
                    self.obj().items_changed(pos, 1, 1);
                }
            }
        }

        pub fn save(&self) {
            save_hangar();
        }

        fn get_item_position(&self, plane: &str) -> Option<u32> {
            let aircraft = self
                .aircraft
                .read()
                .expect("Unable to get a lock on the aircraft hangar");
            for (i, name) in (0_u32..).zip(aircraft.keys()) {
                if name == plane {
                    return Some(i);
                }
            }
            None
        }

        pub fn aircraft_at(&self, position: u32) -> Option<Arc<Aircraft>> {
            let aircraft = self
                .aircraft
                .read()
                .expect("Unable to get a lock on the aircraft hangar");
            aircraft.values().nth(position as usize).cloned()
        }
    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Hangar {
        const NAME: &'static str = "Hangar";
        type Type = super::Hangar;
        type Interfaces = (gio::ListModel, );
    }

    impl ObjectImpl for Hangar {
        fn constructed(&self) {
            self.parent_constructed();

            let mut aircraft = self.aircraft.write().expect("Can't get lock on aircraft");
            for a in load_hangar() {
                aircraft.insert(a.get_name().to_string(), a);
            }
        }
    }

    impl ListModelImpl for Hangar {
        fn item_type(&self) -> glib::Type {
            StringObject::static_type()
        }

        fn n_items(&self) -> u32 {
            let aircraft = self
                .aircraft
                .read()
                .expect("Unable to get a lock on the aircraft hangar");
            aircraft.len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            match self.aircraft_at(position) {
                Some(plane) => {
                    let mut name_string = plane.get_name().to_string();
                    // Get the aircraft and see if it is the default
                    if *plane.is_default() {
                        name_string.push('*');
                    }
                    Some(glib::Object::from(StringObject::new(name_string.as_str())))
                }

                None => None
            }
        }
    }
}

const KEY_NAME: &str = "name";
const KEY_CRUISE_SPEED: &str = "cruise-speed";
const KEY_CRUISE_ALTITUDE: &str = "cruise-altitude";
const KEY_CLIMB_SPEED: &str = "climb-speed";
const KEY_CLIMB_RATE: &str = "climb-rate";
const KEY_SINK_SPEED: &str = "sink-speed";
const KEY_SINK_RATE: &str = "sink-rate";
const KEY_IS_DEFAULT: &str = "is-default";

pub fn get_hangar() -> &'static Hangar {
    &HANGAR
}

// Load aircraft from yaml file
pub fn load_hangar() -> Vec<Arc<Aircraft>> {
    let mut hangar: Vec<Arc<Aircraft>> = Vec::new();
    if let Some(path) = get_hangar_path() {
        let mut contents = String::new();
        match File::open(&path) {
            Ok(mut file) => {
                file.read_to_string(&mut contents)
                    .expect("Unable to read file");
            }
            Err(_) => {
                contents = DEFAULT_AICRAFT.to_string();
            }
        }

        match YamlLoader::load_from_str(&contents) {
            Ok(docs) => {
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
                                hangar.push(Arc::new(aircraft));
                            }
                        }
                    }
                }
            }
            Err(_) => {
                error!("Unable to load aircraft configuration from {:?}", &path);
            }
        }
    }
    hangar
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
    if let Some(path) = get_hangar_path() {
        let hangar = get_hangar().imp().get_all();
        let all = hangar.read().expect("Unable to get read lock on hangar");

        let mut vec = Vec::new();

        // if let Some(mut vec) = vec.as_vec() {
        for (_name, a) in all.iter() {
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
        match emitter.dump(&doc) {
            Ok(_) => {
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
            Err(err) => {
                error!("Unable to save aircraft config : {}", err);
            }
        }
    } else {
        error!("Unable to get config path for hangar");
        error!("Unable to save aircraft config");
    }
}

#[allow(dead_code)]
fn put_bool(map: &mut Hash, key: &str, v: &bool) {
    map.insert(Yaml::String(key.to_string()), Yaml::Boolean(*v));
}

#[allow(dead_code)]
fn put_i32(map: &mut Hash, key: &str, v: &i32) {
    map.insert(Yaml::String(key.to_string()), Yaml::Integer(*v as i64));
}

#[allow(dead_code)]
fn put_string(map: &mut Hash, key: &str, v: &str) {
    map.insert(Yaml::String(key.to_string()), Yaml::String(v.to_string()));
}

pub fn get_hangar_path() -> Option<PathBuf> {
    let x = config_dir().map(|mut p| {
        p.push(&APP_INFO.name);
        p.push("aircraft.yaml");
        p
    });
    x
}

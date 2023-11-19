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

#![allow(unused)]

use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use lazy_static::lazy_static;
use log::error;
use preferences::{AppInfo, Preferences, PreferencesMap};

use crate::event;
use crate::event::Event;

const PREFS_PATH: &str = "planner";
pub const APP_INFO: AppInfo = AppInfo {
    name: "kelpie-flight-planner",
    author: "shartrec.com",
};

// Preference constants
pub const INITIALIZED: &str = "Initialized";
pub const ANIMATE_MAP: &str = "AnimateMap";
pub const SHOW_MAP_IN_BROWSER: &str = "ShowMapInBrowser";
pub const UNITS: &str = "Units";
pub const UNITS_NM: &str = "Nm";
pub const UNITS_KM: &str = "Km";
pub const UNITS_MI: &str = "Mi";
pub const NAVIGATION_DATA: &str = "NavigationData.Type";
pub const FGFS_DIR: &str = "Fgfs.Dir";
pub const FGFS_USE_DFT_PATH: &str = "Fgfs.UseDefaultPath";
pub const AIRPORTS_PATH: &str = "Airports.Path";
pub const NAVAIDS_PATH: &str = "Navaids.Path";
pub const FIXES_PATH: &str = "Fixes.Path";
// Shape files for shoreline data
pub const GSHHG_PATH: &str = "GSHHG.Path";
pub const AIRCRAFT_TYPE: &str = "Aircraft.type";
pub const MAX_DEVIATION: &str = "Autoplanner.maxDeviation";
pub const MAX_LEG_LENGTH: &str = "Autoplanner.maxLegLength";
pub const MIN_LEG_LENGTH: &str = "Autoplanner.minLegLength";
pub const PLAN_TYPE: &str = "Autoplanner.planType";
pub const USE_RADIO_BEACONS: &str = "Autoplanner.useRadioBeacons";
pub const USE_FIXES: &str = "Autoplanner.useFixes";
pub const USE_GPS: &str = "Autoplanner.useGps";
pub const VOR_ONLY: &str = "Autoplanner.vor_only";
pub const VOR_PREFERED: &str = "Autoplanner.vor_prefered";
pub const ADD_WAYPOINTS: &str = "Autoplanner.add_waypoints";
pub const ADD_WAYPOINT_BIAS: &str = "Autoplanner.add_waypoint_bias";
pub const MAP_VIEW_SHOW_AIRPORT: &str = "Mapview.show.airports";
pub const MAP_VIEW_SHOW_NAVAID: &str = "Mapview.show.navaids";
pub const MAP_VIEW_CENTRE_LAT: &str = "Mapview.centre.lat";
pub const MAP_VIEW_CENTRE_LONG: &str = "Mapview.centre.long";
pub const MAP_VIEW_ZOOM: &str = "Mapview.zoom";
pub const AUTO_PLAN: &str = "Autoplanner.auto_plan";
pub const USE_MAGNETIC_HEADINGS: &str = "Plan.useMagneticHeadings";
pub const FGFS_LINK_ENABLED: &str = "FlightGearLink.enabled";
pub const FGFS_LINK_HOST: &str = "FlightGearLink.host";
pub const FGFS_LINK_PORT: &str = "FlightGearLink.port";

lazy_static! {
    static ref MANAGER: PreferenceManager = PreferenceManager {
        preferences: {
            match PreferencesMap::<String>::load(&APP_INFO, PREFS_PATH) {
                Ok(map) => Arc::new(RwLock::new(map)),
                Err(e) => {
                    error!("Error opening preferences {}", e);
                    Arc::new(RwLock::new(PreferencesMap::new()))
                }
            }
        },
        path: PREFS_PATH,
    };
}

pub struct PreferenceManager {
    preferences: Arc<RwLock<PreferencesMap>>,
    path: &'static str,
}

impl PreferenceManager {
    pub fn get<T: FromStr>(&self, key: &str) -> Option<T> {
        match self.preferences.read().unwrap().get(key) {
            Some(s) => match s.parse::<T>() {
                Ok(i) => Some(i),
                Err(_e) => None,
            },
            None => None,
        }
    }
    pub fn put<T: ToString>(&self, key: &str, value: T) -> () {
        {
            let mut prefs = self.preferences.write().unwrap();
            prefs.insert(key.to_string(), value.to_string());
        }
        self.store();
        event::manager().notify_listeners(Event::PreferencesChanged);
        ()
    }

    pub fn remove(&self, key: &str) -> () {
        {
            let mut prefs = self.preferences.write().unwrap();
            let _e = prefs.remove(key);
        }
        self.store();
        ()
    }

    pub fn clear(&self) -> () {
        {
            let mut prefs = self.preferences.write().unwrap();
            prefs.clear();
        }
        self.store();
        ()
    }

    fn store(&self) {
        let prefs = self.preferences.read().unwrap();
        let _ = prefs.save(&APP_INFO, self.path);
        ()
    }
}

pub fn manager() -> &'static PreferenceManager {
    &MANAGER
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use preferences::PreferencesMap;

    use crate::preference;

    #[test]
    fn test_save_restore() {
        let manager = preference::PreferenceManager {
            preferences: Arc::new(RwLock::new(PreferencesMap::new())),
            path: "kelpie-unit-test",
        };

        manager.put("Test_KEY 1", "First");
        manager.put("Test_KEY 2", 1 as i32);
        manager.put("Test_KEY 3", 24.66 as f64);

        assert_eq!(
            manager.get::<String>("Test_KEY 1"),
            Some("First".to_string())
        );
        assert_eq!(manager.get::<i32>("Test_KEY 2"), Some(1));
        assert_eq!(manager.get::<f64>("Test_KEY 3"), Some(24.66));
    }
}

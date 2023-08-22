use std::sync::{Arc, RwLock};

use super::aircraft::Aircraft;
use super::airport::Airport;
use super::location::Location;
use super::sector::Sector;
use super::waypoint::{self, AirportWaypoint, Waypoint};

struct Plan {
    dirty: bool,
    listeners: Arc<RwLock<Vec<Box<dyn IPlanListener>>>>,
    path: Option<String>,
    sectors: Arc<RwLock<Vec<Box<Sector>>>>,
    aircraft: Option<Aircraft>,
    max_altitude: Option<i32>,
}

impl Plan {
    pub fn new() -> Self {
        Self {
            dirty: false,
            listeners: Arc::new(RwLock::new(Vec::new())),
            path: None,
            sectors: Arc::new(RwLock::new(Vec::with_capacity(2))),
            aircraft: None,
            max_altitude: None,
        }
    }

    pub fn add_plan_listener(&mut self, l: Box<dyn IPlanListener>) {
        self.listeners.write().unwrap().push(l);
    }

    pub fn add_sector(&mut self, start: Option<&Airport>, end: Option<&Airport>) {
        let mut sector = Sector::new(start, end);
        self.sectors.write().unwrap().push(Box::new(sector));
        self.update();
    }

    pub fn add_sector_at(&mut self, pos: i32, start: Option<&Airport>, end: Option<&Airport>) {
        let mut sector = Sector::new(start, end);
        self.sectors
            .write()
            .unwrap()
            .insert(pos as usize, Box::new(sector));
        self.update();
    }

    pub fn remove_sector_at(&mut self, pos: i32) {
        self.sectors.write().unwrap().remove(pos as usize);
        self.update();
    }

    pub fn get_aircraft(&self) -> Option<Aircraft> {
        self.aircraft.clone()
    }

    pub fn set_aircraft(&mut self, aircraft: Option<Aircraft>) {
        self.aircraft = aircraft;
        self.update();
    }

    pub fn set_max_altitude(&mut self, max_altitude: Option<i32>) {
        self.max_altitude = max_altitude;
    }

    pub fn get_max_altitude(&self) -> Option<i32> {
        self.max_altitude
    }

    pub fn get_plan_altitude(&self) -> i32 {
        match self.max_altitude {
            Some(a) => a,
            None => match self.get_aircraft() {
                Some(a) => a.get_cruise_altitude(),
                None => 0,
            },
        }
    }

    pub fn get_duration(&self) -> f64 {
        self.sectors
            .read()
            .unwrap()
            .iter()
            .map(|s| s.get_duration())
            .sum()
    }

    //
    //	Return if this plan has been changed since it was loaded from
    //	persistent storage.
    //	@return
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn get_name(&self) -> String {
        if self.path.is_none() {
            let sectors_lock = self.sectors.read().unwrap();
            if sectors_lock.len() > 0 {
                let airport_start = sectors_lock[0].get_start().map(|w| w.get_airport());
                let start = airport_start.map(|a| a.get_id()).unwrap_or(String::new());

                let airport_end = self.sectors.read().unwrap()[0]
                    .get_end()
                    .map(|w| w.get_airport());
                let end = airport_end.map(|a| a.get_id()).unwrap_or(String::new());

                if !start.is_empty() || !end.is_empty() {
                    return format!("{}-{}.fpl", start, end);
                }
                return String::from("new_plan.fpl");
            }
        }

        let f = std::path::PathBuf::from(&self.path.as_ref().unwrap());
        f.file_name().unwrap().to_string_lossy().to_string()
    }

    //
    //	 Get the waypoint that precedes this one in the plan.
    //	 @param loc
    //	 @return previous location

    pub fn get_previous_location(&self, wp: Box<dyn Waypoint>) -> Option<Box<dyn Waypoint>> {
        let sectors_lock = self.sectors.read().unwrap();
        for s in sectors_lock.as_slice() {
            let wp_comp = wp.copy();
            let start_wp = s.get_start();
            match start_wp {
                Some(awp) => {
                    if compare_wp(Box::new(awp), wp_comp) {
                        return None;
                    }
                }
                _ => (),
            }
            let wp_comp = wp.copy();
            let end_wp = s.get_end();
            match end_wp {
                Some(awp) => {
                    if compare_wp(Box::new(awp), wp_comp) {
                        if s.get_waypoint_count() == 0 {
                            let start_wp = s.get_start();
                            match start_wp {
                                Some(awp) => return Some(Box::new(awp.clone())),
                                _ => return None,
                            }
                        } else if s.get_waypoint_count() > 0 {
                            return s.get_waypoint(s.get_waypoint_count() - 1);
                        }
                    }
                }
                _ => (),
            }
            for i in 0..s.get_waypoint_count() {
                let awp = s.get_waypoint(i);
                let wp_comp = wp.copy();
                if compare_wp(awp.unwrap(), wp_comp) {
                    if i == 0 {
                        let start_wp = s.get_start();
                        match start_wp {
                            Some(awp) => return Some(Box::new(awp.clone())),
                            _ => return None,
                        }
                    } else {
                        return s.get_waypoint(i - 1);
                    }
                }
            }
        }
        None
    }

    fn update(&self) {}
}

trait IPlanListener {
    // Definition...
}

fn compare_wp(a: Box<dyn Waypoint>, b: Box<dyn Waypoint>) -> bool {
    waypoint::eq(&a, &b)
}

#[cfg(test)]
mod tests {
    use super::Plan;
    use crate::model::test_utils::make_airport;

    #[test]
    fn test_name() {
        let a = make_airport("YSSY");
        let b = make_airport("YMLB");
        let mut plan = Plan::new();
        plan.add_sector(Some(&a), Some(&b));
        assert_eq!(plan.get_name(), "YSSY-YMLB.fpl");

        let mut plan = Plan::new();
        plan.add_sector(None, Some(&b));
        assert_eq!(plan.get_name(), "-YMLB.fpl");

        let mut plan = Plan::new();
        plan.add_sector(Some(&a), None);
        assert_eq!(plan.get_name(), "YSSY-.fpl");

        let mut plan = Plan::new();
        plan.add_sector(None, None);
        assert_eq!(plan.get_name(), "new_plan.fpl");
    }
}

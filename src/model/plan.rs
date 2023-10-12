use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::sync::Arc;

use crate::earth::coordinate::Coordinate;
use crate::model::waypoint::Waypoint;
use crate::preference::{UNITS, USE_MAGNETIC_HEADINGS};
use crate::util::distance_format::DistanceFormat;
use crate::util::hour_format::HourFormat;
use crate::util::speed_format::SpeedFormat;

use super::aircraft::Aircraft;
use super::airport::Airport;
use super::sector::Sector;

#[derive(Default)]
pub struct Plan {
    dirty: RefCell<bool>,
    path: RefCell<Option<String>>,
    sectors: RefCell<Vec<RefCell<Sector>>>,
    aircraft: RefCell<Option<Aircraft>>,
    max_altitude: RefCell<i32>,
}

impl Plan {
    pub fn new() -> Self {
        Self {
            dirty: RefCell::new(false),
            path: RefCell::new(None),
            sectors: RefCell::new(Vec::with_capacity(2)),
            aircraft: RefCell::new(None),
            max_altitude: RefCell::new(0i32),
        }
    }

    pub fn add_sector(&self, start: Option<Arc<Airport>>, end: Option<Arc<Airport>>) {
        let sector = Sector::new();
        match start {
            Some(s) => sector.set_start(s),
            None => (),
        }
        match end {
            Some(e) => sector.set_end(e),
            None => (),
        }
        self.sectors.borrow_mut().push(RefCell::new(sector));
    }

    pub fn add_sector_at(
        &self,
        pos: usize,
        start: Option<Arc<Airport>>,
        end: Option<Arc<Airport>>,
    ) {
        let sector = Sector::new();
        match start {
            Some(s) => sector.set_start(s),
            None => (),
        }
        match end {
            Some(e) => sector.set_end(e),
            None => (),
        }
        self.sectors
            .borrow_mut()
            .insert(pos as usize, RefCell::new(sector));
    }

    pub fn remove_sector_at(&self, pos: usize) {
        self.sectors.borrow_mut().remove(pos);
    }

    pub fn get_sectors(&self) -> Ref<Vec<RefCell<Sector>>> {
        self.sectors.borrow()
    }

    pub fn get_aircraft(&self) -> Ref<Option<Aircraft>> {
        self.aircraft.borrow()
    }

    pub fn set_aircraft(&self, aircraft: &Option<Aircraft>) {
        self.aircraft.replace(aircraft.clone());
    }

    pub fn set_max_altitude(&self, max_altitude: i32) {
        self.max_altitude.replace(max_altitude.clone());
    }

    pub fn get_max_altitude(&self) -> i32 {
        self.max_altitude.borrow().clone()
    }

    pub fn get_plan_altitude(&self) -> i32 {
        match self.max_altitude.borrow().clone() {
            0 => match self.aircraft.borrow().deref() {
                Some(a) => a.get_cruise_altitude().clone(),
                None => 0,
            },
            max => max,
        }
    }

    pub fn get_duration(&self) -> f64 {
        self.sectors
            .borrow()
            .iter()
            .map(|s| s.borrow().get_duration(self))
            .sum()
    }

    //
    //	Return if this plan has been changed since it was loaded from
    //	persistent storage.
    //	@return
    pub fn is_dirty(&self) -> bool {
        self.dirty.borrow().clone()
    }

    pub fn get_name(&self) -> String {
        if self.path.borrow().is_none() {
            let mut start: String = "".to_string();
            let mut end: String = "".to_string();
            let sectors = self.sectors.borrow();
            if sectors.len() > 0 {
                if let Some(airport_start) = sectors[0].borrow().get_start() {
                    start = airport_start.get_id().to_string();
                }
                if let Some(airport_end) = sectors[0].borrow().get_end() {
                    end = airport_end.get_id().to_string();
                }
                if !start.is_empty() || !end.is_empty() {
                    return format!("{}-{}.fpl", start, end);
                }
                return String::from("new_plan.fpl");
            }
        }

        let f = std::path::PathBuf::from(&self.path.borrow().clone().unwrap_or("".to_string()));
        f.file_name().unwrap().to_string_lossy().to_string()
    }

    //
    //	 Get the waypoint that precedes this one in the plan.
    //	 @param loc
    //	 @return previous location

    pub fn get_previous_location(&self, wp: &Waypoint) -> Option<Coordinate> {
        for s in self.get_sectors().deref() {
            let wp_comp = wp.clone();
            let s_borrowed = s.borrow();
            if let Some(start_wp) = s_borrowed.get_start() {
                if compare_wp(&start_wp, &wp_comp) {
                    return None;
                }
            }
            // let wp_comp = wp.clone();
            if let Some(end_wp) = s_borrowed.get_end() {
                if compare_wp(&end_wp, &wp_comp) {
                    if s_borrowed.get_waypoint_count() == 0 {
                        if let Some(start_wp) = s_borrowed.get_start() {
                            return Some(start_wp.get_loc().clone());
                        }
                    } else {
                        return s_borrowed
                            .get_waypoint(s_borrowed.get_waypoint_count() - 1)
                            .map(|wp| wp.get_loc().clone());
                    }
                }
            }

            for i in 0..s_borrowed.get_waypoint_count() {
                let wp_comp = wp.clone();
                let awp = s_borrowed.get_waypoint(i);
                match awp {
                    None => (),
                    Some(wpx) => {
                        if compare_wp(&wpx, &wp_comp) {
                            if i == 0 {
                                if let Some(start_wp) = s_borrowed.get_start() {
                                    return Some(start_wp.get_loc().clone());
                                }
                            } else {
                                return s_borrowed
                                    .get_waypoint(i - 1)
                                    .map(|wp| wp.get_loc().clone());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn add_airport(&self, airport: Arc<Airport>) {
        for s in self.sectors.borrow_mut().deref() {
            if s.borrow().get_start().is_none() {
                s.borrow_mut().set_start(airport);
                return;
            }
            if s.borrow().get_end().is_none() {
                s.borrow_mut().set_end(airport);
                return;
            }
        }
    }

    /**
     * Get the heading from the previous waypoint to the specified.
     * @param loc
     * @return double Heading
     */
    pub fn get_leg_heading_to(&self, wp: &Waypoint) -> f64 {
        let pref = crate::preference::manager();

        let mut heading = 0.0;

        if let Some(prev) = self.get_previous_location(wp) {
            heading = prev.bearing_to_deg(&wp.get_loc());
            if pref.get::<bool>(USE_MAGNETIC_HEADINGS).unwrap_or(false) {
                todo!();
                // convert to magnetic: This library returns POSITIVE for East variation, so we SUBTRACT it
                // Geomagnetism
                // magnetism = new
                // Geomagnetism(loc.getLong(), loc.getLat(), loc.getElevation());
                // heading -= magnetism.getDeclination();
            }
            if heading < 0.0 {
                heading += 360.0;
            }
        }
        heading
    }

    /**
     * Get the distance from the previous waypoint to the specified one.
     * @param loc
     * @return double Distance
     */
    pub fn get_leg_distance_to(&self, wp: &Waypoint) -> f64 {
        match self.get_previous_location(wp) {
            Some(prev) => prev.distance_to(&wp.get_loc()),
            None => 0.0,
        }
    }

    /**
     * Get the distance from the previous waypoint to the specified one as a string.
     * @param loc
     * @return String Distance
     */
    pub fn get_leg_distance_to_as_string(&self, wp: &Waypoint) -> String {
        let pref = crate::preference::manager();
        let units = pref.get::<String>(UNITS).unwrap_or("Nm".to_string());
        let distance_format = DistanceFormat::new(&units);
        let distance = &self.get_leg_distance_to(wp);
        distance_format.format(distance)
    }
    pub fn get_time_to(&self, waypoint: &Waypoint) -> f64 {
        let speed = self.get_speed_to(waypoint);
        if speed == 0 {
            return 0.0;
        }
        let leg_distance = self.get_leg_distance_to(waypoint);
        leg_distance as f64 / speed as f64
    }

    pub fn get_time_to_as_string(&self, waypoint: &Waypoint) -> String {
        let time_format = HourFormat::new();
        let time = &self.get_time_to(waypoint);
        time_format.format(time)
    }

    pub fn get_speed_to(&self, waypoint: &Waypoint) -> i32 {
        if let Some(aircraft) = &self.aircraft.borrow().deref() {
            let climb = aircraft.get_climb_speed();
            let cruise = aircraft.get_cruise_speed();
            let sink = aircraft.get_sink_speed();

            for s in self.get_sectors().deref() {
                let mut speed = climb.clone();
                let s_borrowed = s.borrow();
                if let Some(start_wp) = s_borrowed.get_start() {
                    if compare_wp(&start_wp, waypoint) {
                        return 0;
                    }
                }
                for wp in s_borrowed
                    .deref()
                    .get_waypoints()
                    .read()
                    .expect("Can't get read lock on sectors")
                    .deref()
                {
                    if compare_wp(wp, waypoint) {
                        return speed;
                    }
                    speed = match wp {
                        Waypoint::Toc { .. } => cruise.clone(),
                        Waypoint::Bod { .. } => sink.clone(),
                        _ => speed,
                    };
                }
                if let Some(end_wp) = s_borrowed.get_end() {
                    if compare_wp(&end_wp, waypoint) {
                        return sink.clone();
                    }
                }
            }
            0
        } else {
            0
        }
    }

    pub fn get_speed_to_as_string(&self, wp: &Waypoint) -> String {
        let pref = crate::preference::manager();
        let units = pref.get::<String>(UNITS).unwrap_or("Nm".to_string());
        let speed_format = SpeedFormat::new(&units);
        let speed = self.get_speed_to(wp) as f64;
        speed_format.format(&speed)
    }
}

fn compare_wp(a: &Waypoint, b: &Waypoint) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    use crate::model::test_utils::make_airport;

    use super::Plan;

    #[test]
    fn test_name() {
        let a = make_airport("YSSY");
        let b = make_airport("YMLB");
        let plan = Plan::new();
        plan.add_sector(Some(a.clone()), Some(b.clone()));
        assert_eq!(plan.get_name(), "YSSY-YMLB.fpl");

        let plan = Plan::new();
        plan.add_sector(None, Some(b.clone()));
        assert_eq!(plan.get_name(), "-YMLB.fpl");

        let plan = Plan::new();
        plan.add_sector(Some(a.clone()), None);
        assert_eq!(plan.get_name(), "YSSY-.fpl");

        let plan = Plan::new();
        plan.add_sector(None, None);
        assert_eq!(plan.get_name(), "new_plan.fpl");
    }
}

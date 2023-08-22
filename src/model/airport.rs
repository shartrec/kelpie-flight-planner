use super::location::Location;
use crate::earth::coordinate::Coordinate;
use crate::earth::FEET_PER_DEGREE;

#[derive(Clone)]
pub struct Airport {
    id: String,
    coordinate: Coordinate,
    elevation: f64,
    control_tower: bool,
    runways: Option<Vec<Runway>>,
    show_default_buildings: bool,
    taxiways: Option<Vec<Taxiway>>,
    max_runway_length: i64,
    airport_type: Option<AirportType>,
    name: String,
}

impl Airport {
    pub fn new(
        id: String,
        latitude: f64,
        longitude: f64,
        elevation: f64,
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
            runways: None,
            show_default_buildings,
            taxiways: None,
            max_runway_length,
            airport_type,
            name,
        }
    }

    pub fn add_runway(&mut self, runway: Runway) {
        self.runways.get_or_insert_with(|| Vec::new()).push(runway);
    }

    pub fn add_taxiway(&mut self, taxiway: Taxiway) {
        self.taxiways
            .get_or_insert_with(|| Vec::new())
            .insert(0, taxiway);
    }

    pub fn get_control_tower(&self) -> bool {
        self.control_tower
    }

    pub fn get_runway(&self, i: usize) -> Option<&Runway> {
        self.runways.as_ref().and_then(|runways| runways.get(i))
    }

    pub fn get_runway_count(&self) -> usize {
        self.runways
            .as_ref()
            .map(|runways| runways.len())
            .unwrap_or(0)
    }

    pub fn get_runways(&mut self) -> &mut Vec<Runway> {
        if self.runways.is_none() {
            self.load_runways_and_taxiways();
        }
        self.runways.as_mut().unwrap()
    }

    pub fn get_longest_runway(&mut self) -> Option<&Runway> {
        if self.runways.is_none() {
            self.load_runways_and_taxiways();
        }
        let longest_runway = self
            .runways
            .as_ref()
            .map(|runways| runways.iter().max_by_key(|runway| runway.get_length()));
        longest_runway.unwrap()
    }

    pub fn get_show_default_buildings(&self) -> bool {
        self.show_default_buildings
    }

    pub fn get_taxiway(&self, i: usize) -> Option<&Taxiway> {
        self.taxiways.as_ref().and_then(|taxiways| taxiways.get(i))
    }

    pub fn get_taxiway_count(&self) -> usize {
        self.taxiways
            .as_ref()
            .map(|taxiways| taxiways.len())
            .unwrap_or(0)
    }

    pub fn get_taxiways(&mut self) -> &mut Vec<Taxiway> {
        if self.taxiways.is_none() {
            self.load_runways_and_taxiways();
        }
        self.taxiways.as_mut().unwrap()
    }

    pub fn get_type(&self) -> Option<AirportType> {
        self.airport_type.clone()
    }

    pub fn load_runways_and_taxiways(&mut self) {
        // ... Load runways and taxiways
    }

    pub fn get_max_runway_length(&self) -> i64 {
        self.max_runway_length
    }

    pub fn set_max_runway_length(&mut self, runway_length: i64) {
        self.max_runway_length = runway_length;
    }

    pub fn calc_airport_extent(&self) -> [f64; 4] {
        // ... Calculate airport extent
        // Return values as an array
        [0.0, 0.0, 0.0, 0.0]
    }

    ///  
    ///	 Calculate the extent of a runway or taxiway
    ///	 @param runway
    ///	 return
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

    fn calc_runway_extent(&self, r: &Runway) -> [f64; 4] {
        self.calc_extent(
            r.get_lat(),
            r.get_long(),
            r.get_heading(),
            r.get_length(),
            r.get_width(),
        )
    }
}

impl Location for Airport {
    fn get_elevation(&self) -> f64 {
        self.elevation
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_lat(&self) -> f64 {
        self.coordinate.get_latitude()
    }

    fn get_lat_as_string(&self) -> String {
        self.coordinate.get_latitude_as_string().clone()
    }

    fn get_long(&self) -> f64 {
        self.coordinate.get_longitude()
    }

    fn get_long_as_string(&self) -> String {
        self.coordinate.get_longitude_as_string().clone()
    }

    fn get_loc(&self) -> Coordinate {
        self.coordinate.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Clone)]
pub struct Runway {
    // ... Runway fields
}

impl Runway {
    pub fn get_length(&self) -> i32 {
        10
    }

    pub fn get_lat(&self) -> f64 {
        10.0
    }

    pub fn get_long(&self) -> f64 {
        10.0
    }

    pub fn get_heading(&self) -> f64 {
        10.0
    }

    pub fn get_width(&self) -> i32 {
        10
    }
}

#[derive(Clone)]
pub struct Taxiway {
    nodes: Vec<LayoutNode>,
}

impl Taxiway {
    pub fn get_nodes(&self) -> Vec<LayoutNode> {
        self.nodes.clone()
    }
}

#[derive(Clone)]
pub struct LayoutNode {
    // ... LayoutNode fields
}

#[derive(Clone)]
pub enum AirportType {
    AIRPORT,
    SEABASE,
    HELIPORT,
}

impl AirportType {
    pub fn type_for(airport_type: &str) -> Option<AirportType> {
        if airport_type == "1" {
            Some(AirportType::AIRPORT)
        } else if airport_type == "16" {
            Some(AirportType::SEABASE)
        } else if airport_type == "17" {
            Some(AirportType::HELIPORT)
        } else {
            None
        }
    }
}

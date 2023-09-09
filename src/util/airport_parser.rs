use crate::earth::coordinate::Coordinate;
use crate::model::airport::{Airport, AirportType};
use crate::model::runway::Runway;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek};

pub struct AirportParserFG850 {
    airport_map: HashMap<String, Airport>,
    runway_map: HashMap<String, Runway>,
    runway_opposite_map: HashMap<String, Runway>,
    runway_offsets: HashMap<String, usize>,
}

impl AirportParserFG850 {
    pub fn new() -> Self {
        Self {
            airport_map: HashMap::with_capacity(25000),
            runway_map: HashMap::with_capacity(25000),
            runway_opposite_map: HashMap::with_capacity(25000),
            runway_offsets: HashMap::with_capacity(25000),
        }
    }

    pub fn load_airports(
        &mut self,
        airports: &mut Vec<Box<Airport>>,
        reader: &mut BufReader<File>,
    ) -> Result<(), String> {
        // Skip header rows
        let mut offset: usize = 0;

        let mut buf = String::new();
        for _i in 0..3 {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => return Ok(()), // EOF
                Ok(_bytes) => offset = reader.stream_position().unwrap() as usize,
                Err(msg) => {
                    match msg.kind() {
                        std::io::ErrorKind::InvalidData => (),
                        _ => {
                            let err_msg = format!("{}", msg).to_string();
                            return Err(err_msg);
                        }
                    }
                    println!("{}", msg.kind());
                    () // We ignore the error on the first two rows - NOT UTF-8
                }
            }
        }

        loop {
            let is_empty = &buf.trim().is_empty();
            if !is_empty {
                let mut tokenizer = buf.split_whitespace();
                let r_type = tokenizer.next().unwrap_or("");
                // Translate other conditions and logic accordingly
                if r_type == "1" || r_type == "16" || r_type == "17" {
                    let airport_type = AirportType::type_for(r_type);
                    let elevation = tokenizer
                        .next()
                        .unwrap_or("0.0")
                        .parse::<f64>()
                        .unwrap_or(0.0);
                    let tower = tokenizer
                        .next()
                        .unwrap_or("0.0")
                        .parse::<bool>()
                        .unwrap_or(false);
                    let default_buildings = tokenizer
                        .next()
                        .unwrap_or("0.0")
                        .parse::<bool>()
                        .unwrap_or(false);
                    let id = tokenizer.next().unwrap_or("????");
                    // Store the offset so we can load the runways later
                    let mut name = String::new();
                    name.push_str(tokenizer.next().unwrap_or(""));
                    for token in tokenizer.into_iter() {
                        name.push_str(&" ");
                        name.push_str(token);
                    }
                    self.runway_offsets.insert(id.to_string(), offset);

                    // Now read runways to get a latitude and longitude
                    // and find the longest
                    let mut max_length = 0.0;

                    let mut latitude = 0.0;
                    let mut longitude = 0.0;

                    // Now we start reading through the runways and taxiways untile we get to the next airport entry
                    let mut buf2 = String::new();

                    buf2.clear();
                    match reader.read_line(&mut buf2) {
                        Ok(0) => return Ok(()), // EOF
                        Ok(_bytes) => offset = reader.stream_position().unwrap() as usize,
                        Err(msg) => {
                            let err_msg = format!("{}", msg).to_string();
                            return Err(err_msg);
                        }
                    }
                    loop {
                        if buf2.len() > 0 {
                            //                            let s = buf.clone();
                            let mut tokenizer = buf2.split_whitespace();
                            let r_type = tokenizer.next().unwrap_or("");
                            if r_type == "1" || r_type == "16" || r_type == "17" {
                                break;
                            }
                            if r_type == "100" {
                                tokenizer.next(); //width
                                tokenizer.next(); //surface type
                                tokenizer.next(); //shoulder surface
                                tokenizer.next(); //smoothness
                                tokenizer.next(); //centre lights
                                tokenizer.next(); //edge lights
                                tokenizer.next(); //auto gen distremaining signs

                                let r_number = tokenizer.next();
                                let r_lat = tokenizer
                                    .next()
                                    .unwrap_or("0.0")
                                    .parse::<f64>()
                                    .unwrap_or(0.0);
                                let r_long = tokenizer
                                    .next()
                                    .unwrap_or("0.0")
                                    .parse::<f64>()
                                    .unwrap_or(0.0);
                                tokenizer.next(); // Length displaced threshold
                                tokenizer.next(); // Length overrun
                                tokenizer.next(); // markings
                                tokenizer.next(); // approach lights
                                tokenizer.next(); // TDZ flag
                                tokenizer.next(); // REIL flag

                                // Now the other end.  needed to get the length
                                let r1_number = tokenizer.next();
                                let r1_lat = tokenizer
                                    .next()
                                    .unwrap_or("0.0")
                                    .parse::<f64>()
                                    .unwrap_or(0.0);
                                let r1_long = tokenizer
                                    .next()
                                    .unwrap_or("0.0")
                                    .parse::<f64>()
                                    .unwrap_or(0.0);
                                tokenizer.next(); // Length displaced threshold
                                tokenizer.next(); // Length overrun
                                tokenizer.next(); // markings
                                tokenizer.next(); // approach lights
                                tokenizer.next(); // TDZ flag
                                tokenizer.next(); // REIL flag

                                let c1 = Coordinate::new(r_lat, r_long);
                                let c2 = Coordinate::new(r1_lat, r1_long);
                                let r_length = c1.distance_to(&c2) * 6076.0;
                                if r_length > max_length {
                                    max_length = r_length;
                                    latitude = (r_lat + r1_lat) / 2.0;
                                    longitude = (r_long + r1_long) / 2.0;
                                }
                            }
                            buf2.clear();
                            match reader.read_line(&mut buf2) {
                                Ok(0) => return Ok(()), // EOF
                                Ok(_bytes) => offset = reader.stream_position().unwrap() as usize,
                                Err(msg) => {
                                    let err_msg = format!("{}", msg).to_string();
                                    return Err(err_msg);
                                }
                            }
                        }
                    }
                    let airport = Airport::new(
                        id.to_string(),
                        latitude,
                        longitude,
                        elevation,
                        airport_type,
                        tower,
                        default_buildings,
                        name,
                        max_length as i64,
                    );
                    airports.push(Box::new(airport));
                    buf.clear();
                    buf.push_str(buf2.as_str());
                } else {
                    buf.clear();
                    match reader.read_line(&mut buf) {
                        Ok(0) => return (Ok(())), // EOF
                        Ok(_bytes) => (),
                        Err(msg) => {
                            let err_msg = format!("{}", msg).to_string();
                            return Err(err_msg);
                        }
                    }
                }
            } else {
                buf.clear();
                match reader.read_line(&mut buf) {
                    Ok(0) => return Ok(()), // EOF
                    Ok(bytes) => offset = reader.stream_position().unwrap() as usize,
                    Err(msg) => {
                        let err_msg = format!("{}", msg).to_string();
                        return Err(err_msg);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::airport::Airport;
    use crate::model::location::Location;
    use std::{fs, io::BufReader, path::PathBuf};

    use super::AirportParserFG850;

    #[test]
    fn test_parse() {
        let mut airports: Vec<Box<Airport>> = Vec::new();

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/airports.dat");
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
            Err(e) => panic!("Unable to open test airport data"),
        }

        assert_eq!(airports.len(), 22);
        assert_eq!(airports[21].get_id(), "RKSG");
        assert_eq!(airports[21].get_max_runway_length(), 8217);
    }
}

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
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::sync::{Arc, RwLock};

use flate2::read::GzDecoder;
use log::{error, warn};

use crate::earth::coordinate::Coordinate;
use crate::model::airport::{Airport, AirportType, LayoutNode, Runway, RunwayType, Taxiway};
use crate::model::location::Location;

pub struct AirportParserFG850 {}

impl AirportParserFG850 {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_airports(
        &mut self,
        airports: &mut Vec<Arc<Airport>>,
        runway_offsets: &mut HashMap<String, usize>,
        reader: &mut BufReader<GzDecoder<File>>,
    ) -> Result<(), String> {
        // Skip header rows
        let mut offset: usize = 0;

        let mut buf = String::new();
        for _i in 0..3 {
            buf.clear();
            // rather than read a line we need to read the non UTF-8 lines and decode ourselves
            match Self::read_ascii_line(reader, &mut buf) {
                Ok(0) => return Ok(()), // EOF
                Ok(_bytes) => {
                    offset += 1;
                }
                Err(msg) => {
                    error!("{}", msg.to_string());
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
                    let elevation = tokenizer.next().unwrap_or("0").parse::<i32>().unwrap_or(0);
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
                    for token in tokenizer {
                        name.push(' ');
                        name.push_str(token);
                    }
                    runway_offsets.insert(id.to_string(), offset);

                    // Now read runways to get a latitude and longitude
                    // and find the longest
                    let mut max_length = 0.0;

                    let mut latitude = 0.0;
                    let mut longitude = 0.0;

                    // Now we start reading through the runways and taxiways until we get to the next airport entry
                    let mut buf2 = String::new();

                    buf2.clear();
                    match Self::read_ascii_line(reader, &mut buf2) {
                        Ok(0) => return Ok(()), // EOF
                        Ok(_bytes) => {
                            offset += 1;
                        }
                        Err(msg) => {
                            error!("{}", msg.to_string());
                        }
                    }
                    loop {
                        if !buf2.is_empty() {
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

                                let _number = tokenizer.next();
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
                                let _number = tokenizer.next();
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
                            } else if r_type == "102" {
                                // let r_width = crate::util::airport_parser::token_f64(tokens.next()) * 3.28;
                                // let r_surface = tokens.next().unwrap_or("");
                                let _number = tokenizer.next().unwrap_or("");
                                let r_lat = crate::util::airport_parser::token_f64(tokenizer.next());
                                let r_long = crate::util::airport_parser::token_f64(tokenizer.next());
                                let _hdg = token_f64(tokenizer.next()); //Orientation
                                let r_length = token_f64(tokenizer.next()) * 3.28;
                                let _width = token_f64(tokenizer.next()) * 3.28;
                                let _surface = tokenizer.next().unwrap_or(""); // Surface
                                tokenizer.next(); // Markings
                                tokenizer.next(); // Shoulder
                                tokenizer.next(); // Smoothness
                                let _edge_lights = tokenizer.next().unwrap_or(""); //edge lights
                                if r_length > max_length {
                                    max_length = r_length;
                                    latitude = r_lat;
                                    longitude = r_long;
                                }

                            }
                        }
                        buf2.clear();
                        match Self::read_ascii_line(reader, &mut buf2) {
                            Ok(0) => break, // EOF
                            Ok(_bytes) => {
                                offset += 1;
                            }
                            Err(msg) => {
                                error!("{}", msg.to_string());
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
                    airports.push(Arc::new(airport));
                    buf.clear();
                    buf.push_str(buf2.as_str());
                } else {
                    buf.clear();
                    match Self::read_ascii_line(reader, &mut buf) {
                        Ok(0) => return Ok(()), // EOF
                        Ok(_bytes) => (),
                        Err(msg) => {
                            error!("{}", msg.to_string());
                        }
                    }
                }
            } else {
                buf.clear();
                match Self::read_ascii_line(reader, &mut buf) {
                    Ok(0) => return Ok(()), // EOF
                    Ok(_bytes) => {
                        offset += 1;
                    }
                    Err(msg) => {
                        error!("{}", msg.to_string());
                    }
                }
            }
        }
    }

    fn read_ascii_line(reader: &mut BufReader<GzDecoder<File>>, buf: &mut String) -> Result<usize, Error> {
        let mut byte_buf = Vec::<u8>::new();
        match reader.read_until(b'\n', &mut byte_buf) {
            Ok(0) => Ok(0), // EOF
            Ok(bytes) => {
                match std::str::from_utf8(&byte_buf) {
                    Ok(ccc) => {
                        buf.push_str(ccc);
                    }
                    Err(e) => {
                        warn!("{}", e);                    }
                }
                Ok(bytes)
            }
            Err(e) => Err(e)
        }
    }

    pub fn load_runways(
        &self,
        airport: &Airport,
        runway_offsets: &RwLock<HashMap<String, usize>>,
        reader: &mut BufReader<GzDecoder<File>>,
    ) -> Result<(), String> {
        let mut tokenizer: std::str::SplitWhitespace;
        let mut buf = String::new();

        let offsets = runway_offsets
            .read()
            .expect("Couldn't get lock on runway offsets");
        let offset = offsets.get(airport.get_id());

        if let Some(o) = offset {
            // We want to quickly read upto the airport we want
            let mut byte_buf = Vec::<u8>::new();
            for _ in 0..o - 2 {
                buf.clear();
                match reader.read_until(b'\n', &mut byte_buf) {
                    Ok(0) => return Ok(()), // EOF
                    Ok(_bytes) => (),
                    Err(_) => {
                        warn!("Seeking for airport runways failed");
                        return Ok(())
                    }
                }
            }
        }


        loop {
            buf.clear();
            match Self::read_ascii_line(reader, &mut buf)  {
                Ok(0) => return Ok(()), // EOF
                Ok(_) => (),
                Err(msg) => {
                    let err_msg = format!("{}", msg).to_string();
                    return Err(err_msg);
                }
            }

            if !buf.is_empty() {
                tokenizer = buf.split_whitespace();
                if let Some(r_type) = tokenizer.next() {
                    if r_type == "1" || r_type == "16" || r_type == "17" {
                        tokenizer.next();
                        tokenizer.next();
                        tokenizer.next();
                        let id = tokenizer.next().unwrap_or("");
                        if airport.get_id() == id {
                            self.load_runways_for_airport(airport, reader)?;
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    fn load_runways_for_airport(
        &self,
        airport: &Airport,
        reader: &mut BufReader<GzDecoder<File>>,
    ) -> Result<(), String> {
        let mut tokenizer;
        let mut match_found = true;

        let mut buf = String::new();
        match Self::read_ascii_line(reader, &mut buf)  {
            Ok(0) => return Ok(()), // EOF
            Ok(_bytes) => (),
            Err(msg) => {
                let err_msg = format!("{}", msg).to_string();
                return Err(err_msg);
            }
        }

        // Load the runways
        while match_found {
            if buf.is_empty() {
                break;
            }
            tokenizer = buf.clone();
            let mut tokens = tokenizer.split_whitespace();

            let r_type = tokens.next().unwrap_or("");
            if r_type == "100" {
                let r_width = token_f64(tokens.next()) * 3.28;
                let r_surface = tokens.next().unwrap_or("");
                tokens.next(); // Shoulder surface
                tokens.next(); // Smoothness
                tokens.next(); // Centre lights
                let r_edge_lights = tokens.next().unwrap_or(""); //edge lights
                tokens.next(); //auto gen dist remaining signs
                let r_number = tokens.next().unwrap_or("");
                let r_lat = token_f64(tokens.next());
                let r_long = token_f64(tokens.next());
                tokens.next(); // Length displaced threshold
                tokens.next(); // Length overrun
                let _markings = tokens.next().unwrap_or(""); //edge lights
                tokens.next(); // Approach lights
                tokens.next(); // TDZ flag
                tokens.next(); // REIL flag

                // Now the other end. needed to get the length
                let _number = tokens.next().unwrap_or("");
                let r1_lat = token_f64(tokens.next());
                let r1_long = token_f64(tokens.next());

                let c1 = Coordinate::new(r_lat, r_long);
                let c2 = Coordinate::new(r1_lat, r1_long);

                let r_length = (c1.distance_to(&c2) * 6076.0) as i32;
                let r_hdg = c1.bearing_to(&c2).to_degrees();

                let lat = (r_lat + r1_lat) / 2.0;
                let long = (r_long + r1_long) / 2.0;

                let runway = Runway::new(
                    r_number.to_string(),
                    Some(RunwayType::Runway),
                    lat,
                    long,
                    r_length,
                    r_width as i32,
                    r_hdg,
                    false,
                    r_surface.to_string(),
                    r_edge_lights.to_string(),
                );

                airport.add_runway(runway);
                buf.clear();
                match Self::read_ascii_line(reader, &mut buf) {
                    Ok(_bytes) => (),
                    Err(msg) => {
                        let err_msg = format!("{}", msg).to_string();
                        return Err(err_msg);
                    }
                }
            } else if r_type == "102" {
                    // let r_width = crate::util::airport_parser::token_f64(tokens.next()) * 3.28;
                    // let r_surface = tokens.next().unwrap_or("");
                    let r_number = tokens.next().unwrap_or("");
                    let r_lat = crate::util::airport_parser::token_f64(tokens.next());
                    let r_long = crate::util::airport_parser::token_f64(tokens.next());
                    let r_hdg = token_f64(tokens.next()); //Orientation
                    let r_length = token_f64(tokens.next()) * 3.28;
                    let r_width = token_f64(tokens.next()) * 3.28;
                    let r_surface = tokens.next().unwrap_or(""); // Surface
                    tokens.next(); // Markings
                    tokens.next(); // Shoulder
                    tokens.next(); // Smoothness
                    let r_edge_lights = tokens.next().unwrap_or(""); //edge lights


                    let runway = Runway::new(
                        r_number.to_string(),
                        Some(RunwayType::Helipad),
                        r_lat,
                        r_long,
                        r_length as i32,
                        r_width as i32,
                        r_hdg,
                        false,
                        r_surface.to_string(),
                        r_edge_lights.to_string(),
                    );

                    airport.add_runway(runway);
                    buf.clear();
                    match Self::read_ascii_line(reader, &mut buf)  {
                        Ok(_bytes) => (),
                        Err(msg) => {
                            let err_msg = format!("{}", msg).to_string();
                            return Err(err_msg);
                        }
                    }
            } else if r_type == "110" {
                // Taxiway processing
                // We don't care about anything but the nodes that we use to draw it.
                let mut nodes = Vec::new();

                loop {
                    buf.clear();
                    match Self::read_ascii_line(reader, &mut buf)  {
                        Ok(_bytes) => (),
                        Err(msg) => {
                            let err_msg = format!("{}", msg).to_string();
                            return Err(err_msg);
                        }
                    }

                    if buf.is_empty() {
                        break;
                    }

                    let mut tokenizer = buf.split_whitespace();

                    let n_type = tokenizer.next().unwrap_or("");
                    if n_type == "111" || n_type == "113" || n_type == "115" {
                        let r1_lat = token_f64(tokenizer.next());
                        let r1_long = token_f64(tokenizer.next());
                        nodes.push(LayoutNode::new(
                            n_type.to_string(),
                            r1_lat,
                            r1_long,
                            0.0,
                            0.0,
                        ));
                    } else if n_type == "112" || n_type == "114" || n_type == "116" {
                        let r1_lat = token_f64(tokenizer.next());
                        let r1_long = token_f64(tokenizer.next());
                        let b1_lat = token_f64(tokenizer.next());
                        let b1_long = token_f64(tokenizer.next());
                        nodes.push(LayoutNode::new(
                            n_type.to_string(),
                            r1_lat,
                            r1_long,
                            b1_lat,
                            b1_long,
                        ));
                    } else {
                        break;
                    }
                }

                let taxiway = Taxiway::new(nodes);

                airport.add_taxiway(taxiway);
            } else if r_type == "1" || r_type == "16" || r_type == "17" {
                match_found = false;
            } else {
                buf.clear();
                match Self::read_ascii_line(reader, &mut buf)  {
                    Ok(_bytes) => (),
                    Err(msg) => {
                        let err_msg = format!("{}", msg).to_string();
                        return Err(err_msg);
                    }
                }
            }
        }

        // Call the loadIls function (you need to implement this function)
        // todo
        // load_ils(rdr_runway_ils, airport)?;

        Ok(())
    }
}

fn token_f64(t: Option<&str>) -> f64 {
    t.unwrap_or("0.0").parse::<f64>().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use std::{fs, io::BufReader, path::PathBuf};
    use std::collections::HashMap;
    use std::sync::Arc;

    use flate2::read;

    use crate::model::airport::Airport;
    use crate::model::location::Location;

    use super::AirportParserFG850;

    #[test]
    fn test_parse() {
        let mut airports: Vec<Arc<Airport>> = Vec::new();
        let mut runway_offsets: HashMap<String, usize> = HashMap::new();

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/airports.dat.gz");
        let file = fs::File::open(path);

        match file {
            Ok(input) => {
                let mut parser = AirportParserFG850::new();
                let decoder = read::GzDecoder::new(input);
                let mut reader = BufReader::new(decoder);
                match parser.load_airports(&mut airports, &mut runway_offsets, &mut reader) {
                    Ok(()) => (),
                    Err(msg) => panic! {"{}", msg},
                }
            }
            Err(_e) => panic!("Unable to open test airport data"),
        }

        assert_eq!(airports.len(), 23);
        assert_eq!(airports[21].get_id(), "RKSG");
        assert_eq!(airports[21].get_max_runway_length(), 8217);
    }
}

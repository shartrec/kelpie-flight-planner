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
use std::io::{BufRead, BufReader, Error};
use std::sync::Arc;

use flate2::read::GzDecoder;
use log::info;

use crate::model::fix::Fix;

pub struct FixParserFG {}

impl FixParserFG {
    pub fn load_fixes(
        &mut self,
        fixes: &mut Vec<Arc<Fix>>,
        reader: &mut BufReader<GzDecoder<File>>,
    ) -> Result<(), Error> {
        let mut buf = String::new();

        // ignore first three lines
        for _i in 0..3 {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => return Ok(()), // EOF
                Ok(_bytes) => (),       // EOF
                Err(msg) => {
                    match msg.kind() {
                        std::io::ErrorKind::InvalidData => (),
                        _ => {
                            return Err(msg);
                        }
                    }
                    info!("{}", msg.kind());
                }
            }
        }
        loop {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => return Ok(()), // EOF
                Ok(_bytes) => (),
                Err(msg) => {
                    return Err(msg);
                }
            }
            let is_empty = buf.trim().is_empty();

            if !is_empty && !buf.starts_with("//") && !buf.starts_with("99") {
                let mut tokenizer = buf.split_whitespace();
                let latitude = tokenizer
                    .next()
                    .unwrap_or("0.0")
                    .parse::<f64>()
                    .unwrap_or(0.0);
                let longitude = tokenizer
                    .next()
                    .unwrap_or("0.0")
                    .parse::<f64>()
                    .unwrap_or(0.0);
                let id = tokenizer.next().unwrap_or("");

                let fix = Fix::new(id.to_string(), latitude, longitude);
                fixes.push(Arc::new(fix));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::BufReader, path::PathBuf};
    use std::sync::Arc;

    use flate2::read;

    use crate::model::fix::Fix;
    use crate::model::location::Location;

    use super::FixParserFG;

    #[test]
    fn test_parse() {
        let mut fixs: Vec<Arc<Fix>> = Vec::new();

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/fixes.dat.gz");
        let file = fs::File::open(path);

        match file {
            Ok(input) => {
                let mut parser = FixParserFG {};
                let decoder = read::GzDecoder::new(input);
                let mut reader = BufReader::new(decoder);
                match parser.load_fixes(&mut fixs, &mut reader) {
                    Ok(()) => (),
                    Err(msg) => panic! {"{}", msg},
                }
            }
            Err(e) => panic!("Unable to open test fix data {}", e),
        }

        assert_eq!(fixs.len(), 97);
        assert_eq!(fixs[0].get_id(), "0000E");
        assert_eq!(fixs[20].get_id(), "03MCT");
    }
}

use crate::earth::coordinate::Coordinate;
use crate::model::navaid::{Navaid, NavaidType};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct NavaidParserFG {}

impl NavaidParserFG {
    pub fn load_navaids(
        &mut self,
        navaids: &mut Vec<Box<Navaid>>,
        reader: &mut BufReader<File>,
    ) -> Result<(), String> {
        let mut buf = String::new();

        // ignore first two lins
        for _i in 0..3 {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => return Ok(()), // EOF
                Ok(_bytes) => (),       // EOF
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
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => return Ok(()), // EOF
                Ok(_bytes) => (),
                Err(msg) => {
                    let err_msg = format!("{}", msg).to_string();
                    return Err(err_msg);
                }
            }
            let is_empty = &buf.trim().is_empty();
            if !is_empty {
                let mut tokenizer = buf.split_whitespace();
                let r_type = tokenizer.next().unwrap_or("");
                // Translate other conditions and logic accordingly
                if r_type == "2" || r_type == "3" {
                    let navaid_type = NavaidType::type_for(r_type);

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

                    let elevation = tokenizer
                        .next()
                        .unwrap_or("0")
                        .parse::<i32>()
                        .unwrap_or(0);

                    let mut frequency = tokenizer
                        .next()
                        .unwrap_or("0.0")
                        .parse::<f64>()
                        .unwrap_or(0.0);

                    if navaid_type.as_ref().unwrap_or(&NavaidType::NDB) == &NavaidType::VOR{
                        frequency /= 100.;
                    }

                    let range = tokenizer
                        .next()
                        .unwrap_or("0")
                        .parse::<i32>()
                        .unwrap_or(0);

                    let mag_var = tokenizer.next().unwrap_or("");
                    let id = tokenizer.next().unwrap_or("");

                    let mut name = String::new();
                    name.push_str(tokenizer.next().unwrap_or(""));
                    for token in tokenizer.into_iter() {
                        name.push_str(&" ");
                        name.push_str(token);
                    }

                    let navaid = Navaid::new(
                        id.to_string(),
                        navaid_type.unwrap_or(NavaidType::NDB),
                        latitude,
                        longitude,
                        elevation,
                        frequency,
                        range,
                        mag_var.to_string(),
                        name.to_string(),
                    );
                    navaids.push(Box::new(navaid));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::location::Location;
    use crate::model::navaid::{Navaid, NavaidType};
    use std::{fs, io::BufReader, path::PathBuf};

    use super::NavaidParserFG;

    #[test]
    fn test_parse() {
        let mut navaids: Vec<Box<Navaid>> = Vec::new();

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/navaids.dat");
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
            Err(e) => panic!("Unable to open test navaid data {}", e.to_string()),
        }

        assert_eq!(navaids.len(), 97);
        assert_eq!(navaids[0].get_id(), "APH");
        assert_eq!(navaids[21].get_id(), "AB");
        match navaids[21].get_type() {
            NavaidType::NDB => (),
            _ => panic!("navaid type is not NDB"),
        }
    }
}

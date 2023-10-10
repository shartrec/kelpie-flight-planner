use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

use log::info;

use crate::model::navaid::{Navaid, NavaidType};

pub struct NavaidParserFG {}

impl NavaidParserFG {
    pub fn load_navaids(
        &mut self,
        navaids: &mut Vec<Arc<Navaid>>,
        ils: &mut HashMap<String, Vec<(String, f64)>>,
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
                    info!("{}", msg.kind());
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

                    let latitude = token_number::<f64>(tokenizer.next());
                    let longitude = token_number::<f64>(tokenizer.next());

                    let elevation = token_number::<i32>(tokenizer.next());

                    let mut frequency = token_number::<f64>(tokenizer.next());

                    if navaid_type.as_ref().unwrap_or(&NavaidType::NDB) == &NavaidType::VOR{
                        frequency /= 100.;
                    }

                    let range = token_number::<i32>(tokenizer.next());

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
                    navaids.push(Arc::new(navaid));
                } else if r_type == "4" || r_type == "5" {
                    /* latitude */ tokenizer.next();
                    /* longitude */ tokenizer.next();
                    /* elevation */ tokenizer.next();
                    let frequency =  token_number::<f64>(tokenizer.next()) / 100.0;
                    /* magVar */ tokenizer.next();
                    tokenizer.next();
                    tokenizer.next();
                    if let Some(airport_id) = tokenizer.next() {
                        if let Some(runway_id) = tokenizer.next() {
                            match ils.get_mut(airport_id) {
                                Some(list) => {
                                    list.push((runway_id.to_string(), frequency));
                                }
                                None => {
                                    let mut list = Vec::new();
                                    list.push((runway_id.to_string(), frequency));
                                    ils.insert(airport_id.to_string(), list);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn token_number<T: std::str::FromStr + std::default::Default>(t: Option<&str>) -> T {
    t.unwrap_or("0.0").parse::<T>().unwrap_or( Default::default())
}

#[cfg(test)]
mod tests {
    use std::{fs, io::BufReader, path::PathBuf};
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::model::location::Location;
    use crate::model::navaid::{Navaid, NavaidType};

    use super::NavaidParserFG;

    #[test]
    fn test_parse() {
        let mut navaids: Vec<Arc<Navaid>> = Vec::new();
        let mut ils: HashMap<String, Vec<(String, f64)>> = HashMap::new();

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/navaids.dat");
        let file = fs::File::open(path);

        match file {
            Ok(f) => {
                let mut parser = NavaidParserFG {};
                let mut reader = BufReader::new(f);
                match parser.load_navaids(&mut navaids, &mut ils, &mut reader) {
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

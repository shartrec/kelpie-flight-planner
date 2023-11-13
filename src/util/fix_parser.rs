use std::fs::File;
use std::io::{BufRead, BufReader};
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
    ) -> Result<(), String> {
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
            Err(e) => panic!("Unable to open test fix data {}", e.to_string()),
        }

        assert_eq!(fixs.len(), 97);
        assert_eq!(fixs[0].get_id(), "0000E");
        assert_eq!(fixs[20].get_id(), "03MCT");
    }
}

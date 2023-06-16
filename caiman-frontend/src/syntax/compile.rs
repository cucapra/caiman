use std::path::Path;

use super::run_parser;
use crate::error::Error;

pub fn run(filename: String, _parse: bool)
{
    let filename_path = Path::new(&filename);
    match filename_path.extension().map(|o| o.to_str()) {
        Some(Some("caiman")) => (),
        _ => {
            println!("Invalid file extension for file {}, expected 'caiman'", filename);
            return;
        },
    }
    match run_parser::parse_file(&filename) {
        Ok(ast) => println!("{:#?}", ast),
        Err(local_e) => {
            let e = Error { kind: local_e.kind, location: local_e.location, filename };
            println!("Error: {}", e);
        },
    }
}

use std::path::Path;

use super::{ast, run_parser};
use crate::error::Error;

pub fn parse(filename: String) -> Result<ast::Program, String>
{
    let filename_path = Path::new(&filename);
    match filename_path.extension().map(|o| o.to_str()) {
        Some(Some("caiman")) => (),
        _ => {
            return Err(format!(
                "Invalid file extension for file {}, expected 'caiman'",
                filename
            ));
        },
    }
    run_parser::parse_file(&filename).map_err(|local_e| {
        let e = Error { kind: local_e.kind, location: local_e.location, filename };
        format!("Error: {}", e)
    })
}

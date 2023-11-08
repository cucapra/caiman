pub mod ast;
pub mod ast_factory;
pub mod parser;
pub mod run_parser;

use crate::error::Error;

pub fn parse(filename: String) -> Result<ast::Program, String> {
    run_parser::parse_file(&filename).map_err(|local_e| {
        let e = Error {
            kind: local_e.kind,
            location: local_e.location,
            filename,
        };
        format!("Error: {}", e)
    })
}

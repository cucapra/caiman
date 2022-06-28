pub mod parser;
pub mod ast;

use std::path::Path;
use std::fs::File;

pub fn parse_read<R: std::io::Read>(mut input: R) -> ast::Program
{
    let mut buf = String::new();
    input.read_to_string(&mut buf).unwrap();
    let parser = parser::ProgramParser::new();
    parser.parse(&buf).unwrap()
}

pub fn parse_file(filename: String) -> ast::Program
{
    let input_path = Path::new(&filename);
    let input_file = match File::open(&input_path) {
        Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
        Ok(file) => file,
    };
    parse_read(input_file)
}

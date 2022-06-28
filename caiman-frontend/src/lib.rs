pub mod parser;
pub mod ast;

use std::path::Path;
use std::fs::File;

use lalrpop_util::ParseError;

fn parse_error_to_string<'a>(
    err : ParseError<usize, parser::Token<'a>, &'a str>
) -> String
{
    match err
    {
        ParseError::InvalidToken{location} => {
            format!("Invalid token at {}", location)
        },
        ParseError::UnrecognizedToken{token, expected} => {
            format!(
                "Unrecognized token {:?} at {}, expected one of {:#?}", 
                token.1, 
                token.0,
                expected,
            )
        },
        ParseError::ExtraToken{token} => {
            format!("Extra token {:?} at {}", token.1, token.0)
        },
        ParseError::User{error} => {
            format!("User error: {}", String::from(error))
        },
        ParseError::UnrecognizedEOF{location, expected} => {
            format!(
                "Unrecognized EOF at {}, expected one of {:#?}", 
                location, 
                expected,
            )
        },
    }
}

pub fn parse_string(buf: String) -> Result<ast::Program, String>
{
    let parser = parser::ProgramParser::new();
    match parser.parse(&buf)
    {
        Ok(program) => Ok(program),
        Err(why) => Err(parse_error_to_string(why)),
    }
}

pub fn parse_read<R: std::io::Read>(
    mut input: R
) -> Result<ast::Program, String>
{
    let mut buf = String::new();
    input.read_to_string(&mut buf).unwrap();
    parse_string(buf)
}

pub fn parse_file(filename: String) -> Result<ast::Program, String>
{
    let input_path = Path::new(&filename);
    let input_file = match File::open(&input_path) {
        Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
        Ok(file) => file,
    };
    parse_read(input_file)
}

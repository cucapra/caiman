use crate::value_language::parser;
use crate::value_language::ast;
use crate::value_language::ast_factory::ASTFactory;
use crate::error::LocalError;
use crate::error::ErrorKind;
use crate::error::ErrorLocation;
use std::path::Path;
use std::fs::File;

use lalrpop_util::ParseError;

pub enum ParsingError
{
    InvalidToken,
    UnrecognizedToken(String, String),
    ExtraToken(String),
    EOF(String),
    User(String),
}

fn make_error((l, c): (usize, usize), p: ParsingError) -> LocalError
{
    LocalError
    {
        kind: ErrorKind::Parsing(p),
        location: ErrorLocation::Single(l, c),
    }
}

fn parse_error_to_error<'a>(
    err : ParseError<usize, parser::Token<'a>, &'a str>,
    ast_factory: &ASTFactory,
) -> LocalError
{
    match err
    {
        ParseError::InvalidToken{location} => {
            make_error(
                ast_factory.line_and_column(location), 
                ParsingError::InvalidToken,
            )
        },
        ParseError::UnrecognizedToken{token, expected} => {
            make_error(
                ast_factory.line_and_column(token.0), 
                ParsingError::UnrecognizedToken(
                    format!("{:?}", token.1),
                    format!("{:#?}", expected),
                ),
            )
        },
        ParseError::ExtraToken{token} =>
            make_error(
                ast_factory.line_and_column(token.0), 
                ParsingError::ExtraToken(format!("{:?}", token.1)),
            ),
        ParseError::User{error} => 
            make_error(
                (0, 0), 
                ParsingError::User(error.to_string())
            ),
        ParseError::UnrecognizedEOF{location, expected} => {
            make_error(
                ast_factory.line_and_column(location), 
                ParsingError::EOF(format!("{:#?}", expected)),
            )
        },
    }
}

pub fn parse_string(
    buf: String,
    filename: &str,
) -> Result<ast::ParsedProgram, LocalError>
{
    let parser = parser::ProgramParser::new();
    let ast_factory = ASTFactory::new(filename, &buf);
    parser.parse(&ast_factory, &buf).map_err(|e|
       parse_error_to_error(e, &ast_factory)
    )
}

pub fn parse_read<R: std::io::Read>(
    mut input: R,
    filename: &str,
) -> Result<ast::ParsedProgram, LocalError>
{
    let mut buf = String::new();
    input.read_to_string(&mut buf).unwrap();
    parse_string(buf, filename)
}

pub fn parse_file(filename: &str) -> Result<ast::ParsedProgram, LocalError>
{
    let input_path = Path::new(filename);
    let input_file = match File::open(&input_path) {
        Err(why) => panic!("Couldn't open {}: {}", input_path.display(), why),
        Ok(file) => file,
    };
    parse_read(input_file, filename)
}


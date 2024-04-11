pub mod ast;
pub mod ast_factory;
pub mod parser;

use std::{fs::File, path::Path};

use crate::error::{CustomParsingError, Error, ErrorKind, ErrorLocation, LocalError};
use lalrpop_util::ParseError;

use self::ast_factory::ASTFactory;

/// Construct a `LocalError` from a `ParsingError`
fn make_error((l, c): (usize, usize), p: &str) -> LocalError {
    LocalError {
        kind: ErrorKind::SyntaxParsing(p.to_string()),
        location: ErrorLocation::Single(l, c),
    }
}

/// Converts a parsing error from LALRPOP to a `LocalError`
fn parse_error_to_error(
    err: ParseError<usize, parser::Token<'_>, CustomParsingError>,
    ast_factory: &ASTFactory,
) -> LocalError {
    match err {
        ParseError::InvalidToken { location } => {
            make_error(ast_factory.line_and_column(location), "Invalid token")
        }
        ParseError::UnrecognizedToken { token, expected } => make_error(
            ast_factory.line_and_column(token.0),
            &format!(
                "Unrecognized token: '{}', expected one of {:#?}",
                token.1, expected
            ),
        ),
        ParseError::ExtraToken { token } => make_error(
            ast_factory.line_and_column(token.0),
            &format!("Extra token: '{:?}'", token.1),
        ),
        ParseError::User { error } => make_error(error.loc.start_ln_and_col, &error.msg),
        ParseError::UnrecognizedEOF { location, expected } => make_error(
            ast_factory.line_and_column(location),
            &format!("Unexpected EOF, expected one of: {expected:#?}"),
        ),
    }
}

fn parse_string(buf: &str, filename: &str) -> Result<ast::Program, LocalError> {
    let parser = parser::ProgramParser::new();
    let ast_factory = ASTFactory::new(filename, buf);
    parser
        .parse(&ast_factory, buf)
        .map_err(|e| parse_error_to_error(e, &ast_factory))
}

/// Parses a string into an AST
/// # Errors
/// Returns an error if the string cannot be parsed
/// # Panics
/// Panics if the string cannot be read
#[allow(clippy::module_name_repetitions)]
pub fn parse_read<R: std::io::Read>(
    mut input: R,
    filename: &str,
) -> Result<ast::Program, LocalError> {
    let mut buf = String::new();
    input.read_to_string(&mut buf).unwrap();
    parse_string(&buf, filename)
}

/// Parses a file into an AST
///
/// # Errors
/// Returns an error if the file cannot be opened or if the file cannot be parsed
#[allow(clippy::module_name_repetitions)]
pub fn parse_file(filename: &str) -> Result<ast::Program, Error> {
    let input_path = Path::new(filename);
    let input_file = File::open(input_path).map_err(|e| Error {
        error: LocalError {
            kind: ErrorKind::IO(e.to_string()),
            location: ErrorLocation::Single(0, 0),
        },
        filename: filename.to_string(),
    })?;
    parse_read(input_file, filename).map_err(|e| Error {
        error: e,
        filename: filename.to_string(),
    })
}

use std::fmt;
use crate::value_language::run_parser::ParsingError;
use crate::value_language::check::SemanticError;

pub enum ErrorKind
{
    Parsing(ParsingError),
    Semantic(SemanticError),
}

pub struct Error
{
    pub kind: ErrorKind,
    // (Line, Column)
    pub location: (usize, usize),
    pub filename: String,
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let (l, c) = self.location;
        write!(f, "at {}:{}:{}, ", self.filename, l, c)?;
        match &self.kind
        {
            ErrorKind::Parsing(e) => write!(f, "Parsing Error: {}", e),
            ErrorKind::Semantic(e) => panic!("TODO"),
        }
    }
}

impl fmt::Display for ParsingError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            ParsingError::InvalidToken => write!(f, "Invalid token"),
            ParsingError::UnrecognizedToken(tok, expected) =>
                write!(
                    f, 
                    "Unrecognized token {}, expected one of {}", 
                    tok, 
                    expected,
                ),
            ParsingError::ExtraToken(tok) => write!(f, "Extra token {}", tok),
            ParsingError::EOF(expected) => 
                write!(f, "Unexpected EOF, expected one of {}", expected),
            ParsingError::User(e) => write!(f, "User Error: {}", e),
        }
    }
}


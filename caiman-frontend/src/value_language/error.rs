use std::fmt;
use crate::value_language::check::SemanticError;

pub enum Error
{
    Parsing(String),
    Semantic(SemanticError),
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            Error::Parsing(s) => write!(f, "Parsing Error: {}", s),
            Error::Semantic(e) => panic!("TODO"),
        }
    }
}



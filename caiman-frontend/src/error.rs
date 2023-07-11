//use crate::to_ir::ToIRError;
//use crate::value_language::check::SemanticError;
//use crate::value_language::run_parser::ParsingError;
use crate::syntax::run_parser::ParsingError as SPE;
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct Info
{
    // (Line, Column) and (Beginning, Ending)
    pub location: ((usize, usize), (usize, usize)),
}

impl Default for Info
{
    fn default() -> Self { Self { location: ((0, 0), (0, 0)) } }
}

pub trait HasInfo
{
    fn info(&self) -> Info;
}

pub enum ErrorKind
{
    //Parsing(ParsingError),
    SyntaxParsing(SPE),
    //Semantic(SemanticError),
    //ToIR(ToIRError),
}

pub enum ErrorLocation
{
    // (Line, Column)
    Single(usize, usize),
    Double(((usize, usize), (usize, usize))),
}

// Local as in it happens in one file
pub struct LocalError
{
    pub kind: ErrorKind,
    pub location: ErrorLocation,
}

pub enum FileKind
{
    Value,
    Scheduling,
}

// Local to one value-scheduling compilation unit
pub struct DualLocalError
{
    pub error: LocalError,
    pub file_kind: FileKind,
}

pub struct Error
{
    pub kind: ErrorKind,
    pub location: ErrorLocation,
    pub filename: String,
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self.location {
            ErrorLocation::Single(l, c) => {
                write!(f, "At {}:{}:{}, \n  ", self.filename, l, c)?;
            },
            ErrorLocation::Double(info) => {
                let ((l, c), _) = info;
                write!(f, "At {}:{}:{}, \n  ", self.filename, l, c)?;
            },
        }
        match &self.kind {
            //ErrorKind::Parsing(e) => write!(f, "Parsing Error: {}", e),
            ErrorKind::SyntaxParsing(e) => write!(f, "Parsing Error: {}", e),
            //ErrorKind::Semantic(e) => write!(f, "Semantic Error: {}", e),
            //ErrorKind::ToIR(e) => write!(f, "ToIR Error: {}", e),
        }
    }
}

/*impl fmt::Display for ParsingError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            ParsingError::InvalidToken => write!(f, "Invalid token"),
            ParsingError::UnrecognizedToken(tok, expected) =>
            {
                write!(f, "Unrecognized token {}, expected one of {}", tok, expected,)
            },
            ParsingError::ExtraToken(tok) => write!(f, "Extra token {}", tok),
            ParsingError::EOF(expected) =>
            {
                write!(f, "Unexpected EOF, expected one of {}", expected)
            },
            ParsingError::User(e) => write!(f, "User Error: {}", e),
        }
    }
}*/

// Awful awful awful sorry
impl fmt::Display for SPE
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self {
            SPE::InvalidToken => write!(f, "Invalid token"),
            SPE::UnrecognizedToken(tok, expected) => {
                write!(f, "Unrecognized token {}, expected one of {}", tok, expected,)
            },
            SPE::ExtraToken(tok) => write!(f, "Extra token {}", tok),
            SPE::EOF(expected) => {
                write!(f, "Unexpected EOF, expected one of {}", expected)
            },
            SPE::User(e) => write!(f, "User Error: {}", e),
        }
    }
}

/*
impl fmt::Display for SemanticError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            SemanticError::NameCollision(x) =>
            {
                write!(f, "Declared variable {} already exists!", x)
            },
            SemanticError::TypeMismatch(t, et) =>
            {
                write!(f, "Type mismatch: Expected {:?}, found {:?}", t, et)
            },
            SemanticError::UnboundVariable(x) =>
            {
                write!(f, "Unbound variable {}", x)
            },
            SemanticError::Incompatible(et1, et2) =>
            {
                write!(f, "Incompatible types {:?} and {:?}", et1, et2)
            },
        }
    }
}

impl fmt::Display for ToIRError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            ToIRError::UnknownScheduling(x, subs, full) =>
            {
                write!(
                    f,
                    "Found a scheduled expression {}.{:?}.{:?}, but it's not found in the value \
                     language",
                    x, subs, full
                )
            },
            ToIRError::ForgottenExpr(x, subs, full) =>
            {
                write!(f, "Expression {}.{:?}.{:?} was never scheduled", x, subs, full)
            },
        }
    }
}*/

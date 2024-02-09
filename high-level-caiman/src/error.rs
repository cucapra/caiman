#![allow(clippy::module_name_repetitions)]
use std::fmt;

/// Struct containing information about a token's starting and ending
/// position in a file
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Info {
    pub start_ln_and_col: (usize, usize),
    pub end_ln_and_col: (usize, usize),
}

impl Info {
    /// Constructs a new `Info` struct that represents a range of characters
    /// from `start` to `end`
    #[must_use]
    pub const fn new_range(start: &Self, end: &Self) -> Self {
        Self {
            start_ln_and_col: start.start_ln_and_col,
            end_ln_and_col: end.end_ln_and_col,
        }
    }
}

impl std::fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (l1, c1) = self.start_ln_and_col;
        let (l2, c2) = self.end_ln_and_col;
        write!(f, "{l1}:{c1} - {l2}:{c2}")
    }
}

/// A parsing error that occurs from custom sanitation logic
pub struct CustomParsingError {
    pub loc: Info,
    pub msg: String,
}

/// Constructs a custom parsing error
/// ## Arguments
/// * `$loc` - The location of the error
/// * ...`format_args` - the arguments to the format macro which will format a message to display
#[macro_export]
macro_rules! custom_parse_error {
    ($loc:expr, $($msg:expr),*) => {
        ParseError::User{
            error: CustomParsingError {
                loc: $loc,
                msg: format!($($msg),*),
            }
        }
    };
}

pub trait HasInfo {
    fn info(&self) -> Info;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    SyntaxParsing(String),
    IO(String),
    TypeError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorLocation {
    // (Line, Column)
    Single(usize, usize),
    Double(Info),
}

impl From<Info> for ErrorLocation {
    fn from(info: Info) -> Self {
        Self::Double(info)
    }
}

/// An error that occurs in a single file
#[derive(Debug)]
pub struct LocalError {
    pub kind: ErrorKind,
    pub location: ErrorLocation,
}

/// An error in the frontend
pub struct Error {
    pub error: LocalError,
    pub filename: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.error.location {
            ErrorLocation::Single(l, c) => {
                write!(f, "At {}:{}:{}, \n  ", self.filename, l, c)?;
            }
            ErrorLocation::Double(info) => {
                let (l, c) = info.start_ln_and_col;
                write!(f, "At {}:{}:{}, \n  ", self.filename, l, c)?;
            }
        }
        match &self.error.kind {
            ErrorKind::SyntaxParsing(e) => write!(f, "Parsing Error: {e}"),
            ErrorKind::IO(e) => write!(f, "IO Error: {e}"),
            ErrorKind::TypeError(e) => write!(f, "Type Error: {e}"),
        }
    }
}

impl std::process::Termination for Error {
    fn report(self) -> std::process::ExitCode {
        eprintln!("{self}");
        std::process::ExitCode::FAILURE
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// Constructs a type error
#[must_use]
pub fn type_error(info: Info, msg: &str) -> LocalError {
    LocalError {
        kind: ErrorKind::TypeError(msg.to_string()),
        location: info.into(),
    }
}

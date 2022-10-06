use std::fmt;
use crate::value_language::run_parser::ParsingError;
use crate::value_language::check::SemanticError;

pub enum ErrorKind
{
    Parsing(ParsingError),
    Semantic(SemanticError),
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
        match self.location
        {
            ErrorLocation::Single(l, c) => {
                write!(f, "At {}:{}:{}, \n  ", self.filename, l, c)?;
            },
            ErrorLocation::Double(info) => {
                let ((l, c), _) = info;
                write!(f, "At {}:{}:{}, \n  ", self.filename, l, c)?;
            },
        }
        match &self.kind
        {
            ErrorKind::Parsing(e) => write!(f, "Parsing Error: {}", e),
            ErrorKind::Semantic(e) => write!(f, "Semantic Error: {}", e),
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

impl fmt::Display for SemanticError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            SemanticError::FunctionNameCollision(name) => 
                write!(f, "Name collision for function named {}", name),
            SemanticError::TypeMismatch(t, et) =>
                write!(f, "Type mismatch: Expected {:?}, found {:?}", t, et),
            SemanticError::UnboundVariable(x) =>
                write!(f, "Unbound variable {}", x),
            SemanticError::UnboundFunction(x) =>
                write!(f, "Unbound function {}", x),
            SemanticError::Incompatible(et1, et2) => 
                write!(f, "Incompatible types {:?} and {:?}", et1, et2),
            SemanticError::WrongBinop(et, bop) => write!(
                f, 
                "Cannot use binary operator {:?} with data of type {:?}", 
                bop, 
                et,
            ),
            SemanticError::WrongUnop(et, uop) => write!(
                f, 
                "Cannot use unary operator {:?} with data of type {:?}", 
                uop, 
                et,
            ),
            SemanticError::ReturnTypeMismatch(t, et) =>
                write!(
                    f, 
                    "Return type mismatch: Expected {:?}, found {:?}", 
                    t, 
                    et
                ),
            SemanticError::WrongNumberArgs(exp, act) =>
                write!(f, "Expected {} arguments, got {}", exp, act),
        }
    }
}


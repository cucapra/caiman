use crate::error;
use crate::scheduling_language::schedulable;

pub use crate::error::{Info, DualLocalError};

pub type ToIRResult<T> = Result<T, error::DualLocalError>;

pub enum ToIRError
{
    ForgottenExpr(String, Vec<schedulable::SubExpr>, schedulable::FullExpr),
    UnknownScheduling(String, Vec<schedulable::SubExpr>, schedulable::FullExpr),
}

pub fn make_error(e: ToIRError, i: error::Info) -> error::DualLocalError
{
    let file_kind = match &e
    {
        ToIRError::ForgottenExpr(_,_,_) => error::FileKind::Value,
        ToIRError::UnknownScheduling(_,_,_) => error::FileKind::Scheduling,
    };
    error::DualLocalError {
        error: error::LocalError {
            kind: error::ErrorKind::ToIR(e),
            location: error::ErrorLocation::Double(i.location),
        },
        file_kind,
    }
}


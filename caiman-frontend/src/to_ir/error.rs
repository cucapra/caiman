use crate::error;

pub use crate::error::{Info, DualLocalError};

pub type ToIRResult<T> = Result<T, error::DualLocalError>;

pub enum ToIRError
{
    UnboundScheduleVar(String),
    IncompatibleArgumentNum(usize, usize),
}

pub fn make_error(e: ToIRError, i: error::Info) -> error::DualLocalError
{
    let file_kind = match &e
    {
        ToIRError::UnboundScheduleVar(_) => error::FileKind::Scheduling,
        ToIRError::IncompatibleArgumentNum(_,_) => error::FileKind::Value,
    };
    error::DualLocalError {
        error: error::LocalError {
            kind: error::ErrorKind::ToIR(e),
            location: error::ErrorLocation::Double(i.location),
        },
        file_kind,
    }
}


use std::fmt;

pub enum Error
{
    Parsing(String),
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            Error::Parsing(s) => write!(f, "Parsing Error: {}", s),
        }
    }
}



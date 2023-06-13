//use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
	Unknown{message : String},
	Generic{message : String}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{:?}", self)
	}
}

impl std::error::Error for Error {}

impl Error
{
	pub fn append_message(mut self, new_message: String) -> Self {
		match self {
			Self::Unknown{message} => Self::Unknown{message: format!("{}\n{}", message, new_message)},
			Self::Generic{message} => Self::Generic{message: format!("{}\n{}", message, new_message)},
		}
	}
}

pub struct ErrorContext<'scope> {
	parent_opt : Option<&'scope Self>,
	contextualize_cb_opt : Option<&'scope dyn Fn(&mut dyn std::fmt::Write) -> Result<(), std::fmt::Error>>
}

impl<'scope> ErrorContext<'scope> {
	pub fn new(parent_opt : Option<&'scope Self>, contextualize_cb_opt : Option<&'scope dyn Fn(&mut dyn std::fmt::Write) -> Result<(), std::fmt::Error>>) -> Self {
		Self {parent_opt, contextualize_cb_opt}
	}
}
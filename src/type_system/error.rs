//use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
	Unknown{message : String},
	Generic{message : String}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			Self::Unknown{message} => write!(f, "Unknown error: {}", message),
			Self::Generic{message} => write!(f, "Generic error: {}", message),
		}
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

	pub fn generic_error(&self, m : & dyn std::fmt::Display) -> Error {
		Error::Generic{message: format!("{}\n{}", m, self)}
	}
}

impl<'scope> std::fmt::Display for ErrorContext<'scope> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		if let Some(cb) = self.contextualize_cb_opt {
			cb(f)?;
		}
		write!(f, "\n")?;
		if let Some(parent) = self.parent_opt {
			write!(f, "{}", parent)?;
		}
		return Ok(());
	}
}

// Based on assert_eq!
#[macro_export]
macro_rules! error_ifn_eq {
	($ctx:expr, $left:expr, $right:expr $(,)?) => {
		match (&$left, &$right) {
			(left_val, right_val) => {
				if !(*left_val == *right_val) {
					Err(assert_failed($ctx, &*left_val, &*right_val))
				}
				else {
					Ok(())
				}
			}
		}
	}
}

fn assert_failed<T : std::fmt::Debug>(error_context : &ErrorContext, a : T, b : T) -> Error {
	error_context.generic_error(& format!("{:?} != {:?}", a, b))
}
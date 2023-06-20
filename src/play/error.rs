use crate::parse::ParseError;
use std::io;

#[derive(Debug)]
pub enum PlayError {
	Io(io::Error),
	CannotParse(ParseError),
}

pub type PlayResult<T> = Result<T, PlayError>;

impl From<io::Error> for PlayError {
	fn from(err: io::Error) -> Self {
		Self::Io(err)
	}
}

impl From<ParseError> for PlayError {
	fn from(err: ParseError) -> Self {
		Self::CannotParse(err)
	}
}

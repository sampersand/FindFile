use std::io;

#[derive(Debug)]
pub enum PlayError {
	Io(io::Error),
}

pub type PlayResult<T> = Result<T, PlayError>;

impl From<io::Error> for PlayError {
	fn from(err: io::Error) -> Self {
		Self::Io(err)
	}
}

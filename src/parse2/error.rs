use crate::parse2::stream::SourcePosition;

#[derive(Debug)]
pub struct Error {
	whence: SourcePosition,
	kind: ErrorKind,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
	pub fn new(kind: ErrorKind, whence: SourcePosition) -> Self {
		Self { kind, whence }
	}

	pub fn whence(&self) -> SourcePosition {
		self.whence
	}

	pub fn kind(&self) -> &ErrorKind {
		&self.kind
	}
}

#[derive(Debug)]
pub enum ErrorKind {
	UnterminatedString,
	UnterminatedRegex,
	InvalidEscape,
	IntegerLiteralTooLarge,
}

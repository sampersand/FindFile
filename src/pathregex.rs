use std::ffi::OsStr;

#[derive(Debug, Clone, PartialEq)]
pub struct PathRegex {
	// todo
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathParseError {
	Todo,
}

impl PathRegex {
	pub fn parse(
		begin: crate::parse::token::BeginPathKind,
		source: &OsStr,
	) -> Result<Self, PathParseError> {
		todo!()
	}
}

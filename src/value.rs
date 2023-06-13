use crate::PathRegex;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Text(Vec<u8>),
	Number(f64),
	Path(PathBuf),
	PathRegex(PathRegex),
}

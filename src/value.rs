use crate::play::PlayResult;
use crate::PathRegex;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Text(Vec<u8>),
	Number(f64),
	Path(PathBuf),
	PathRegex(PathRegex),
}

impl Default for Value {
	fn default() -> Self {
		Self::Text(vec![])
	}
}

impl Value {
	pub fn is_truthy(&self) -> bool {
		match self {
			Self::Text(v) => !v.is_empty(),
			Self::Number(v) => *v != 0.0,
			_ => todo!(),
		}
	}

	pub fn negate(&self) -> PlayResult<Self> {
		todo!()
	}

	pub fn add(&self, rhs: &Self) -> PlayResult<Self> {
		todo!()
	}

	pub fn subtract(&self, rhs: &Self) -> PlayResult<Self> {
		todo!()
	}

	pub fn multiply(&self, rhs: &Self) -> PlayResult<Self> {
		todo!()
	}

	pub fn divide(&self, rhs: &Self) -> PlayResult<Self> {
		todo!()
	}

	pub fn modulo(&self, rhs: &Self) -> PlayResult<Self> {
		todo!()
	}

	pub fn compare(&self, rhs: &Self) -> PlayResult<std::cmp::Ordering> {
		match (self, rhs) {
			(Self::Number(lhs), Self::Number(rhs)) => {
				Ok(lhs.partial_cmp(&rhs).expect("todo: handle NaN <=> NaN"))
			}
			_ => todo!("{:?} {:?}", self, rhs),
		}
	}
}

impl From<bool> for Value {
	fn from(b: bool) -> Self {
		if b {
			Self::Text(vec![b'1'])
		} else {
			Self::Text(vec![])
		}
	}
}

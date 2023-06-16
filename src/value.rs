use crate::play::PlayContext;
use crate::play::PlayResult;
use crate::play::RunContext;
use crate::{FileSize, PathRegex, Regex};
use os_str_bytes::OsStrBytes;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Text(Rc<[u8]>),
	Number(f64),
	Path(PathBuf),
	PathRegex(PathRegex),
	FileSize(FileSize),
	Regex(Regex),
}

impl Default for Value {
	fn default() -> Self {
		Self::Text(vec![].into())
	}
}

fn slice_contains(haystack: &[u8], needle: &[u8]) -> bool {
	haystack.windows(needle.len()).any(|c| c == needle)
}

impl Value {
	pub fn is_truthy(&self) -> bool {
		match self {
			Self::Text(v) => !v.is_empty(),
			Self::Number(v) => *v != 0.0,
			_ => todo!(),
		}
	}

	pub fn matches(&self, rhs: &Self) -> PlayResult<bool> {
		match (self, rhs) {
			(Self::FileSize(lhs), Self::FileSize(rhs)) => Ok(lhs.fuzzy_matches(*rhs)),
			(Self::Regex(regex), Self::Text(rhs)) => Ok(regex.is_match(&rhs)),
			(Self::Text(needle), Self::Text(haystack)) => Ok(slice_contains(haystack, needle)),
			_ => todo!(),
		}
	}

	pub fn run(&self, ctx: &mut PlayContext, rctx: RunContext) -> PlayResult<Self> {
		match (self, rctx) {
			(Self::Text(s), RunContext::Logical) => {
				Ok({ ctx.is_file() && slice_contains(&ctx.contents()?, &s) }.into())
			}
			(Self::FileSize(size), RunContext::Logical) => Ok(size.fuzzy_matches(ctx.size()).into()),
			(Self::Regex(regex), RunContext::Logical) => Ok(regex.is_match(&ctx.contents()?).into()),

			(_, RunContext::Any) => Ok(self.clone()),
			(Self::Number(x), RunContext::Logical) => Ok((*x != 0.0).into()),
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
			(Self::FileSize(lhs), Self::FileSize(rhs)) => Ok(lhs.cmp(&rhs)),
			_ => todo!("{:?} {:?}", self, rhs),
		}
	}
}

impl From<bool> for Value {
	fn from(b: bool) -> Self {
		if b {
			Self::Number(1.0)
		} else {
			Self::Number(0.0)
		}
	}
}

impl From<FileSize> for Value {
	fn from(size: FileSize) -> Self {
		Self::FileSize(size)
	}
}

impl From<Rc<[u8]>> for Value {
	fn from(text: Rc<[u8]>) -> Self {
		Self::Text(text)
	}
}

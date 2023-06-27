use crate::play::{PlayContext, PlayResult, RunContext};
use crate::{FileSize, PathGlob, Regex};
use os_str_bytes::OsStrBytes;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::rc::Rc;

mod assoc_array;
pub use assoc_array::AssocArray;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	AssocArray(AssocArray),
	Text(Rc<[u8]>),
	Number(f64),
	Path(Rc<Path>),
	PathGlob(PathGlob),
	FileSize { fs: FileSize, precision: u8 },
	Regex(Regex),
}

impl Default for Value {
	fn default() -> Self {
		Self::Text(vec![].into())
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

	pub fn matches(&self, rhs: &Self) -> PlayResult<bool> {
		match (self, rhs) {
			(Self::FileSize { fs: lhs, precision }, Self::FileSize { fs: rhs, .. }) => {
				Ok(lhs.fuzzy_matches(*rhs, *precision))
			}
			(Self::Regex(regex), Self::Text(rhs)) => Ok(regex.is_match(&rhs)),
			(Self::Text(needle), Self::Text(haystack)) => Ok(crate::slice_contains(haystack, needle)),
			(Self::PathGlob(glob), Self::Path(path)) => Ok(glob.is_match(&path)),
			(Self::PathGlob(glob), Self::Text(path)) => {
				Ok(glob.is_match(std::path::Path::new(&OsStr::assert_from_raw_bytes(path.as_ref()))))
			}
			_ => todo!(),
		}
	}

	pub fn run(&self, ctx: &mut PlayContext, rctx: RunContext) -> PlayResult<Self> {
		match (self, rctx) {
			(Self::Text(s), RunContext::Logical) => {
				Ok((ctx.is_file() && ctx.info_mut().contents_contains(&s)?).into())
			}
			(Self::FileSize { fs, precision }, RunContext::Logical) => {
				Ok(fs.fuzzy_matches(ctx.info().content_size(), *precision).into())
			}

			(Self::Regex(regex), RunContext::Logical) => Ok(regex.is_match(&ctx.contents()?).into()),
			(Self::PathGlob(glob), RunContext::Logical) => {
				Ok(glob.is_match(&ctx.info().path()).into())
			}

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
			(Self::FileSize { fs: lhs, .. }, Self::FileSize { fs: rhs, .. }) => Ok(lhs.cmp(&rhs)),
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
		Self::FileSize { fs: size, precision: 0 }
	}
}

impl From<Rc<[u8]>> for Value {
	fn from(text: Rc<[u8]>) -> Self {
		Self::Text(text)
	}
}

impl From<Rc<Path>> for Value {
	fn from(path: Rc<Path>) -> Self {
		Self::Path(path)
	}
}

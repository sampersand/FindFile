use crate::play::{PlayContext, PlayResult, RunContextOld};
use crate::vm::RunResult;
use crate::vm::Vm;
use crate::{FileSize, PathGlob, Regex};
use os_str_bytes::OsStrBytes;
use os_str_bytes::RawOsStr;
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
	pub fn is_truthy_old(&self) -> bool {
		match self {
			Self::Text(v) => !v.is_empty(),
			Self::Number(v) => *v != 0.0,
			_ => todo!("{:?}", self),
		}
	}

	pub fn is_truthy(&self, vm: &mut Vm) -> RunResult<bool> {
		match self {
			Self::Text(v) => Ok(!v.is_empty()),
			Self::Number(v) => Ok(*v != 0.0),
			Self::AssocArray(ary) => Ok(!ary.is_empty()),
			Self::Path(path) => todo!(),
			Self::PathGlob(glob) => Ok(glob.is_match(&vm.info().path()._rc())),
			Self::FileSize { fs, precision } => {
				Ok(fs.fuzzy_matches(vm.info().content_size(), *precision))
			}
			Self::Regex(regex) => Ok(regex.is_match(&vm.info_mut().contents()?)),
		}
	}

	pub fn logical(&self, vm: &mut Vm) -> RunResult<bool> {
		match self {
			Self::Text(v) => Ok(vm.info_mut().contents_contains(v)?),
			other => self.is_truthy(vm),
		}
	}

	pub fn matches(&self, rhs: &Self) -> RunResult<bool> {
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

	pub fn run(&self, ctx: &mut PlayContext, rctx: RunContextOld) -> PlayResult<Self> {
		match (self, rctx) {
			(Self::Text(s), RunContextOld::Logical) => {
				Ok((ctx.is_file() && ctx.info_mut().contents_contains(&s)?).into())
			}
			(Self::FileSize { fs, precision }, RunContextOld::Logical) => {
				Ok(fs.fuzzy_matches(ctx.info().content_size(), *precision).into())
			}

			(Self::Regex(regex), RunContextOld::Logical) => {
				Ok(regex.is_match(&ctx.contents()?).into())
			}
			(Self::PathGlob(glob), RunContextOld::Logical) => {
				Ok(glob.is_match(&ctx.info().path()._rc()).into())
			}

			(_, RunContextOld::Any) => Ok(self.clone()),
			(Self::Number(x), RunContextOld::Logical) => Ok((*x != 0.0).into()),
			_ => todo!(),
		}
	}

	pub fn negate(&self) -> RunResult<Self> {
		todo!()
	}

	pub fn add(&self, rhs: &Self) -> RunResult<Self> {
		todo!()
	}

	pub fn subtract(&self, rhs: &Self) -> RunResult<Self> {
		todo!()
	}

	pub fn multiply(&self, rhs: &Self) -> RunResult<Self> {
		todo!()
	}

	pub fn divide(&self, rhs: &Self) -> RunResult<Self> {
		todo!()
	}

	pub fn modulo(&self, rhs: &Self) -> RunResult<Self> {
		todo!()
	}

	pub fn compare(&self, rhs: &Self) -> RunResult<std::cmp::Ordering> {
		match (self, rhs) {
			(Self::Number(lhs), Self::Number(rhs)) => {
				Ok(lhs.partial_cmp(&rhs).expect("todo: handle NaN <=> NaN"))
			}
			(Self::FileSize { fs: lhs, .. }, Self::FileSize { fs: rhs, .. }) => Ok(lhs.cmp(&rhs)),
			(Self::Text(lhs), Self::Text(rhs)) => Ok(lhs.cmp(rhs)),
			_ => todo!("{:?} {:?}", self, rhs),
		}
	}

	pub fn call(&self, args: &[Self]) -> RunResult<Self> {
		todo!();
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

impl From<&Path> for Value {
	fn from(path: &Path) -> Self {
		Self::Path(Rc::from(path.to_owned()))
	}
}

impl From<&OsStr> for Value {
	fn from(osstr: &OsStr) -> Self {
		Self::Text(osstr.to_raw_bytes().to_owned().into())
	}
}

impl From<&RawOsStr> for Value {
	fn from(osstr: &RawOsStr) -> Self {
		Self::Text(osstr.as_raw_bytes().to_owned().into())
	}
}

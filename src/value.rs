use crate::vm::{RunError, RunResult, Vm};
use crate::{FileSize, PathGlob, Regex};
use os_str_bytes::OsStrBytes;
use os_str_bytes::RawOsStr;
use std::ffi::OsStr;
use std::path::Path;
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
		Self::from(<&RawOsStr>::default())
	}
}

impl Value {
	pub fn is_truthy(&self) -> bool {
		match self {
			Self::Text(v) => !v.is_empty(),
			Self::Number(v) => *v != 0.0,
			Self::AssocArray(ary) => !ary.is_empty(),
			Self::FileSize { fs, precision: _ } => !fs.is_empty(),
			Self::Path(_path) => todo!(),
			Self::PathGlob(_glob) => todo!(),
			Self::Regex(_regex) => todo!(),
		}
	}

	pub fn logical(&self, vm: &mut Vm) -> RunResult<bool> {
		match self {
			Self::Text(v) => Ok(vm.info_mut().contents_contains(v)?),
			Self::PathGlob(glob) => Ok(glob.is_match(&vm.info().path()._rc())),
			Self::FileSize { fs, precision } => {
				Ok(fs.fuzzy_matches(vm.info().content_size(), *precision))
			}
			Self::Regex(regex) => Ok(regex.is_match(&vm.info_mut().contents()?)),
			_other => Ok(self.is_truthy()),
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

	pub fn typename(&self) -> &'static str {
		match self {
			Self::AssocArray(_) => "array",
			Self::Text(_) => "string",
			Self::Number(_) => "number",
			Self::Path(_) => "path",
			Self::PathGlob(_) => "pathglob",
			Self::FileSize { .. } => "filesize",
			Self::Regex(_) => "regex",
		}
	}

	pub fn negate(&self) -> RunResult<Self> {
		match self {
			Self::Number(num) => Ok(Self::Number(-num)),
			_ => Err(RunError::InvalidType { func: "unary-", given: self.typename() }),
		}
	}

	pub fn add(&self, rhs: &Self) -> RunResult<Self> {
		match (self, rhs) {
			(Self::Number(lhs), Self::Number(rhs)) => Ok((lhs + rhs).into()),
			_ => todo!("{:?} {:?}", self, rhs),
		}
	}

	pub fn subtract(&self, _rhs: &Self) -> RunResult<Self> {
		todo!()
	}

	pub fn multiply(&self, _rhs: &Self) -> RunResult<Self> {
		todo!()
	}

	pub fn divide(&self, _rhs: &Self) -> RunResult<Self> {
		todo!()
	}

	pub fn modulo(&self, _rhs: &Self) -> RunResult<Self> {
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

	pub fn call(&self, _args: &[Self]) -> RunResult<Self> {
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

impl From<f64> for Value {
	fn from(num: f64) -> Self {
		Self::Number(num)
	}
}

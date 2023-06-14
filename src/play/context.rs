use crate::play::Program;
use os_str_bytes::OsStringBytes;
use std::ffi::{OsStr, OsString};
use std::fs::DirEntry;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct PlayContext<'a> {
	entry: DirEntry,
	path: PathBuf,
	program: &'a mut Program,
	contents: Option<OsString>,
}

impl<'a> PlayContext<'a> {
	pub fn new(program: &'a mut Program, entry: DirEntry) -> Self {
		Self { path: entry.path(), entry, program, contents: None }
	}

	pub fn path(&self) -> &Path {
		&self.path
	}

	pub fn take_path(self) -> PathBuf {
		self.path
	}

	pub fn is_dir(&self) -> std::io::Result<bool> {
		Ok(self.entry.file_type()?.is_dir())
	}

	pub fn is_file(&self) -> std::io::Result<bool> {
		Ok(self.entry.file_type()?.is_file())
	}

	pub fn contents(&mut self) -> std::io::Result<&OsStr> {
		if self.contents.is_none() {
			self.contents = Some(OsString::assert_from_raw_vec(std::fs::read(self.path())?));
		}

		Ok(self.contents.as_deref().unwrap())
	}
}

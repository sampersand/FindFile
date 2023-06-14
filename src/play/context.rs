use crate::play::{PlayResult, Program};
use crate::Value;
use os_str_bytes::OsStringBytes;
use std::ffi::{OsStr, OsString};
use std::fs::{DirEntry, FileType, Metadata};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct PlayContext<'a> {
	path: PathBuf,
	program: &'a mut Program,
	contents: Option<OsString>,
	file_type: FileType,
	metadata: Metadata,
}

impl<'a> PlayContext<'a> {
	pub fn new(program: &'a mut Program, entry: DirEntry) -> io::Result<Self> {
		let path = entry.path();
		let file_type = entry.file_type()?;
		let metadata = entry.metadata()?;

		Ok(Self { path, program, contents: None, file_type, metadata })
	}

	pub fn path(&self) -> &Path {
		&self.path
	}

	pub fn take_path(self) -> PathBuf {
		self.path
	}

	pub fn len(&self) -> u64 {
		self.metadata.len()
	}

	pub fn is_dir(&self) -> bool {
		self.file_type.is_dir()
	}

	pub fn is_file(&self) -> bool {
		self.file_type.is_file()
	}

	pub fn contents(&mut self) -> io::Result<&OsStr> {
		if self.contents.is_none() {
			self.contents = Some(OsString::assert_from_raw_vec(std::fs::read(self.path())?));
		}

		Ok(self.contents.as_deref().unwrap())
	}

	pub fn lookup_var(&self, name: &str) -> Value {
		match name {
			"size" => Value::Number(self.len() as f64), // todo: make this filesize
			_ => self.program.vars.get(name).cloned().unwrap_or_default(),
		}
	}

	pub fn assign_var(&mut self, name: &str, value: Value) {
		self.program.vars.insert(name.into(), value);
	}
}

use crate::play::{PlayResult, Program};
use crate::FileSize;
use crate::Value;
use os_str_bytes::OsStringBytes;
use std::ffi::{OsStr, OsString};
use std::fs::{DirEntry, FileType, Metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug)]
pub struct PlayContext<'a> {
	path: PathBuf,
	program: &'a mut Program,
	contents: Option<Rc<[u8]>>,
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

	pub fn size(&self) -> FileSize {
		FileSize::from_bytes(self.metadata.len(), None)
	}

	pub fn is_dir(&self) -> bool {
		self.file_type.is_dir()
	}

	pub fn is_file(&self) -> bool {
		self.file_type.is_file()
	}

	pub fn contents(&mut self) -> io::Result<Rc<[u8]>> {
		if self.contents.is_none() {
			self.contents = Some(std::fs::read(self.path())?.into());
		}

		Ok(self.contents.clone().unwrap())
	}

	pub fn lookup_var(&mut self, name: &str) -> PlayResult<Value> {
		match name {
			"dir?" | "directory?" | "d?" => Ok(self.is_dir().into()),
			"file?" | "f?" => Ok(self.is_file().into()),
			"size" | "z" => Ok(self.size().into()),
			"contents" | "c" => Ok(self.contents()?.into()),
			_ => Ok(self.program.vars.get(name).cloned().unwrap_or_default()),
		}
	}

	pub fn assign_var(&mut self, name: &str, value: Value) {
		self.program.vars.insert(name.into(), value);
	}
}

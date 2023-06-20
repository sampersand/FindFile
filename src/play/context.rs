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
	program: &'a mut Program,
	pub file_info: FileInfo,
}

#[derive(Debug)]
pub struct FileInfo {
	pub path: Rc<Path>,
	pub contents: Option<Rc<[u8]>>,
	pub file_type: FileType,
	pub metadata: Metadata,
}

impl<'a> PlayContext<'a> {
	pub fn new(program: &'a mut Program, entry: DirEntry) -> io::Result<Self> {
		let path = entry.path().into();
		let file_type = entry.file_type()?;
		let metadata = entry.metadata()?;

		Ok(Self { program, file_info: FileInfo { path, contents: None, file_type, metadata } })
	}

	pub fn path(&self) -> &Path {
		&self.file_info.path
	}

	pub fn size(&self) -> FileSize {
		FileSize::from_bytes(self.file_info.metadata.len(), None)
	}

	pub fn is_dir(&self) -> bool {
		self.file_info.file_type.is_dir()
	}

	pub fn is_file(&self) -> bool {
		self.file_info.file_type.is_file()
	}

	pub fn contents(&mut self) -> io::Result<Rc<[u8]>> {
		if self.file_info.contents.is_none() {
			self.file_info.contents = Some(std::fs::read(self.path())?.into());
		}

		Ok(self.file_info.contents.clone().unwrap())
	}

	pub fn lookup_var(&mut self, name: &str) -> PlayResult<Value> {
		match name {
			"dir?" | "directory?" | "d?" => Ok(self.is_dir().into()),
			"file?" | "f?" => Ok(self.is_file().into()),
			"size" | "z" => Ok(self.size().into()),
			"contents" | "c" => Ok(self.contents()?.into()),
			"path" | "p" => Ok(self.file_info.path.clone().into()),
			_ => Ok(self.program.get_var(name).unwrap_or_default()),
		}
	}

	pub fn assign_var(&mut self, name: &str, value: Value) {
		self.program.assign_var(name, value);
	}
}

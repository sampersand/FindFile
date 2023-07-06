use crate::play::{PathInfo, PlayResult, Program};
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
	pathinfo: PathInfo,
}

impl<'a> PlayContext<'a> {
	pub fn new(program: &'a mut Program, path: PathBuf) -> io::Result<Self> {
		Ok(Self { program, pathinfo: PathInfo::new(&path)? })
	}

	pub fn info(&self) -> &PathInfo {
		&self.pathinfo
	}

	pub fn info_mut(&mut self) -> &mut PathInfo {
		&mut self.pathinfo
	}

	pub fn into_pathinfo(self) -> PathInfo {
		self.pathinfo
	}

	pub fn is_dir(&self) -> bool {
		self.pathinfo.is_dir()
	}

	pub fn is_file(&self) -> bool {
		self.pathinfo.is_file()
	}

	pub fn contents(&mut self) -> io::Result<Rc<[u8]>> {
		self.pathinfo.contents()
	}

	pub fn lookup_var(&mut self, name: &str) -> PlayResult<Value> {
		match name {
			"dir?" | "directory?" | "d?" => Ok(self.is_dir().into()),
			"file?" | "f?" => Ok(self.is_file().into()),
			"size" | "z" => Ok(self.info().content_size().into()),
			"contents" | "c" => Ok(self.pathinfo.contents()?.into()),
			"path" | "p" => Ok(self.pathinfo.path()._rc().clone().into()),
			_ => Ok(self.program.get_var(name).unwrap_or_default()),
		}
	}

	pub fn assign_var(&mut self, name: &str, value: Value) {
		self.program.assign_var(name, value);
	}
}

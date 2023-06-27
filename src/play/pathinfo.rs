use crate::FileSize;
use std::fs::Metadata;
use std::io;
use std::path::Path;
use std::rc::Rc;

#[derive(Debug)]
pub struct PathInfo {
	path: Rc<Path>,
	contents: FileContents,
	metadata: Metadata,
}

#[derive(Debug)]
pub struct FileContents {
	contents: Option<Rc<[u8]>>,
}

impl PathInfo {
	/// Creates a new [`PathInfo`]. Returns an error if there was a problem reading the metadata.
	pub fn new<P: AsRef<Path> + ?Sized>(path: &P) -> io::Result<Self> {
		let path = Rc::from(path.as_ref().to_owned());
		let metadata = std::fs::metadata(&path)?;

		Ok(Self { path, metadata, contents: FileContents { contents: None } })
	}

	/// Fetches the name of the file.
	pub fn path(&self) -> &Rc<Path> {
		&self.path
	}

	/// Returns the metadata of the file.
	pub fn metadata(&self) -> &Metadata {
		&self.metadata
	}

	/// Returns whether the file is a directory. (todo: Does it follow symlinks)
	pub fn is_dir(&self) -> bool {
		self.metadata.is_dir()
	}

	/// Returns whether the file is a directory. (todo: Does it follow symlinks)
	pub fn is_file(&self) -> bool {
		self.metadata.is_file()
	}

	/// Returns the contents of the path.
	pub fn contents(&mut self) -> io::Result<Rc<[u8]>> {
		if self.contents.contents.is_none() {
			self.contents.contents = Some(std::fs::read(self.path())?.into());
		}

		Ok(self.contents.contents.clone().unwrap())
	}

	/// Returns the size of `contents`.
	pub fn content_size(&self) -> FileSize {
		FileSize::from_bytes(self.metadata.len())
	}

	/// Returns whether `self` contains `slice`.
	pub fn contents_contains(&mut self, slice: &[u8]) -> io::Result<bool> {
		Ok(crate::slice_contains(&self.contents()?, slice))
	}
}

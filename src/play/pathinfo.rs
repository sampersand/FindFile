use crate::FileSize;
use os_str_bytes::OsStrBytes;
use os_str_bytes::{RawOsStr, RawOsString};
use std::ffi::OsStr;
use std::fs::Metadata;
use std::io;
use std::path::Path;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct PathInfo {
	path: RawOsString,
	contents: FileContents,
	metadata: Metadata,
}

#[derive(Debug, Clone)]
pub struct FileContents {
	contents: Option<Rc<[u8]>>,
}

impl PathInfo {
	/// Creates a new [`PathInfo`]. Returns an error if there was a problem reading the metadata.
	pub fn new<P: AsRef<Path> + ?Sized>(path: &P) -> io::Result<Self> {
		let path = path.as_ref().to_owned();
		let metadata = std::fs::metadata(&path)?;
		debug_assert!(!path.as_os_str().is_empty());
		let path = RawOsString::new(path.into());

		Ok(Self { path, metadata, contents: FileContents { contents: None } })
	}

	/// Fetches the name of the file.
	pub fn path(&self) -> &crate::play::Path {
		crate::play::Path::new(&self.path)
	}

	pub fn is_hidden(&self) -> bool {
		let filename = self.path()._rc();
		let filename = filename.file_name().expect("pathinfo called with `..` or `.`?");
		filename.to_raw_bytes()[0] == b'.'
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
			self.contents.contents = Some(std::fs::read(self.path()._rc())?.into());
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

	/// Returns the name of the enclosing directory, working on the string itself.
	///
	/// # Caveats
	/// - If the path is a relative path of one component, it'll return `.`
	/// - If the path is the root directory or a windows prefix, it'll return `self.path()`.
	/// - Otherwise, it'll return the parent directory.
	pub fn dirname(&self) -> Rc<Path> {
		debug_assert_ne!(self.path()._rc().to_str(), Some(""));

		match self.path()._rc().parent() {
			Some(p) if p.as_os_str().is_empty() => {
				Path::new(match OsStr::assert_from_raw_bytes(".".as_bytes()) {
					std::borrow::Cow::Borrowed(b) => b,
					_ => unreachable!(),
				})
				.into()
			}
			None => self.path()._rc(), // return self if it's a dirname
			Some(other) => other.into(),
		}
	}

	/// Returns the the extension of the path.
	///
	/// # Caveats
	/// - If the basename starts with `.` and contains no other `.`s, the return value is empty.
	/// - If the basename has more than one `.`
	/// - If the path doesn't have a `.`,
	/// - If the path is a relative path of one component, it'll return `.`
	/// - If the path is the root directory or a windows prefix, it'll return `self.path()`.
	/// - Otherwise, it'll return the parent directory.
	/*

	./foo          => <empty>
	./foo.         => .
	./foo.bar      => .bar
	./foo.bar.baz  => .baz
	./.foo         => <empty>
	./.foo.        => .
	./.foo.bar     => .bar
	./.foo.bar.baz => .baz
		*/
	pub fn extname(&self) -> Option<&OsStr> {
		// self.basename_segments().last()
		todo!()
	}

	pub fn extnamedot(&self) -> &Path {
		todo!()
	}

	pub fn basename(&self) -> &OsStr {
		// self.path().file_name()
		todo!()
	}

	// /// Returns the stemname of the path, or none if the path is `.`, `/`, or ends in `..`
	// pub fn stemname(&self) -> Option<&OsStr> {
	// 	self.path()._rc().file_stem()
	// }
}

mod filesize;
pub use filesize::FileSize;

#[derive(Debug)]
pub enum Value<'a> {
	FileSize(FileSize),
	// Filename(),
}

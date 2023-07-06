use os_str_bytes::RawOsStr;

#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Path(RawOsStr);

impl Path {
	pub fn new(t: &RawOsStr) -> &Self {
		unsafe { &*(t as *const RawOsStr as *const Self) }
	}

	pub fn _rc(&self) -> std::rc::Rc<std::path::Path> {
		use os_str_bytes::OsStringBytes;
		std::rc::Rc::from(std::path::PathBuf::from(std::ffi::OsString::assert_from_raw_vec(
			self.0.as_raw_bytes().to_owned(),
		)))
	}

	pub fn parent(&self) -> &RawOsStr {
		if let Some((parent, _base)) = self.0.rsplit_once(std::path::MAIN_SEPARATOR) {
			parent
		} else {
			&self.0 // parent of `.` or `/` is `.` or `/`
		}
	}

	/// Returns the part of the path following the `/`. If No `/` is present, it'll just
	/// the base.
	pub fn base(&self) -> &RawOsStr {
		if let Some((_parent, base)) = self.0.rsplit_once(std::path::MAIN_SEPARATOR) {
			base
		} else {
			&self.0
		}
	}

	/// Returns the basename without the extension. That is `stem + ext == base`.
	///
	/// If no extension exists, or if the filename starts with `.` and has no other `.`, it returns
	/// the basename.
	pub fn stem(&self) -> &RawOsStr {
		let base = self.base();

		match base.rsplit_once('.') {
			None => base,
			Some((_, post)) if post.is_empty() => base,
			Some((stem, _)) => stem,
		}
	}

	/// Returns the extension.
	///
	/// Returns the component of `base` not covered by `stem`.
	pub fn extension(&self) -> &RawOsStr {
		&self.base()[self.stem().raw_len()..]
	}
}

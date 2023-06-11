use std::ffi::OsStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
	WithinPath,
	EndOfPath,
	DollarSignEscape,
	BraceEscape,
}

#[derive(Debug)]
pub struct Stream<'a> {
	index: usize,

	pub phases: Vec<Phase>,

	#[cfg(any(unix, wasi))]
	source: &'a [u8],

	#[cfg(not(any(unix, wasi)))]
	source: Vec<u8>,
}

impl<'a> Stream<'a> {
	pub fn new(source: &'a OsStr) -> Self {
		Self {
			index: 0,
			phases: vec![],

			#[cfg(any(unix, wasi))]
			source: std::os::unix::ffi::OsStrExt::as_bytes(source),

			#[cfg(not(any(unix, wasi)))]
			source: source.to_str().expect("todo: deal with non-bytes").as_bytes().to_vec(),
		}
	}

	pub fn is_eof(&self) -> bool {
		self.index >= self.source.len()
	}

	#[must_use]
	pub fn peek(&self) -> Option<u8> {
		self.source.get(self.index).copied()
	}

	pub fn advance(&mut self) {
		debug_assert!(self.index <= self.source.len());
		self.index += 1;
	}

	pub fn unadvance(&mut self) {
		debug_assert_ne!(self.index, 0); // technically redundant with builtin wrap checks
		self.index -= 1;
	}

	pub fn take(&mut self) -> Option<u8> {
		let next = self.peek()?;
		self.advance();
		Some(next)
	}

	fn as_u8(&self) -> &[u8] {
		self.source.as_ref()
	}

	pub fn take_if_byte(&mut self, what: u8) -> bool {
		if !self.peek().map_or(false, |x| what == x) {
			return false;
		}
		self.advance();
		true
	}
	pub fn take_if_starts_with(&mut self, what: &[u8]) -> bool {
		if !self.as_u8()[self.index..].starts_with(what) {
			return false;
		}

		self.index += what.len();
		true
	}

	pub fn take_while(&mut self, mut func: impl FnMut(u8) -> bool) -> Option<Vec<u8>> {
		let mut acc = Vec::new();

		while self.peek().map_or(false, &mut func) {
			acc.push(self.take().unwrap());
		}

		if acc.is_empty() {
			None
		} else {
			Some(acc)
		}
	}
}

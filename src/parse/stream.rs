#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
	WithinString,
	WithinPath,
	DollarSignEscape,
	BraceEscape,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stream<'a> {
	index: usize,
	source: &'a [u8],
	pub phases: Vec<Phase>,
}

impl<'a> Stream<'a> {
	pub fn new(source: &'a [u8]) -> Self {
		Self { index: 0, phases: vec![], source }
	}

	/// Returns whether the stream is currently at end of file.
	#[must_use]
	pub fn is_eof(&self) -> bool {
		self.source.len() <= self.index
	}

	/// Returns the first character without consuming it.
	#[must_use]
	pub fn peek(&self) -> Option<u8> {
		self.remainder().get(0).copied()
	}

	/// Advances the stream by `len` bytes.
	pub fn advance_by(&mut self, len: usize) {
		self.index += len;
		debug_assert!(self.index <= self.source.len());
	}

	/// Undoes a `take`.
	pub fn untake(&mut self) {
		debug_assert_ne!(self.index, 0); // technically redundant with builtin wrap checks
		self.index -= 1;
	}

	/// Returns the first character of the stream and advances. If at end of stream,
	/// returns `None`.
	pub fn take(&mut self) -> Option<u8> {
		let next = self.peek()?;
		self.advance_by(1);
		Some(next)
	}

	/// Returns the first character of the stream and advances. If at end of stream,
	/// returns `None`.
	pub fn take_n<const N: usize>(&mut self) -> Option<[u8; N]> {
		let x = <[u8; N]>::try_from(self.remainder().get(..N)?).unwrap();
		self.advance_by(N);
		Some(x)
	}

	/// The remaining bytes in the source stream
	#[must_use]
	pub fn remainder(&self) -> &[u8] {
		&self.source[self.index..]
	}

	/// Advances the stream forward if `condition` matches.
	pub fn advance_if<C: TakeCondition>(&mut self, mut condition: C) -> bool {
		let Some(match_len) = condition.match_len(self.remainder()) else {
			return false;
		};

		self.advance_by(match_len);
		true
	}

	/// Same
	pub fn take_while<F: FnMut(&u8) -> bool>(&mut self, mut condition: F) -> &'a [u8] {
		let start = self.index;

		while self.advance_if(&mut condition) {
			// do nothing
		}

		&self.source[start..self.index]
	}
}

pub trait TakeCondition {
	// how many bytes matched is the `usize`
	fn match_len(&mut self, source: &[u8]) -> Option<usize>;
}

impl TakeCondition for u8 {
	fn match_len(&mut self, source: &[u8]) -> Option<usize> {
		(source.get(0) == Some(self)).then_some(1)
	}
}

impl TakeCondition for &[u8] {
	fn match_len(&mut self, source: &[u8]) -> Option<usize> {
		source.starts_with(self).then_some(self.len())
	}
}

impl<F: FnMut(&u8) -> bool> TakeCondition for F {
	fn match_len(&mut self, source: &[u8]) -> Option<usize> {
		self(source.get(0)?).then_some(1)
	}
}

impl<const N: usize> TakeCondition for &[u8; N] {
	fn match_len(&mut self, source: &[u8]) -> Option<usize> {
		self.as_slice().match_len(source)
	}
}

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
	pub fn remainder(&self) -> &'a [u8] {
		&self.source[self.index..]
	}

	/// Advances the stream forward if `condition` matches.
	pub fn take_if<C: TakeCondition<'a>>(&mut self, mut condition: C) -> Option<C::Output> {
		condition.take_from(self)
	}

	/// Advances the stream forward if `condition` matches.
	pub fn advance_if<C: TakeCondition<'a>>(&mut self, mut condition: C) -> bool {
		self.take_if(condition).is_some()
	}

	/// Same
	pub fn take_while<F: FnMut(u8) -> bool>(&mut self, mut condition: F) -> &'a [u8] {
		let start = self.index;

		while self.advance_if(&mut condition) {
			// do nothing
		}

		&self.source[start..self.index]
	}
}

pub trait TakeCondition<'a> {
	type Output;

	fn take_from(&mut self, stream: &mut Stream<'a>) -> Option<Self::Output>;
}

impl<'a> TakeCondition<'a> for u8 {
	type Output = Self;

	fn take_from(&mut self, stream: &mut Stream<'a>) -> Option<Self::Output> {
		(stream.peek() == Some(*self)).then(|| stream.take()).flatten()
	}
}

impl<'a> TakeCondition<'a> for &[u8] {
	type Output = Self;

	fn take_from(&mut self, stream: &mut Stream<'a>) -> Option<Self::Output> {
		if !stream.remainder().starts_with(self) {
			return None;
		}

		stream.advance_by(self.len());
		Some(self)
	}
}

impl<'a, F: FnMut(u8) -> bool> TakeCondition<'a> for F {
	type Output = u8;

	fn take_from(&mut self, stream: &mut Stream<'a>) -> Option<Self::Output> {
		self(stream.peek()?).then(|| stream.take()).flatten()
	}
}

impl<'a, 'b, const N: usize> TakeCondition<'a> for &'b [u8; N] {
	type Output = &'b [u8];

	fn take_from(&mut self, stream: &mut Stream<'a>) -> Option<Self::Output> {
		self.as_slice().take_from(stream)
	}
}

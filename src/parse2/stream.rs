use crate::parse2::error::{Error, ErrorKind};
use std::fmt::{self, Display, Formatter};
use std::str::Chars;

#[derive(Debug, Clone)]
pub struct Stream<'a> {
	source: &'a str,
	chars: Chars<'a>,
}

/// A position in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourcePosition {
	/// The line; the first line of the file is line 1.
	pub line: usize,
	/// The Column; the column of each line is 1.
	pub column: usize,
}

impl SourcePosition {
	/// Returns the position at the end of the source.
	pub fn at_end(source: &str) -> Self {
		// TODO: replace this with `.count('\n')+1` and then remaining_len+1.
		let mut line = 1;
		let mut column = 1;

		for chr in source.chars() {
			if chr == '\n' {
				line += 1;
				column = 1;
			} else {
				column += 1;
			}
		}

		Self { line, column }
	}
}

impl Display for SourcePosition {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}:{}", self.line, self.column)
	}
}

impl<'a> Stream<'a> {
	#[must_use]
	pub fn new(source: &'a str) -> Self {
		Self { source, chars: source.chars() }
	}

	/// Returns whether we're at the end of file
	#[must_use]
	pub fn is_eof(&self) -> bool {
		self.peek().is_none()
	}

	/// Returns a tuple of `(current line, current column)`.
	#[must_use]
	pub fn source_position(&self) -> SourcePosition {
		let source = std::str::from_utf8(
			&self.source.as_bytes()[..self.source.len() - self.chars.as_str().len()],
		)
		.expect("the slicing didn't work?");
		SourcePosition::at_end(source)
	}

	/// Creates a `kind` [`Error`] that happens at the current location.
	#[must_use]
	pub fn error(&self, kind: ErrorKind) -> Error {
		Error::new(kind, self.source_position())
	}

	/// Returns the next character, if it exists.
	#[must_use]
	pub fn peek(&self) -> Option<char> {
		self.chars.clone().next()
	}

	/// Takes the next character.
	pub fn take(&mut self) -> Option<char> {
		self.chars.next()
	}

	/// "Untake"s the character `chr` from the source. Note that it must be the last character
	/// yielded from `.take()`.
	pub fn untake(&mut self, chr: char) {
		let chrlen = chr.len_utf8();
		let source_without_remainder = &self.source[..self.source.len() - self.chars.as_str().len()];
		let (last_index, _) = source_without_remainder.char_indices().next_back().unwrap();
		self.chars = self.source[last_index..].chars();
		debug_assert_eq!(self.peek(), Some(chr));
	}

	pub fn take_if<T: TakeIf<'a>>(&mut self, cond: T) -> Option<T::Output> {
		cond.take_if(self)
	}

	pub fn remainder(&self) -> &'a str {
		self.chars.as_str()
	}

	pub fn take_while(&mut self, mut cond: impl FnMut(char) -> bool) -> Option<&'a str> {
		let start = self.remainder();
		while self.take_if(&mut cond).is_some() {
			// do nothing
		}
		let stop = self.remainder();

		if std::ptr::eq(start, stop) {
			return None;
		}

		Some(&start[..start.len() - stop.len()])
	}
}

pub trait TakeIf<'a> {
	type Output;
	fn take_if(self, stream: &mut Stream<'a>) -> Option<Self::Output>;
}

impl TakeIf<'_> for char {
	type Output = char;
	fn take_if(self, stream: &mut Stream<'_>) -> Option<Self::Output> {
		stream.take_if(|c| c == self)
	}
}

impl<F: FnOnce(char) -> bool> TakeIf<'_> for F {
	type Output = char;
	fn take_if(self, stream: &mut Stream<'_>) -> Option<Self::Output> {
		if stream.peek().map_or(false, self) {
			Some(stream.take().unwrap())
		} else {
			None
		}
	}
}

impl<'a> TakeIf<'a> for &'_ str {
	type Output = &'a str;
	fn take_if(self, stream: &mut Stream<'_>) -> Option<Self::Output> {
		todo!()
		// if stream.peek().map_or(false, self) {
		// 	Some(stream.take().unwrap())
		// } else {
		// 	None
		// }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn is_eof() {
		let stream = Stream::new("");
		assert!(stream.is_eof());

		let mut stream = Stream::new("f");
		assert!(!stream.is_eof());
		stream.take();
		assert!(stream.is_eof());
	}

	#[test]
	fn source_position() {
		let mut stream = Stream::new("A b\ncd\n\nd\n");
		for (line, column) in
			[(1, 1), (1, 2), (1, 3), (1, 4), (2, 1), (2, 2), (2, 3), (3, 1), (4, 1), (4, 2), (5, 1)]
		{
			assert_eq!(SourcePosition { line, column }, stream.source_position());
			stream.take();
		}
	}

	#[test]
	fn error() {
		let mut stream = Stream::new("a\nb");

		macro_rules! assert_err {
			() => {
				let err = stream.error(ErrorKind::InvalidEscape);
				assert_eq!(stream.source_position(), err.whence());
				assert!(matches!(err.kind(), ErrorKind::InvalidEscape));
			};
		}

		assert_err!();

		assert_eq!(Some('a'), stream.take());
		assert_err!();

		assert_eq!(Some('\n'), stream.take());
		assert_err!();

		assert_eq!(Some('b'), stream.take());
		assert_err!();

		assert_eq!(None, stream.take());
		assert_err!();
	}

	#[test]
	fn peek() {
		let mut stream = Stream::new("abc");
		assert_eq!(Some('a'), stream.peek());
		assert_eq!(Some('a'), stream.take());

		assert_eq!(Some('b'), stream.peek());
		assert_eq!(Some('b'), stream.take());

		assert_eq!(Some('c'), stream.peek());
		assert_eq!(Some('c'), stream.take());

		assert_eq!(None, stream.peek());
	}

	#[test]
	fn untake() {
		let mut stream = Stream::new("a😊❦🎲f🀄");

		assert_eq!(Some('a'), stream.take());
		assert_eq!(Some('😊'), stream.take());
		assert_eq!(Some('❦'), stream.take());
		stream.untake('❦');
		assert_eq!(Some('❦'), stream.take());
		assert_eq!(Some('🎲'), stream.take());
		stream.untake('🎲');
		stream.untake('❦');
		stream.untake('😊');
		assert_eq!(Some('😊'), stream.take());
		assert_eq!(Some('❦'), stream.take());
		assert_eq!(Some('🎲'), stream.take());
		assert_eq!(Some('f'), stream.take());
		assert_eq!(Some('🀄'), stream.take());
		stream.untake('🀄');
		assert_eq!(Some('🀄'), stream.take());
	}
}

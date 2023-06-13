use crate::parse::{ParseError, Stream, Token};
use std::ffi::OsStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
	WithinPath,
	WithinString,
	WithinRegex,
	DollarSignEscape,
	BraceEscape,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexContext<'a> {
	pub(super) stream: Stream<'a>,
	phases: Vec<Phase>,
	tokens: Vec<Token>,
}

impl<'a> LexContext<'a> {
	pub fn new(source: &'a OsStr) -> Self {
		Self {
			stream: Stream::new(source),
			phases: Vec::with_capacity(2), // sensible defaults
			tokens: Vec::with_capacity(2),
		}
	}

	pub fn get_cli(&self, pos: isize) -> Option<&OsStr> {
		todo!()
	}

	pub fn get_env(&self, pos: &OsStr) -> Option<&OsStr> {
		todo!()
	}

	pub fn phase(&self) -> Option<Phase> {
		self.phases.last().copied()
	}

	pub fn push_phase(&mut self, phase: Phase) {
		self.phases.push(phase);
	}

	pub fn pop_phase(&mut self, expected_phase: Phase) {
		let current_phase = self.phases.pop();
		assert_eq!(current_phase, Some(expected_phase));
	}

	pub fn pop_phase_unchecked(&mut self) -> Option<Phase> {
		self.phases.pop()
	}

	pub fn push_token(&mut self, token: Token) {
		self.tokens.push(token);
	}

	pub fn take_if_fn(
		&mut self,
		cond: impl FnOnce(&Token) -> bool,
	) -> Result<Option<Token>, ParseError> {
		if !self.peek()?.map_or(false, cond) {
			return Ok(None);
		}

		Ok(Some(self.next().unwrap().unwrap()))
	}

	pub fn peek(&mut self) -> Result<Option<&Token>, ParseError> {
		if self.tokens.is_empty() {
			if let Some(token) = Token::parse(self)? {
				self.tokens.push(token);
			}
		}

		Ok(self.tokens.last())
	}

	pub fn take_if(&mut self, token: Token) -> Result<bool, ParseError> {
		if self.peek()? == Some(&token) {
			let _ = self.next()?;
			Ok(true)
		} else {
			Ok(false)
		}
	}

	pub fn next(&mut self) -> Result<Option<Token>, ParseError> {
		if let Some(token) = self.tokens.pop() {
			return Ok(Some(token));
		}

		Token::parse(self)
	}
}

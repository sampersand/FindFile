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
pub struct ParseContext<'a> {
	pub(super) stream: Stream<'a>,
	phases: Vec<Phase>,
	tokens: Vec<Token>,
}

impl<'a> ParseContext<'a> {
	pub fn new(source: &'a OsStr) -> Self {
		Self {
			stream: Stream::new(source),
			phases: Vec::with_capacity(2), // sensible defaults
			tokens: Vec::with_capacity(2),
		}
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

	pub fn push_token(&mut self, token: Token) {
		self.tokens.push(token);
	}

	pub fn next(&mut self) -> Result<Option<Token>, ParseError> {
		if let Some(token) = self.tokens.pop() {
			return Ok(Some(token));
		}

		Token::parse(self)
	}
}

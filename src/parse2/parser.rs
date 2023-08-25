// use crate::parse2::token::{Operator, Parsable, Primary};
use crate::parse2::{Result, Stream, Token};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
	Text,
	Regex,
	Path,
}

pub struct Parser<'a> {
	stream: Stream<'a>,
	phases: Vec<Phase>,
}

impl<'a> Parser<'a> {
	pub fn new(source: &'a str) -> Self {
		Self { stream: Stream::new(source), phases: vec![] }
	}

	pub fn phase(&self) -> Option<Phase> {
		self.phases.last().copied()
	}

	pub(super) fn stream(&self) -> &Stream<'a> {
		&self.stream
	}

	pub fn enter_phase(&mut self, phase: Phase) {
		self.phases.push(phase);
	}

	pub fn leave_phase(&mut self, phase: Phase) {
		let top_phase = self.phases.pop();
		debug_assert_eq!(top_phase, Some(phase));
	}

	pub(super) fn stream_mut(&mut self) -> &mut Stream<'a> {
		&mut self.stream
	}

	// pub fn next<T: Parsable<'a>>(&mut self) -> Result<Option<Token<'a, T>>> {
	// 	Token::parse(self)
	// }
}

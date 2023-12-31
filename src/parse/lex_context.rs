use crate::parse::{ParseError, Stream, Token};
use crate::play::Program;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
	WithinPath,
	WithinString,
	WithinRegex,
	DollarSignEscape,
	BraceEscape,
}

#[derive(Debug)]
pub struct LexContext<'a> {
	pub(super) stream: Stream<'a>,
	phases: Vec<Phase>,
	tokens: Vec<Token>,
	env: HashMap<OsString, Option<OsString>>,
	program: &'a mut Program,
}

impl<'a> LexContext<'a> {
	pub fn new<T: AsRef<[u8]> + ?Sized + 'a>(source: &'a T, program: &'a mut Program) -> Self {
		Self {
			stream: Stream::new(source.as_ref()),
			phases: Vec::with_capacity(2), // sensible defaults
			tokens: Vec::with_capacity(2),
			env: Default::default(),
			program,
		}
	}

	pub fn get_cli(&self, pos: isize) -> Option<&OsStr> {
		assert_ne!(pos, 0, "$0 doesnt exist rn");

		let pos = if let Ok(pos) = usize::try_from(pos) {
			pos - 1
		} else {
			usize::try_from((self.program.env().cli_len() as isize) - pos).ok()?
		};
		self.program.env().get_cli(pos)
	}

	pub fn get_env<'b>(&'b mut self, name: &OsStr) -> Option<&'b OsStr> {
		let env = &mut self.env;

		if !env.contains_key(name) {
			env.insert(name.to_owned(), std::env::var_os(name));
		}

		env[name].as_deref()
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

	pub fn take_ident(&mut self) -> Result<Option<String>, ParseError> {
		match self.take_if_fn(|tkn| matches!(tkn, Token::Variable(_)))? {
			Some(Token::Variable(name)) => Ok(Some(name)),
			Some(_) => unreachable!(),
			None => Ok(None),
		}
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

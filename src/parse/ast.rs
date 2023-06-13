use crate::parse::token::{BeginPathKind, Token};
use crate::parse::{LexContext, ParseError};
use crate::{DateTime, FileSize, PathRegex};
use os_str_bytes::{OsStrBytes, OsStringBytes};
use std::ffi::{OsStr, OsString};

// pub enum AstContext
#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
	Atom(Atom),
	// Binary(Box<Ast>, Operator< Box<Ast)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block(Vec<Expression>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathOperator {
	Add,
	Subtract,
	Multiply,
	Divide,
	Modulo,
}

impl MathOperator {
	// the bool is whether it was an assignment
	fn from_token(token: &Token) -> Option<(Self, bool)> {
		match token {
			Token::Add => Some((Self::Add, false)),
			Token::AddAssign => Some((Self::Add, true)),
			Token::Subtract => Some((Self::Subtract, false)),
			Token::SubtractAssign => Some((Self::Subtract, true)),
			Token::Multiply => Some((Self::Multiply, false)),
			Token::MultiplyAssign => Some((Self::Multiply, true)),
			Token::Divide => Some((Self::Divide, false)),
			Token::DivideAssign => Some((Self::Divide, true)),
			Token::Modulo => Some((Self::Modulo, false)),
			Token::ModuloAssign => Some((Self::Modulo, true)),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicOperator {
	Equal,
	NotEqual,
	LessThan,
	LessThanOrEqual,
	GreaterThan,
	GreaterThanOrEqual,
}

impl LogicOperator {
	// the bool is whether it was an assignment
	fn from_token(token: &Token) -> Option<Self> {
		match token {
			Token::Equal => Some(Self::Equal),
			Token::NotEqual => Some(Self::NotEqual),
			Token::LessThan => Some(Self::LessThan),
			Token::LessThanOrEqual => Some(Self::LessThanOrEqual),
			Token::GreaterThan => Some(Self::GreaterThan),
			Token::GreaterThanOrEqual => Some(Self::GreaterThanOrEqual),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
	Atom(Atom),
	Math(Box<Self>, MathOperator, Box<Self>),
	Logic(Box<Self>, LogicOperator, Box<Self>),
	Assignment(OsString, Option<MathOperator>, Box<Self>),
	And(Box<Self>, Box<Self>),
	Or(Box<Self>, Box<Self>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precedence {
	Atom,
	MulDivMod,
	AddSub,
	Logic,
	ShortCircuit,
	Assignment,
}

impl Expression {
	pub fn parse(lctx: &mut LexContext, comma_is_and: bool) -> Result<Option<Self>, ParseError> {
		let Some(begin) = Atom::parse(lctx)? else {
			return Ok(None);
		};

		let Some(next) = lctx.next()? else {
			return Ok(Some(Self::Atom(begin)));
		};

		if next == Token::Assign {
			let Atom::Variable(var) = begin else {
				return Err(ParseError::AssignToNonVariable);
			};

			let Some(rhs) = Self::parse(lctx, comma_is_and)? else {
				return Err(ParseError::MissingRhsToAssignment);
			};

			return Ok(Some(Self::Assignment(var, None, rhs.into())));
		}

		if let Some((math, assign)) = MathOperator::from_token(&next) {
			let Some(rhs) = Self::parse(lctx, comma_is_and)? else {
				return Err(ParseError::MissingRhsToMathOp);
			};

			if !assign {
				return Ok(Some(Self::Math(Self::Atom(begin).into(), math, rhs.into())));
			}

			let Atom::Variable(var) = begin else {
				return Err(ParseError::AssignToNonVariable);
			};

			return Ok(Some(Self::Assignment(var, Some(math), rhs.into())));
		}

		if let Some(logic) = LogicOperator::from_token(&next) {
			let Some(rhs) = Self::parse(lctx, comma_is_and)? else {
				return Err(ParseError::MissingRhsToLogicOp);
			};

			return Ok(Some(Self::Logic(Self::Atom(begin).into(), logic, rhs.into())));
		}

		if next == Token::And || next == Token::Comma && comma_is_and {
			let Some(rhs) = Self::parse(lctx, comma_is_and)? else {
				return Err(ParseError::MissingRhsToLogicOp);
			};

			return Ok(Some(Self::And(Self::Atom(begin).into(), rhs.into())));
		}

		if next == Token::Or {
			let Some(rhs) = Self::parse(lctx, comma_is_and)? else {
				return Err(ParseError::MissingRhsToLogicOp);
			};

			return Ok(Some(Self::Or(Self::Atom(begin).into(), rhs.into())));
		}

		lctx.push_token(next);

		Ok(Some(Self::Atom(begin)))
	}

	pub fn parse_until(lctx: &mut LexContext, until: Token) -> Result<Self, ParseError> {
		todo!();
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpolated {
	parts: Vec<(Vec<u8>, Box<Expression>)>,
	tail: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegexFlags;

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
	Not(Box<Self>),
	Negate(Box<Self>),
	Block(Block),

	InterpolatedPath(BeginPathKind, Interpolated),
	Path(PathRegex),

	InterpolatedString(Interpolated),
	String(OsString),

	InterpolatedRegex(Interpolated, RegexFlags),
	Regex(OsString, RegexFlags), // todo: actual regex

	Variable(OsString),
	Number(f64),
	DateTime(DateTime),
	FileSize(FileSize),

	FnCall(Box<Self>, Vec<Expression>), // note that only variables and blocks are the first arg.
}

impl Interpolated {
	pub fn parse_until(lctx: &mut LexContext, end: Token) -> Result<Self, ParseError> {
		let mut parts = Vec::new();
		let mut current = Vec::new();

		loop {
			match lctx.next()?.expect("this should be an error in the lexer") {
				Token::CliArg(pos) => {
					let cli = lctx.get_cli(pos).ok_or(ParseError::InvalidCliPosition(pos))?;
					current.extend(cli.to_raw_bytes().iter());
				}
				Token::EnvVar(var) => {
					let cli = lctx.get_env(&var).ok_or(ParseError::MissingEnvVar(var))?;
					current.extend(cli.to_raw_bytes().iter());
				}
				Token::Raw(data) => current.extend(&data),
				Token::BeginBraceEscape => {
					let expr = Expression::parse_until(lctx, Token::EndBraceEscape)?;
					parts.push((std::mem::take(&mut current), Box::new(expr)));
				}
				x if x == end => break,
				token => unreachable!("invalid token in interpolation: {token:?}"),
			}
		}

		Ok(Self { parts, tail: current })
	}
}

impl Block {
	pub fn parse(lctx: &mut LexContext) -> Result<Self, ParseError> {
		let mut args = Vec::new();

		while let Some(expr) = Expression::parse(lctx, true)? {
			args.push(expr);
			if !lctx.take_if(Token::Semicolon)? {
				break;
			}
		}

		if !lctx.take_if(Token::RightParen)? {
			return Err(ParseError::NoClosingRightParen);
		}

		Ok(Self(args))
	}
}

impl Atom {
	fn parse_path(begin: BeginPathKind, lctx: &mut LexContext) -> Result<Self, ParseError> {
		let interpolated = Interpolated::parse_until(lctx, Token::EndPath)?;

		if interpolated.parts.is_empty() {
			let osstr = OsStr::assert_from_raw_bytes(&interpolated.tail);
			PathRegex::parse(begin, &osstr).map(Self::Path).map_err(ParseError::BadPath)
		} else {
			Ok(Self::InterpolatedPath(begin, interpolated))
		}
	}

	fn parse_string(lctx: &mut LexContext) -> Result<Self, ParseError> {
		let interpolated = Interpolated::parse_until(lctx, Token::EndString)?;

		if interpolated.parts.is_empty() {
			Ok(Self::String(OsString::assert_from_raw_vec(interpolated.tail)))
		} else {
			Ok(Self::InterpolatedString(interpolated))
		}
	}

	fn parse_regex(lctx: &mut LexContext) -> Result<Self, ParseError> {
		let interpolated = Interpolated::parse_until(lctx, Token::EndRegex)?;

		let _ = interpolated;
		todo!();
		// if interpolated.parts.is_empty() {
		// 	Ok(Self::String(OsString::assert_from_raw_vec(interpolated.tail)))
		// } else {
		// 	Ok(Self::InterpolatedString(interpolated))
		// }
	}

	fn parse_fncall_if_given(self, lctx: &mut LexContext) -> Result<Self, ParseError> {
		if !lctx.take_if(Token::LeftParen)? {
			return Ok(self);
		}
		let mut args = Vec::new();

		while let Some(expr) = Expression::parse(lctx, false)? {
			args.push(expr);
			if !lctx.take_if(Token::Comma)? {
				break;
			}
		}

		if !lctx.take_if(Token::RightParen)? {
			return Err(ParseError::NoClosingRightParen);
		}

		Ok(Self::FnCall(Box::new(self), args))
	}

	fn parse(lctx: &mut LexContext) -> Result<Option<Self>, ParseError> {
		match lctx.next()? {
			Some(Token::BeginPath(begin)) => Ok(Some(Self::parse_path(begin, lctx)?)),
			Some(Token::BeginString) => Ok(Some(Self::parse_string(lctx)?)),
			Some(Token::BeginRegex) => Ok(Some(Self::parse_regex(lctx)?)),

			Some(Token::Not) => Ok(Some(Self::Not(Box::new(
				Self::parse(lctx)?.ok_or(ParseError::NotAndEndOfExpression)?,
			)))),

			Some(Token::Subtract) => Ok(Some(Self::Negate(Box::new(
				Self::parse(lctx)?.ok_or(ParseError::NotAndEndOfExpression)?,
			)))),

			Some(Token::LeftParen) => {
				Ok(Some(Self::Block(Block::parse(lctx)?).parse_fncall_if_given(lctx)?))
			}
			Some(Token::Variable(var)) => Ok(Some(Self::Variable(var).parse_fncall_if_given(lctx)?)),
			Some(Token::Number(num)) => Ok(Some(Self::Number(num))),
			Some(Token::FileSize(num)) => Ok(Some(Self::FileSize(num))),
			Some(Token::DateTime(num)) => Ok(Some(Self::DateTime(num))),
			_ => Ok(None),
		}
	}
}

impl Ast {
	pub fn parse(lctx: &mut LexContext) -> Result<Option<Self>, ParseError> {
		let Some(start) = lctx.next()? else {
			return Ok(None);
		};

		match start {
			Token::BeginPath(begin) => Ok(Some(Self::Atom(Atom::parse_path(begin, lctx)?))),
			Token::BeginString => Ok(Some(Self::Atom(Atom::parse_string(lctx)?))),
			// Token::
			_ => todo!(),
		}
		// match Token (
		// 	)
	}
}

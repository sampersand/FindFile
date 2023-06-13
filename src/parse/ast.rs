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
	fn precedence(self) -> Precedence {
		match self {
			Self::Add | Self::Subtract => Precedence::AddSub,
			Self::Multiply | Self::Divide | Self::Modulo => Precedence::MulDivMod,
		}
	}
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
	Math(MathOperator, Box<Self>, Box<Self>),
	Logic(LogicOperator, Box<Self>, Box<Self>),
	Assignment(OsString, Option<MathOperator>, Box<Self>),
	And(Box<Self>, Box<Self>),
	Or(Box<Self>, Box<Self>),
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
	MulDivMod,
	AddSub,
	Logic,
	ShortCircuit,
	Assignment,
	#[default]
	Any,
}

fn precedence(token: &Token, comma_is_and: bool) -> Option<Precedence> {
	match token {
		Token::Add | Token::Subtract => Some(Precedence::AddSub),
		Token::Multiply | Token::Divide | Token::Modulo => Some(Precedence::MulDivMod),

		Token::AddAssign
		| Token::SubtractAssign
		| Token::MultiplyAssign
		| Token::DivideAssign
		| Token::ModuloAssign
		| Token::Assign => Some(Precedence::Assignment),

		Token::Equal
		| Token::NotEqual
		| Token::LessThan
		| Token::LessThanOrEqual
		| Token::GreaterThan
		| Token::GreaterThanOrEqual => Some(Precedence::Logic),

		Token::And | Token::Or => Some(Precedence::Logic),
		Token::Comma if comma_is_and => Some(Precedence::Logic),
		_ => None,
	}
}

impl Expression {
	pub fn parse(
		lctx: &mut LexContext,
		comma_is_and: bool,
		prec: Precedence,
	) -> Result<Option<Self>, ParseError> {
		let Some(begin) = Atom::parse(lctx)? else {
			return Ok(None);
		};
		let mut lhs = Self::Atom(begin);

		while let Some(token) = lctx.next()? {
			let token_prec = match precedence(&token, comma_is_and) {
				Some(p) if p <= prec => p,
				_ => {
					lctx.push_token(token);
					break;
				}
			};

			let Some(rhs) = Self::parse(lctx, comma_is_and, token_prec)? else {
				return Err(ParseError::MissingRhsToOp);
			};

			if token == Token::Assign {
				let Self::Atom(Atom::Variable(var)) = lhs else {
					return Err(ParseError::AssignToNonVariable);
				};

				lhs = Self::Assignment(var, None, rhs.into());
				continue;
			};

			if let Some((math, assign)) = MathOperator::from_token(&token) {
				if assign {
					let Self::Atom(Atom::Variable(var)) = lhs else {
						return Err(ParseError::AssignToNonVariable);
					};

					lhs = Self::Assignment(var, None, rhs.into());
				} else {
					lhs = Self::Math(math, lhs.into(), rhs.into());
				}
				continue;
			}

			if let Some(logic) = LogicOperator::from_token(&token) {
				lhs = Self::Logic(logic, lhs.into(), rhs.into());
				continue;
			}

			if token == Token::And || token == Token::Comma && comma_is_and {
				lhs = Self::And(lhs.into(), rhs.into());
				continue;
			}
			if token == Token::Or {
				lhs = Self::Or(lhs.into(), rhs.into());
				continue;
			}

			lctx.push_token(token);
			break;
		}

		Ok(Some(lhs))
	}

	pub fn parse_until(lctx: &mut LexContext, until: Token) -> Result<Self, ParseError> {
		todo!();
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpolated {
	parts: Vec<(Vec<u8>, Block)>,
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
					let env = lctx.get_env(&var).ok_or(ParseError::MissingEnvVar(var))?;
					current.extend(env.to_raw_bytes().iter());
				}
				Token::Raw(data) => current.extend(&data),
				Token::BeginBraceEscape => {
					let expr = Block::parse_until(lctx, Token::EndBraceEscape)?;
					parts.push((std::mem::take(&mut current), expr));
				}
				x if x == end => break,
				token => unreachable!("invalid token in interpolation: {token:?}"),
			}
		}

		Ok(Self { parts, tail: current })
	}
}

impl Block {
	pub fn parse_until(lctx: &mut LexContext, end: Token) -> Result<Self, ParseError> {
		let mut args = Vec::new();

		while let Some(expr) = Expression::parse(lctx, true, Precedence::Any)? {
			args.push(expr);
			if !lctx.take_if(Token::Semicolon)? {
				break;
			}
		}

		if !lctx.take_if(end)? {
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

		while let Some(expr) = Expression::parse(lctx, false, Precedence::Any)? {
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

			Some(Token::LeftParen) => Ok(Some(
				Self::Block(Block::parse_until(lctx, Token::RightParen)?)
					.parse_fncall_if_given(lctx)?,
			)),
			Some(Token::Variable(var)) => Ok(Some(Self::Variable(var).parse_fncall_if_given(lctx)?)),
			Some(Token::Number(num)) => Ok(Some(Self::Number(num))),
			Some(Token::FileSize(num)) => Ok(Some(Self::FileSize(num))),
			Some(Token::DateTime(num)) => Ok(Some(Self::DateTime(num))),
			Some(Token::CliArg(pos)) => {
				let cli = lctx.get_cli(pos).ok_or(ParseError::InvalidCliPosition(pos))?;
				Ok(Some(Self::String(OsString::assert_from_raw_vec(
					cli.to_raw_bytes().into_owned(),
				))))
			}
			Some(Token::EnvVar(var)) => {
				let env = lctx.get_env(&var).ok_or(ParseError::MissingEnvVar(var))?;
				Ok(Some(Self::String(OsString::assert_from_raw_vec(
					env.to_raw_bytes().into_owned(),
				))))
			}

			Some(other) => {
				lctx.push_token(other);
				Ok(None)
			}
			None => Ok(None),
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

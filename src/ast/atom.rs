use crate::ast::{Block, Expression, Precedence};
use crate::parse::{LexContext, ParseError, Token};
use crate::{DateTime, FileSize, PathRegex};
use os_str_bytes::{OsStrBytes, OsStringBytes};
use std::ffi::{OsStr, OsString};

mod interpolated;
pub use interpolated::Interpolated;

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
	Not(Box<Self>),
	Negate(Box<Self>),
	Block(Block),

	InterpolatedPath(crate::parse::token::BeginPathKind, Interpolated),
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

#[derive(Debug, Clone, PartialEq)]
pub struct RegexFlags;

impl Atom {
	fn parse_path(
		begin: crate::parse::token::BeginPathKind,
		lctx: &mut LexContext,
	) -> Result<Self, ParseError> {
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

	pub fn parse(lctx: &mut LexContext) -> Result<Option<Self>, ParseError> {
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
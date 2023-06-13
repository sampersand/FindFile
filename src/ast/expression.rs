use crate::ast::{Atom, LogicOperator, MathOperator, Precedence};
use crate::parse::{LexContext, ParseError, Token};
use std::ffi::OsString;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
	Atom(Atom),
	Math(MathOperator, Box<Self>, Box<Self>),
	Logic(LogicOperator, Box<Self>, Box<Self>),
	Assignment(OsString, Option<MathOperator>, Box<Self>),
	And(Box<Self>, Box<Self>),
	Or(Box<Self>, Box<Self>),
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
			let token_prec = match Precedence::of(&token, comma_is_and) {
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

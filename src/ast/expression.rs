use crate::ast::{Atom, LogicOperator, MathOperator, Precedence};
use crate::parse::{LexContext, ParseError, Token};
use crate::play::{PlayContext, PlayResult, RunContext};
use crate::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
	Atom(Atom),
	Math(MathOperator, Box<Self>, Box<Self>),
	Logic(LogicOperator, Box<Self>, Box<Self>),
	Assignment(String, Option<MathOperator>, Box<Self>),
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

// #[derive(Debug, Clone, PartialEq)]
// pub enum Expression {
// 	Atom(Atom),
// 	Math(MathOperator, Box<Self>, Box<Self>),
// 	Logic(LogicOperator, Box<Self>, Box<Self>),
// 	Assignment(String, Option<MathOperator>, Box<Self>),
// 	And(Box<Self>, Box<Self>),
// 	Or(Box<Self>, Box<Self>),
// }

impl Expression {
	pub fn run(&self, ctx: &mut PlayContext, rctx: RunContext) -> PlayResult<Value> {
		match self {
			Self::Atom(atom) => atom.run(ctx, rctx),
			Self::Math(op, lhs, rhs) => {
				op.run(&lhs.run(ctx, RunContext::Any)?, &rhs.run(ctx, RunContext::Any)?)
			}
			Self::Logic(op, lhs, rhs) => op
				.run(&lhs.run(ctx, RunContext::Logical)?, &rhs.run(ctx, RunContext::Logical)?)
				.map(Value::from),
			Self::Assignment(name, op, rhs) => {
				let value = if let Some(op) = op {
					let old = ctx.lookup_var(name);
					op.run(&old, &rhs.run(ctx, RunContext::Any)?)?
				} else {
					rhs.run(ctx, RunContext::Any)?
				};

				ctx.assign_var(name, value.clone());
				Ok(value)
			}
			Self::And(lhs, rhs) => {
				let lhs = lhs.run(ctx, RunContext::Logical)?;
				if lhs.is_truthy() {
					rhs.run(ctx, RunContext::Logical)
				} else {
					Ok(lhs)
				}
			}
			Self::Or(lhs, rhs) => {
				let lhs = lhs.run(ctx, RunContext::Logical)?;
				if lhs.is_truthy() {
					Ok(lhs)
				} else {
					rhs.run(ctx, RunContext::Logical)
				}
			}
		}
	}

	// pub fn matches(&self, ctx: &mut PlayContext) -> PlayResult<bool> {
	// 	match self {
	// 		Self::Atom(atom) => atom.matches(ctx, lctx),
	// 		Self::And(lhs, rhs) => Ok(lhs.matches(ctx, lctx)? && rhs.matches(ctx, lctx)?),
	// 		Self::Or(lhs, rhs) => Ok(lhs.matches(ctx, lctx)? || rhs.matches(ctx, lctx)?),
	// 		_ => todo!(),
	// 	}
	// }
}

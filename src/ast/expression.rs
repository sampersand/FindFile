use crate::ast::{Atom, LogicOperator, MathOperator, Precedence};
use crate::parse::{LexContext, ParseError, Token};
use crate::play::{PlayContext, PlayResult, RunContext};
use crate::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortCircuit {
	And,
	Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
	Atom(Atom),
	Math(MathOperator, Box<Self>, Box<Self>),
	Logic(LogicOperator, Box<Self>, Box<Self>),
	Assignment(String, Option<MathOperator>, Box<Self>),
	ShortCircuitAssignment(String, ShortCircuit, Box<Self>),
	ShortCircuit(ShortCircuit, Box<Self>, Box<Self>),
}

impl Expression {
	pub fn parse_toplevel(lctx: &mut LexContext) -> Result<Self, ParseError> {
		Ok(Self::parse(lctx, true, Precedence::default())?
			.expect("Todo: error for no valid expression"))
	}

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

			let rhs =
				Self::parse(lctx, comma_is_and, token_prec)?.ok_or(ParseError::MissingRhsToOp)?;

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
				lhs = Self::ShortCircuit(ShortCircuit::And, lhs.into(), rhs.into());
				continue;
			}
			if token == Token::Or {
				lhs = Self::ShortCircuit(ShortCircuit::Or, lhs.into(), rhs.into());
				continue;
			}

			lctx.push_token(token);
			break;
		}

		Ok(Some(lhs))
	}

	pub fn parse_until1(lctx: &mut LexContext, until: Token) -> Result<Self, ParseError> {
		todo!();
	}

	// im not really confident in this algorithm; in the future i'll make it more robust.
	pub fn begin_position(&self) -> Vec<PathBuf> {
		match self {
			Self::Atom(Atom::Value(Value::Path(path))) => vec![path.to_path_buf()],
			Self::Atom(Atom::Value(Value::PathGlob(pathglob))) => vec![pathglob.begin_position()],
			Self::ShortCircuit(ShortCircuit::And, lhs, rhs) => {
				let mut beginnings = lhs.begin_position();

				'out: for new in rhs.begin_position() {
					for idx in 0..beginnings.len() {
						// `a/b && a/b` -> `a/b`
						if beginnings[idx] == new {
							continue 'out;
						}

						// `a/b && a/` -> keep `a/b`
						if beginnings[idx].starts_with(&new) {
							continue 'out;
						}

						// `a/ && a/b` -> replace with `a/b`
						if new.starts_with(&beginnings[idx]) {
							beginnings[idx] = new;
							continue 'out;
						}

						// `a/b/c && a/b/d` -> replace with `a/b`
						for ancestor in beginnings[idx].ancestors() {
							if new.starts_with(&ancestor) {
								beginnings[idx] = ancestor.into();
								continue 'out;
							}
						}

						panic!("when does this happen?: {new:?} {old:?}", old = beginnings[idx]);
					}
				}
				beginnings
			}

			Self::ShortCircuit(ShortCircuit::Or, lhs, rhs) => {
				let mut beginnings = lhs.begin_position();

				'out: for new in rhs.begin_position() {
					for current in beginnings.iter_mut() {
						// we already have it, nothing to do
						if current.starts_with(&new) {
							continue 'out;
						}

						// it's more specific, let's add it in
						if new.starts_with(&current) {
							*current = new;
							continue 'out;
						}
					}
					// If nothing starts with it, let's add it to the list.
					beginnings.push(new)
				}
				beginnings
			}
			_ => vec![],
		}
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
				.run(&lhs.run(ctx, RunContext::Any)?, &rhs.run(ctx, RunContext::Any)?)
				.map(Value::from),
			Self::Assignment(name, op, rhs) => {
				let value = if let Some(op) = op {
					let old = ctx.lookup_var(name)?;
					op.run(&old, &rhs.run(ctx, RunContext::Any)?)?
				} else {
					rhs.run(ctx, RunContext::Any)?
				};

				ctx.assign_var(name, value.clone());
				Ok(value)
			}
			Self::ShortCircuitAssignment(sc, lhs, rhs) => {
				todo!()
				// let value = if let Some(op) = op {
				// 	let old = ctx.lookup_var(name);
				// 	op.run(&old, &rhs.run(ctx, RunContext::Any)?)?
				// } else {
				// 	rhs.run(ctx, RunContext::Any)?
				// };

				// ctx.assign_var(name, value.clone());
				// Ok(value)
			}

			Self::ShortCircuit(sc, lhs, rhs) => {
				let lhs = lhs.run(ctx, RunContext::Logical)?;
				if lhs.is_truthy() == (*sc == ShortCircuit::And) {
					rhs.run(ctx, RunContext::Logical)
				} else {
					Ok(lhs)
				}
			} // Self::Or(lhs, rhs) => {
			  // 	let lhs = lhs.run(ctx, RunContext::Logical)?;
			  // 	if lhs.is_truthy() {
			  // 		Ok(lhs)
			  // 	} else {
			  // 		rhs.run(ctx, RunContext::Logical)
			  // 	}
			  // }
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

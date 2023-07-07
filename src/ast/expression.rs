use crate::ast::{Atom, Block, LogicOperator, MathOperator, Precedence};
use crate::parse::{LexContext, ParseError, Token};
use crate::play::{PlayContext, PlayResult};
use crate::vm::block::{BuildContext, Builder};
use crate::vm::Opcode;
use crate::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortCircuit {
	And,
	Or,
	Defined,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
	Atom(Atom),
	Math(MathOperator, Box<Self>, Box<Self>),
	Logic(LogicOperator, Box<Self>, Box<Self>),
	Assignment(String, Option<MathOperator>, Box<Self>),
	ShortCircuitAssignment(String, ShortCircuit, Box<Self>),
	ShortCircuit(ShortCircuit, Box<Self>, Box<Self>),

	If(Vec<(Self, Self)>, Option<Box<Self>>),
	While(Box<Self>, Box<Self>),
	Break,
	Continue,
	Return(Option<Box<Self>>),
	FnDecl(Option<String>, Vec<String>, Box<Self>),
}

impl Expression {
	pub fn parse_toplevel(lctx: &mut LexContext) -> Result<Self, ParseError> {
		Ok(Self::parse(lctx, true, Precedence::default())?
			.expect("Todo: error for no valid expression"))
	}

	fn parse_single(lctx: &mut LexContext, msg: &'static str) -> Result<Self, ParseError> {
		Self::parse(lctx, true, Precedence::default())?.ok_or(ParseError::Message(msg))
	}

	fn parse_statement(lctx: &mut LexContext) -> Result<Option<Self>, ParseError> {
		const STATEMENT_BEGIN: [Token; 6] =
			[Token::If, Token::While, Token::Continue, Token::Break, Token::Return, Token::Fn];

		let Some(token) = lctx.take_if_fn(|x| STATEMENT_BEGIN.contains(&x))? else {
			return Ok(None);
		};

		match token {
			Token::If => {
				let cond = Self::parse_single(lctx, "missing condition for if")?;
				if !lctx.take_if(Token::Do)? {
					return Err(ParseError::Message("expected `do` after `if` condition"));
				}

				let body = Self::parse_single(lctx, "missing body for if")?;

				let mut conds = vec![(cond, body)];
				while lctx.take_if(Token::Elif)? {
					let elif_cond = Self::parse_single(lctx, "missing condition for elif")?;
					if !lctx.take_if(Token::Do)? {
						return Err(ParseError::Message("expected `do` after `elif` condition"));
					}

					conds.push((elif_cond, Self::parse_single(lctx, "missing body for elif")?));
				}

				let else_body = if lctx.take_if(Token::Else)? {
					Some(Self::parse_single(lctx, "missing body for else")?)
				} else {
					None
				};

				Ok(Some(Self::If(conds, else_body.map(From::from))))
			}

			Token::While => {
				let cond = Self::parse_single(lctx, "missing condition for while")?;
				if !lctx.take_if(Token::Do)? {
					return Err(ParseError::Message("expected `do` after `while` condition"));
				}

				let body = Self::parse_single(lctx, "missing body for while")?;

				Ok(Some(Self::While(cond.into(), body.into())))
			}

			Token::Continue => Ok(Some(Self::Continue)),
			Token::Break => Ok(Some(Self::Break)),
			Token::Return => Ok(Some(Self::Return(
				Self::parse(lctx, true, Precedence::default())?.map(From::from),
			))),

			Token::Fn => {
				let name = lctx.take_ident()?;

				if !lctx.take_if(Token::LeftParen)? {
					return Err(ParseError::Message("expected `(` after fn name"));
				}

				let mut args = Vec::new();
				while !lctx.take_if(Token::RightParen)? {
					let ident =
						lctx.take_ident()?.ok_or(ParseError::Message("expected variable name"))?;
					args.push(ident);
					if !lctx.take_if(Token::Comma)? {
						if !lctx.take_if(Token::RightParen)? {
							return Err(ParseError::Message("expected `,` or `)` after variable"));
						}
						break;
					}
				}

				let body = Self::parse_single(lctx, "missing body for fn")?;

				Ok(Some(Self::FnDecl(name, args, body.into())))
			}
			_ => unreachable!(),
		}
	}

	pub fn parse(
		lctx: &mut LexContext,
		comma_is_and: bool,
		prec: Precedence,
	) -> Result<Option<Self>, ParseError> {
		if let Some(statement) = Self::parse_statement(lctx)? {
			return Ok(Some(statement));
		}

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

					lhs = Self::Assignment(var, Some(math), rhs.into());
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

impl Expression {
	pub fn compile(self, builder: &mut Builder, ctx: BuildContext) -> Result<(), ParseError> {
		match self {
			Self::Atom(atom) => atom.compile(builder, ctx),
			Self::Math(mop, lhs, rhs) => {
				lhs.compile(builder, BuildContext::Normal)?;
				rhs.compile(builder, BuildContext::Normal)?;
				mop.compile(builder);
				Ok(())
			}
			Self::Logic(lop, lhs, rhs) => {
				lhs.compile(builder, BuildContext::Normal)?;
				rhs.compile(builder, BuildContext::Normal)?;
				lop.compile(builder);
				Ok(())
			}
			Self::Assignment(name, None, value) => {
				// `foo = "a"` should not use `a` as logical at toplevel.
				value.compile(
					builder,
					if ctx == BuildContext::Logical { ctx } else { BuildContext::Normal },
				)?;
				builder.opcode(Opcode::Dup); // it returns a value
				builder.store_variable(&name);
				Ok(())
			}
			Self::Assignment(name, Some(mop), value) => {
				builder.load_variable(&name);
				value.compile(builder, BuildContext::Normal)?; // All math assignments are normal context.
				mop.compile(builder);
				builder.opcode(Opcode::Dup); // it returns a value
				builder.store_variable(&name);
				Ok(())
			}
			Self::ShortCircuitAssignment(name, cond, value) => {
				builder.load_variable(&name);
				let end_jump = builder.defer_jump();
				value.compile(builder, BuildContext::Normal)?;
				// note: this is different from short circuit itself!! this allows `a //= "b"`.
				builder.opcode(Opcode::Dup); // it returns a value
				builder.store_variable(&name);
				match cond {
					ShortCircuit::Or => end_jump.jump_if(builder),
					ShortCircuit::And => end_jump.jump_unless(builder),
					ShortCircuit::Defined => todo!(),
				}
				Ok(())
			}

			Self::ShortCircuit(cond, lhs, rhs) => {
				lhs.compile(builder, BuildContext::Logical)?;
				builder.opcode(Opcode::Dup);
				let end_jump = builder.defer_jump();
				builder.opcode(Opcode::Pop);
				rhs.compile(builder, BuildContext::Logical)?;
				match cond {
					ShortCircuit::Or => end_jump.jump_if(builder),
					ShortCircuit::And => end_jump.jump_unless(builder),
					ShortCircuit::Defined => todo!(),
				}
				Ok(())
			}

			Self::If(conds, else_body) => {
				let mut deferred_jumps = Vec::with_capacity(conds.len());
				let mut to_next: Option<crate::vm::block::JumpIndex> = None;

				for (cond, body) in conds {
					if let Some(prev) = to_next {
						prev.jump_unless(builder);
					}

					cond.compile(builder, BuildContext::Logical)?;
					to_next = Some(builder.defer_jump());
					body.compile(builder, ctx)?;
					deferred_jumps.push(builder.defer_jump());
				}

				to_next.unwrap().jump_unless(builder);

				if let Some(elsebody) = else_body {
					let jump_to_end = builder.defer_jump();
					elsebody.compile(builder, ctx)?;
					jump_to_end.jump_unconditional(builder);
				}

				for deferred_jump in deferred_jumps {
					deferred_jump.jump_unconditional(builder);
				}

				Ok(())
			}

			Self::While(cond, body) => {
				let token = builder.enter_loop();
				let start = builder.position();

				cond.compile(builder, BuildContext::Logical)?;
				let jump_to_end = builder.defer_jump();
				// if we use the return value in the future, then we can forward `ctx`.
				body.compile(builder, BuildContext::Normal)?;
				builder.jump_unconditional(start);
				jump_to_end.jump_unless(builder);

				builder.exit_loop(token);
				Ok(())
			}

			Self::Break => builder.jump_to_loop_end(),
			Self::Continue => builder.jump_to_loop_start(),
			Self::Return(result) => {
				if let Some(result) = result {
					result.compile(builder, BuildContext::Normal)?;
				} else {
					builder.load_constant(Value::default());
				}

				builder.opcode(Opcode::Return);
				Ok(())
			}
			Self::FnDecl(name, args, body) => todo!(),
		}
	}
}

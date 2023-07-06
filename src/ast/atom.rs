use crate::ast::{Block, Expression, Precedence};
use crate::parse::{LexContext, ParseError, Token};
use crate::play::{PlayContext, PlayResult, RunContextOld};
use crate::vm::{block::BuildContext, Builder, Opcode};
use crate::Regex;
use crate::{DateTime, FileSize, PathGlob, Value};
use os_str_bytes::{OsStrBytes, OsStringBytes};
use std::ffi::{OsStr, OsString};

mod interpolated;
pub use interpolated::Interpolated;

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
	Not(Box<Self>),
	Negate(Box<Self>),
	UPositive(Box<Self>),
	Block(Block),
	ForcedLogical(Box<Self>),

	InterpolatedPath(Interpolated),
	InterpolatedString(Interpolated),
	InterpolatedRegex(Interpolated, RegexFlags),

	Value(Value),
	Variable(String),

	FnCall(Box<Self>, Vec<Expression>), // note that only variables and blocks are the first arg.
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegexFlags;

impl Atom {
	fn parse_path(lctx: &mut LexContext) -> Result<Self, ParseError> {
		let (mut interpolated, _) = Interpolated::parse_until(lctx, Token::EndPath)?;

		if interpolated.parts.is_empty() {
			// todo: fix this horrible hack
			if interpolated.tail.get(0) == Some(&b'+') {
				interpolated.tail[0] = b'*';
				interpolated.tail.insert(0, b'/');
				interpolated.tail.insert(0, b'*');
				interpolated.tail.insert(0, b'*');
				interpolated.tail.insert(0, b'/');
				interpolated.tail.insert(0, b'.');
			}
			PathGlob::parse(std::path::Path::new(&OsStr::assert_from_raw_bytes(&interpolated.tail)))
				.map(Value::PathGlob)
				.map(Self::Value)
				.map_err(ParseError::BadPath)
		} else {
			Ok(Self::InterpolatedPath(interpolated))
		}
	}

	fn parse_string(lctx: &mut LexContext) -> Result<Self, ParseError> {
		let (interpolated, _) = Interpolated::parse_until(lctx, Token::EndString)?;

		if interpolated.parts.is_empty() {
			Ok(Self::Value(Value::Text(interpolated.tail.into())))
		} else {
			Ok(Self::InterpolatedString(interpolated))
		}
	}

	fn parse_regex(lctx: &mut LexContext) -> Result<Self, ParseError> {
		let (interpolated, flags) =
			Interpolated::parse_until(lctx, |r: &Token| matches!(*r, Token::EndRegex(_)))?;

		let Token::EndRegex(flags) = flags else { unreachable!(); };

		if interpolated.parts.is_empty() {
			return Ok(Self::Value(Value::Regex(Regex::new(&interpolated.tail, &flags)?)));
		} else {
			todo!();
		}
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

		while let Some(expr) = Expression::parse(lctx, false, Precedence::default())? {
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
			Some(Token::BeginPath) => Ok(Some(Self::parse_path(lctx)?)),
			Some(Token::BeginString) => Ok(Some(Self::parse_string(lctx)?)),
			Some(Token::BeginRegex) => Ok(Some(Self::parse_regex(lctx)?)),

			Some(Token::Question) => Ok(Some(Self::ForcedLogical(Box::new(
				Self::parse(lctx)?.ok_or(ParseError::NotAndEndOfExpression)?,
			)))),

			Some(Token::Not) => Ok(Some(Self::Not(Box::new(
				Self::parse(lctx)?.ok_or(ParseError::NotAndEndOfExpression)?,
			)))),

			Some(Token::Subtract) => Ok(Some(Self::Negate(Box::new(
				Self::parse(lctx)?.ok_or(ParseError::NotAndEndOfExpression)?,
			)))),

			Some(Token::Add) => Ok(Some(Self::UPositive(Box::new(
				Self::parse(lctx)?.ok_or(ParseError::NotAndEndOfExpression)?,
			)))),

			Some(Token::LeftParen) => Ok(Some(
				Self::Block(Block::parse_until(lctx, Token::RightParen)?)
					.parse_fncall_if_given(lctx)?,
			)),
			Some(Token::Variable(var)) => Ok(Some(Self::Variable(var).parse_fncall_if_given(lctx)?)),
			Some(Token::Number(num)) => Ok(Some(Self::Value(Value::Number(num)))),
			Some(Token::FileSize { fs, precision }) => {
				Ok(Some(Self::Value(Value::FileSize { fs, precision })))
			}
			Some(Token::DateTime(dt)) => Ok(Some(Self::Value(todo!() /*Value::DateTime(dt)*/))),
			Some(Token::CliArg(pos)) => {
				let cli = lctx.get_cli(pos).ok_or(ParseError::InvalidCliPosition(pos))?;
				Ok(Some(Self::Value(Value::Text(cli.to_raw_bytes().into_owned().into()))))
			}
			Some(Token::EnvVar(var)) => {
				let env = lctx.get_env(&var).ok_or(ParseError::MissingEnvVar(var))?;
				Ok(Some(Self::Value(Value::Text(env.to_raw_bytes().into_owned().into()))))
			}
			Some(Token::True) => Ok(Some(Self::Value(true.into()))),
			Some(Token::False) => Ok(Some(Self::Value(false.into()))),

			Some(other) => {
				lctx.push_token(other);
				Ok(None)
			}
			None => Ok(None),
		}
	}
}

impl Atom {
	pub fn run(&self, ctx: &mut PlayContext, rctx: RunContextOld) -> PlayResult<Value> {
		match (self, rctx) {
			(Self::ForcedLogical(atom), _) => atom.run(ctx, RunContextOld::Logical),
			(Self::Not(atom), _) => {
				atom.run(ctx, RunContextOld::Logical).map(|x| (!x.is_truthy_old()).into())
			}
			(Self::Negate(atom), RunContextOld::Logical)
				if matches!(&**atom, Self::Value(Value::FileSize { .. })) =>
			{
				let Self::Value(Value::FileSize { fs, precision: _ }) = **atom else { unreachable!() };
				Ok((ctx.info().content_size() < fs).into())
			}
			(Self::UPositive(atom), RunContextOld::Logical)
				if matches!(&**atom, Self::Value(Value::FileSize { .. })) =>
			{
				let Self::Value(Value::FileSize { fs, precision: _ }) = **atom else { unreachable!() };
				Ok((ctx.info().content_size() > fs).into())
			}
			(Self::Variable(var), _) => ctx.lookup_var(var),
			(Self::Block(block), _) => block.run(ctx, rctx),
			(Self::Value(val), _) => val.run(ctx, rctx),
			// Self::String(s) => Ok(ctx.is_file()?
			// 	&& ctx.contents()?.to_str().expect("todo").contains(s.to_str().expect("todo1"))),
			// Self::
			other => todo!("{other:?}"),
		}
	}

	// pub fn matches(&self, ctx: &mut PlayContext) -> PlayResult<bool> {
	// 	match self {
	// 		Self::String(s) => Ok({
	// 			ctx.is_file() && slice_contains(&ctx.contents()?.to_raw_bytes(), &s.to_raw_bytes())
	// 		}),
	// 		Self::Variable(var) => Ok(ctx.lookup_var(var).is_truthy_old()),
	// 		// Self::String(s) => Ok(ctx.is_file()?
	// 		// 	&& ctx.contents()?.to_str().expect("todo").contains(s.to_str().expect("todo1"))),
	// 		// Self::
	// 		other => todo!("{other:?}"),
	// 	}
	// }
}

impl Atom {
	pub fn compile(self, builder: &mut Builder, ctx: BuildContext) -> Result<(), ParseError> {
		match self {
			Self::Not(atom) => {
				atom.compile(builder, BuildContext::Logical);
				builder.opcode(Opcode::Not);
			}
			Self::Negate(atom) => {
				atom.compile(builder, BuildContext::Normal);
				builder.opcode(Opcode::Negate);
			}
			Self::UPositive(atom) => {
				atom.compile(builder, BuildContext::Normal);
				builder.opcode(Opcode::UPositive);
			}
			Self::Block(_block) => todo!(),
			Self::ForcedLogical(atom) => {
				atom.compile(builder, BuildContext::Logical);
				builder.opcode(Opcode::ForcedLogical);
			}

			Self::InterpolatedPath(interpolated) => {
				let amount = interpolated.compile(builder);
				builder.opcode(Opcode::CreatePath(amount));
			}
			Self::InterpolatedString(interpolated) => {
				let amount = interpolated.compile(builder);
				builder.opcode(Opcode::CreateString(amount));
			}
			Self::InterpolatedRegex(interpolated, _flags) => {
				let amount = interpolated.compile(builder);
				builder.opcode(Opcode::CreateRegex(amount));
				panic!("todo: flags");
			}

			Self::Value(value) => {
				builder.load_constant(value);
				if ctx != BuildContext::Normal {
					// todo: better solution than forced logical, preferably a unique opcode.
					builder.opcode(Opcode::ForcedLogical);
				}
			}

			Self::Variable(variable) => {
				// only builtin functions can be called without parens.
				if !Opcode::compile_fn_call(&variable, 0, builder) {
					builder.load_variable(&variable);
				}
			}

			Self::FnCall(func, args) => {
				if !matches!(*func, Self::Variable(_)) {
					func.clone().compile(builder, BuildContext::Normal);
				}

				let arglen = args.len();
				for arg in args {
					arg.compile(builder, BuildContext::Normal);
				}

				if let Self::Variable(name) = &*func {
					if Opcode::compile_fn_call(&name, arglen, builder) {
						return Ok(());
					}

					builder.load_variable(&name);
				}

				builder.opcode(Opcode::GenericCall(arglen));
			}
		}

		Ok(())
	}
}

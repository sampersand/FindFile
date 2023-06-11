use crate::parse::token::{BeginPathKind, Token};
use crate::parse::LexContext;
use crate::parse::ParseError;
use crate::PathRegex;
use os_str_bytes::OsStrBytes;
use std::ffi::{OsStr, OsString};

// pub enum AstContext
#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
	Atom(Atom),
	// Binary(Box<Ast>, Operator< Box<Ast)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
	Todo,
}

impl Expression {
	pub fn parse_until(lexctx: &mut LexContext, until: Token) -> Result<Self, ParseError> {
		todo!();
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterpolatedPath {
	begin: BeginPathKind,
	parts: Vec<(Vec<u8>, Box<Expression>)>,
	tail: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
	InterpolatedPath(InterpolatedPath),
	NormalPath(PathRegex),
	Variable(OsString),
	// {
	// 	// misc
	// 	Raw(Vec<u8>),
	// 	CliArg(isize),
	// 	EnvVar(OsString),
	// 	Variable(OsString),

	// 	// Begin / end pairs
	// 	BeginPath(BeginPathKind),
	// 	EndPath,
	// 	BeginString,
	// 	EndString,
	// 	BeginRegex,
	// 	EndRegex,
	// 	BeginBraceEscape, //
	// 	EndBraceEscape,   //

	// 	// Block delims
	// 	BeginBlockStart, // `^(`
	// 	EndBlockStart,   // `$(`
	// 	LeftParen,       // `(`
	// 	RightParen,      // `)`

	// 	// control characters
	// 	Question, // `?`
	// 	Colon,    // `:`
	// 	Comma,    // `,`
	// 	And,      // `&`
	// 	Or,       // `|`
	// 	Equal,    // `=`

	// 	// Math
	// 	Add,            // `+`
	// 	AddAssign,      // `+=`
	// 	Subtract,       // `-`
	// 	SubtractAssign, // `-=`
	// 	Multiply,       // `*`
	// 	MultiplyAssign, // `*=`
	// 	Divide,         // `//` or `/` followed by a space (todo: make it non-path char)
	// 	DivideAssign,   // `/=` (for `/=` the path, do `/\=`)
	// 	Modulo,         // `%`
	// 	ModuloAssign,   // `%=`

	// 	// logic
	// 	NotEqual,           // `!=`
	// 	Not,                // `!`
	// 	Assign,             // `==`
	// 	LessThanOrEqual,    // `<=`
	// 	LessThan,           // `<`
	// 	GreaterThanOrEqual, // `>=`
	// 	GreaterThan,        // `>`
	// }
}

impl Atom {
	fn parse_path(begin: BeginPathKind, lexctx: &mut LexContext) -> Result<Self, ParseError> {
		let mut parts = Vec::new();
		let mut current = Vec::new();

		loop {
			match dbg!(lexctx.next()?).expect("this should be an error in the lexer") {
				Token::CliArg(pos) => {
					let cli = lexctx.get_cli(pos).ok_or(ParseError::InvalidCliPosition(pos))?;
					current.extend(cli.to_raw_bytes().iter());
				}
				Token::EnvVar(var) => {
					let cli = lexctx.get_env(&var).ok_or(ParseError::MissingEnvVar(var))?;
					current.extend(cli.to_raw_bytes().iter());
				}
				Token::Raw(data) => current.extend(&data),
				Token::BeginBraceEscape => {
					let expr = Expression::parse_until(lexctx, Token::EndBraceEscape)?;
					parts.push((std::mem::take(&mut current), Box::new(expr)));
				}
				Token::EndPath => break,
				token => unreachable!("invalid token in path literal: {token:?}"),
			}
		}

		if parts.is_empty() {
			let osstr = OsStr::assert_from_raw_bytes(&current);
			PathRegex::parse(begin, &osstr).map(Self::NormalPath).map_err(ParseError::BadPath)
		} else {
			Ok(Self::InterpolatedPath(InterpolatedPath { begin, tail: current, parts }))
		}
	}
}

impl Ast {
	pub fn parse(lexctx: &mut LexContext) -> Result<Option<Self>, ParseError> {
		let Some(start) = lexctx.next()? else {
			return Ok(None);
		};

		match start {
			Token::BeginPath(begin) => Atom::parse_path(begin, lexctx).map(Self::Atom).map(Some),
			Token::BeginString => todo!(),
			_ => todo!(),
		}
		// match Token (
		// 	)
	}
}

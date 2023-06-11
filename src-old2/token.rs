use crate::context::Context;
use crate::filesize::{FileSize, Suffix};
use crate::pathregex::PathRegex;
use crate::stream::{Phase, Stream};
use crate::Regex;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
	/*#[token("^{")] */ BeginBlockStart,
	/*#[token("${")] */ EndBlockStart,
	/*#[token("}")] */ BlockEnd,
	PathStart(PathBuf),
	PathEnd,
	HomeDirPath,
	Raw(Vec<u8>),

	/*#[token("?")] */ Question,
	/*#[token(":")] */ Colon,
	/*#[token("(")] */ LeftParen,
	/*#[token(")")] */ RightParen,
	/*#[token(",")] */ Comma,
	/*#[token("&")] */ And,
	/*#[token("|")] */ Or,

	/*#[token("+")] */ Plus,
	/*#[token("-")] */ Minus,
	/*#[token("*")] */ Asterisk,
	/*#[token("//")] */ Divide,
	/*#[token("^")] */ Pow,

	/*#[token("!")] */ Not,
	/*#[token("!")] */ NotEqual,
	/*#[token(":=")] */ Assign,
	/*#[regex("==?")] */ Equal,
	/*#[token("<")] */ LessThan,
	/*#[token("<=")] */ LessThanOrEqual,
	/*#[token(">")] */ GreaterThan,
	/*#[token(">=")] */ GreaterThanOrEqual,

	// #[regex(r"\$(?&ident)", |lex| OsStr::new(&lex.slice()[1..]))]
	EnvVar(OsString),

	// #[regex(r"\$-?(?&digits)", |lex| lex.slice()[1..].parse())]
	CliArg(isize),

	// #[regex(r"(?&ident)")]
	Identifier(Vec<u8>),

	// #[regex(r"(?i)(?&digits)[kmgtpe]i?b?", |lex| lex.slice().parse::<FileSize>().expect("it should always work"))]
	FileSize(FileSize),

	// TODO: strip `\`s from path
	Path(PathRegex),

	// #[regex(r"(?i)(?&digits)[kmgtpe]i?b?", |lex| lex.slice().parse::<FileSize>().expect("it should always work"))]
	// DateTime(FileSize),

	// #[regex(r"(?&digits)(.?(&digits))?([eE][-+]?(?&digits))", |lex| lex.slice().parse())]
	Number(f64),

	// #[regex(r#""(\\.|[^"])*""#, |lex| lex.slice())]
	String(String),

	// #[regex(r"\$/(\\.|[^/])*/", |lex| lex.slice().parse())]
	PerlRegex(Regex),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
	Eof,
	InvalidDollarSign,
	MissingBrace,
	InvalidEscape,
	InvalidNumber(&'static str),
}

impl Token {
	fn next_path(stream: &mut Stream) -> Option<Self> {
		match () {
			_ if stream.take_if_starts_with(b"?/") => Some(Self::PathStart("?/".into())),
			_ if stream.take_if_starts_with(b"./") => Some(Self::PathStart("./".into())),
			_ if stream.take_if_starts_with(b"../") => Some(Self::PathStart("../".into())),
			_ if stream.take_if_starts_with(b"~/") => Some(Self::PathStart("~/".into())),
			_ if stream.take_if_starts_with(b"/") => {
				if stream.peek() == Some(b'/') {
					stream.unadvance();
					return None;
				}
				Some(Self::PathStart("/".into()))
			}
			_ => None,
		}
	}

	fn next_path_body(stream: &mut Stream) -> Result<Self, ParseError> {
		let mut buf = Vec::new();

		while let Some(c) = stream.take() {
			match c {
				b'$' => {
					stream.phases.push(Phase::DollarSignEscape);
					break;
				}

				b'{' => {
					stream.phases.push(Phase::BraceEscape);
					break;
				}

				b'\\' => {
					dbg!(&stream);
					match stream.take().ok_or(ParseError::InvalidEscape)? {
						c if c.is_ascii_whitespace() || b"*?[${".contains(&c) => buf.push(c),
						b'n' => buf.push(b'\n'),
						b't' => buf.push(b'\t'),
						b'r' => buf.push(b'\r'),
						b'0' => buf.push(b'\0'),
						b'x' | b'u' | b'U' => todo!(),
						_ => return Err(ParseError::InvalidEscape)
					}
				}

				_ if matches!(c, b',' | b'(' | b')' | b';' | b'&' | b'|')
					|| c.is_ascii_whitespace() =>
				{
					stream.unadvance();
					let x = stream.phases.pop();
					debug_assert_eq!(x, Some(Phase::WithinPath));
					stream.phases.push(Phase::EndOfPath);
					break;
				}

				_ => buf.push(c),
			}
		}

		Ok(Self::Raw(buf))
	}

	fn parse_cli_arg(stream: &mut Stream, is_braced: bool) -> Result<Self, ParseError> {
		let mut buf = String::new();

		if stream.take_if_byte(b'-') {
			buf.push('-');
		} else if stream.take_if_byte(b'+') {
			// do nothing
		}

		if !parse_integer(stream, &mut buf) {
			return Err(ParseError::InvalidDollarSign);
		}

		if is_braced && !stream.take_if_byte(b'}') {
			return Err(ParseError::MissingBrace);
		}

		buf.parse::<isize>().map(Self::CliArg).or(Err(ParseError::InvalidDollarSign))
	}

	fn parse_env_var(stream: &mut Stream, is_braced: bool) -> Result<Self, ParseError> {
		let mut buf = OsString::new();

		loop {
			let Some(chr) = stream.take() else {
				if is_braced {
					return Err(ParseError::MissingBrace);
				} else {
					break;
				}
			};

			if chr == b'}' && is_braced {
				break;
			}

			if !chr.is_ascii_alphanumeric() && chr != b'_' {
				stream.unadvance();
				break;
			}

			buf.push((chr as char).to_string());
		}

		if buf.is_empty() {
			return Err(ParseError::InvalidDollarSign);
		}

		Ok(Self::EnvVar(buf))
	}

	fn parse_dollar_sign_escape(stream: &mut Stream) -> Result<Self, ParseError> {
		let is_braced = stream.take_if_byte(b'{');
		match stream.peek() {
			Some(b'-' | b'+' | b'0'..=b'9') => Self::parse_cli_arg(stream, is_braced),
			Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => Self::parse_env_var(stream, is_braced),
			_ => Err(ParseError::InvalidDollarSign),
		}
	}

	pub fn next(stream: &mut Stream, context: &Context) -> Result<Self, ParseError> {
		if stream.is_eof() {
			return Err(ParseError::Eof);
		}

		match stream.phases.pop() {
			Some(phase @ Phase::WithinPath) => {
				stream.phases.push(phase);
				Self::next_path_body(stream)
			},
			Some(Phase::EndOfPath) => Ok(Self::PathEnd),
			Some(Phase::DollarSignEscape) => Self::parse_dollar_sign_escape(stream),
			Some(Phase::BraceEscape) => todo!(),
			None => Self::next_normal(stream, context),
		}
	}

	pub fn next_normal(stream: &mut Stream, context: &Context) -> Result<Self, ParseError> {
		let _ = stream.take_while(|c| c.is_ascii_whitespace());

		if let Some(path) = Self::next_path(stream) {
			stream.phases.push(Phase::WithinPath);
			return Ok(path);
		}

		match stream.take().ok_or(ParseError::Eof)? {
			c if c.is_ascii_whitespace() => unreachable!("was already parsed"),

			// block
			b'^' if stream.take_if_byte(b'{') => Ok(Self::BeginBlockStart),
			b'$' if stream.take_if_byte(b'{') => Ok(Self::EndBlockStart),
			b'}' => Ok(Self::BlockEnd),

			b'?' => Ok(Self::Question),
			b':' if stream.take_if_byte(b'=') => Ok(Self::Assign),
			b':' => Ok(Self::Colon),
			b'(' => Ok(Self::LeftParen),
			b')' => Ok(Self::RightParen),
			b',' => Ok(Self::Comma),
			b'&' => Ok(Self::And),
			b'|' => Ok(Self::Or),

			b'+' => Ok(Self::Plus),
			b'-' => Ok(Self::Minus),
			b'*' => Ok(Self::Asterisk),
			b'/' if stream.take_if_byte(b'/') => Ok(Self::Divide),
			b'^' => Ok(Self::Pow),
			b'!' => {
				if stream.take_if_byte(b'=') {
					Ok(Self::NotEqual)
				} else {
					Ok(Self::Not)
				}
			}
			b'=' => {
				let _ = stream.take_if_byte(b'='); // `=` an d `==` are the same
				Ok(Self::Equal)
			}
			b'<' => {
				if stream.take_if_byte(b'=') {
					Ok(Self::LessThanOrEqual)
				} else {
					Ok(Self::LessThan)
				}
			}
			b'>' => {
				if stream.take_if_byte(b'=') {
					Ok(Self::GreaterThanOrEqual)
				} else {
					Ok(Self::GreaterThan)
				}
			}

			b'$' => {
				match Self::next(stream, context)? {
					Self::Identifier(ident) => {
						Ok(Self::EnvVar(String::from_utf8(ident).expect("todo").into()))
					}
					// You might be worried about casting from a float to an int. But,
					// it's ok---every shell i've looked at's maximum argument length
					// is far less than `i32::MAX` (you can get it via `getconf ARG_MAX`),
					// which is far less than `f64`'s maximum integer value it can hold before
					// we get rounding errors.
					Self::Number(number) => {
						debug_assert_eq!(number as isize as f64, number);
						Ok(Self::CliArg(number as isize))
					}
					_ => Err(ParseError::InvalidDollarSign),
				}
			}

			c if c.is_ascii_alphabetic() || c == b'_' => {
				stream.unadvance();
				let base = stream.take_while(|c| c.is_ascii_alphanumeric() || c == b'_').unwrap();
				Ok(Self::Identifier(base))
			}

			c if c.is_ascii_digit() => {
				stream.unadvance();
				let num = parse_number(stream)?;
				if let Some(suffix) = stream.take_while(|c| c.is_ascii_alphabetic()) {
					parse_suffix(num, &suffix)
				} else {
					Ok(Self::Number(num))
				}
			}

			c => todo!("oops: {:?}", c as char),
		}
	}
}

fn parse_integer(stream: &mut Stream, out: &mut String) -> bool {
	let orig_len = out.len();

	while let Some(c) = stream.take() {
		match c {
			b'_' => continue,
			b'0'..=b'9' => out.push(c as char),
			_ => {
				stream.unadvance();
				break;
			}
		}
	}

	orig_len != out.len()
}

fn parse_base(stream: &mut Stream, base: u32) -> Result<f64, ParseError> {
	let mut buf = String::new();
	while let Some(chr) = stream.take() {
		if chr == b'_' {
			continue;
		}

		if (chr as char).to_digit(base).is_some() {
			buf.push(chr as char);
		}
	}

	buf.parse().or(Err(ParseError::InvalidNumber("invalid base integer")))
}

// parses a floating point number out into `out`.
fn parse_number(stream: &mut Stream) -> Result<f64, ParseError> {
	// check to see if it has a prefix like `0x`
	if stream.take_if_byte(b'0') {
		match stream.take() {
			Some(b'x' | b'X') => return parse_base(stream, 16),
			Some(b'o' | b'O') => return parse_base(stream, 8),
			Some(b'b' | b'b') => return parse_base(stream, 2),
			_ => {
				stream.unadvance();
				stream.unadvance();
			}
		}
	}

	let mut out = String::new();
	// todo: is there a builtin to do this?
	if !parse_integer(stream, &mut out) {
		unreachable!("parse_number should always be called with a number");
	}

	dbg!(stream.peek());
	if stream.take_if_byte(b'.') {
		out.push('.');
		if !parse_integer(stream, &mut out) {
			return Err(ParseError::InvalidNumber("couldn't parse int after `.`"));
		}
	}

	if stream.take_if_byte(b'e') || stream.take_if_byte(b'E') {
		out.push('e');
		if stream.take_if_byte(b'+') {
			// do nothing
		} else if stream.take_if_byte(b'-') {
			out.push('-');
		}
		if !parse_integer(stream, &mut out) {
			return Err(ParseError::InvalidNumber("couldn't parse exponent"));
		}
	}

	out.parse().or(Err(ParseError::InvalidNumber("unable to parse float")))
}

fn parse_suffix(num: f64, suffix: &[u8]) -> Result<Token, ParseError> {
	if let Some(suffix) = Suffix::from_bytes(suffix) {
		return FileSize::from_float_and_suffix(num, suffix)
			.ok_or(ParseError::InvalidNumber("too large"))
			.map(Token::FileSize);
	}

	todo!("other suffixes");
}

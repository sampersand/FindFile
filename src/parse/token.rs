use crate::parse::{ParseContext, ParseError, Phase};
use os_str_bytes::OsStringBytes;
use std::ffi::OsString;
use std::fmt::{self, Debug, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeginPathKind {
	Root,     // `/`
	Pwd,      // `./`
	Parent,   // `../`
	Home,     // `~/`
	Anywhere, // `?/`
}

#[derive(Clone, PartialEq)]
pub enum Token {
	// Begin / end pairs
	BeginPath(BeginPathKind),
	EndPath,
	BeginString,
	EndString,
	BeginRegex,
	EndRegex,
	BeginBraceEscape, //
	EndBrace,         // also used for `BeginBlockStart` and `EndBlockStart`

	// Block delims
	EndBlockStart,
	BeginBlockStart,
	LeftParen,
	RightParen,

	Raw(Vec<u8>),
	CliArg(isize),
	EnvVar(OsString),
}

impl Debug for Token {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::BeginPath(kind) => write!(f, "Token::BeginPath({kind:?})"),
			Self::EndPath => write!(f, "Token::EndPath"),
			Self::BeginString => write!(f, "Token::BeginString"),
			Self::EndString => write!(f, "Token::EndString"),
			Self::BeginRegex => write!(f, "Token::BeginRegex"),
			Self::EndRegex => write!(f, "Token::EndRegex"),
			Self::BeginBraceEscape => write!(f, "Token::BeginBraceEscape"),
			Self::EndBrace => write!(f, "Token::EndBrace"),
			Self::Raw(raw) => {
				if let Ok(s) = std::str::from_utf8(&raw) {
					write!(f, "Token::Raw({s:?})")
				} else {
					write!(f, "Token::Raw({raw:?}")
				}
			}
			Self::CliArg(idx) => write!(f, "Token::CliArg({idx})"),
			Self::EnvVar(var) => {
				if let Some(s) = var.to_str() {
					write!(f, "Token::EnvVar({s:?})")
				} else {
					write!(f, "Token::EnvVar({var:?}")
				}
			}
			_ => todo!(),
		}
	}
}

fn append(vec: &mut Vec<u8>, chr: char) {
	// let mut buf = if_windows!([0u16; 2], [0u8; 4]);

	// (if_windows!(char::encode_utf16, char::encode_utf8))(chr, &mut buf);

	// vec.push(buf.to_String().into());
	// TODO: don't use `.to_string()`
	vec.push(u8::try_from(chr).expect("todo: actually translate it over"));
}

fn parse_escape(ctx: &mut ParseContext, is_path: bool) -> Result<char, ParseError> {
	fn parse_hex(byte: u8) -> Result<u32, ParseError> {
		(byte as char).to_digit(16).ok_or(ParseError::BadEscape("not a hex digit"))
	}

	match ctx.stream.take().ok_or(ParseError::BadEscape("nothing after backslash"))? as char {
		c @ ('\\' | '\"' | '\'') => Ok(c),
		c @ ('*' | '{' | '$' | '[') if is_path => Ok(c),
		'n' => Ok('\n'),
		't' => Ok('\t'),
		'r' => Ok('\r'),
		'0' => Ok('\0'),
		'x' => {
			let upper =
				parse_hex(ctx.stream.take().ok_or(ParseError::BadEscape("nothing after `x`"))?)?;
			let lower =
				parse_hex(ctx.stream.take().ok_or(ParseError::BadEscape("nothing after `x`"))?)?;
			char::from_u32((upper << 4) | lower).ok_or(ParseError::BadEscape("invalid `\\u` escape"))
		}
		'u' => {
			let x1 = parse_hex(ctx.stream.take().ok_or(ParseError::BadEscape("nothing after `u`"))?)?;
			let x2 = parse_hex(ctx.stream.take().ok_or(ParseError::BadEscape("nothing after `u`"))?)?;
			let x3 = parse_hex(ctx.stream.take().ok_or(ParseError::BadEscape("nothing after `u`"))?)?;
			let x4 = parse_hex(ctx.stream.take().ok_or(ParseError::BadEscape("nothing after `u`"))?)?;

			char::from_u32((x1 << 12) | (x2 << 8) | (x3 << 4) | x4)
				.ok_or(ParseError::BadEscape("invalid `\\u` escape"))
		}
		'U' => todo!(),
		other => Err(ParseError::InvalidEscape(other)),
	}
}

fn parse_integer(ctx: &mut ParseContext, into: &mut String) -> bool {
	if !ctx.stream.peek().map_or(false, |x| x.is_ascii_digit()) {
		return false;
	}

	while let Some(c) = ctx.stream.take() {
		match c {
			b'_' => continue,
			_ if c.is_ascii_digit() => into.push(c as char),
			_ => {
				ctx.stream.untake();
				break;
			}
		}
	}

	true
}

fn strip_whitespace_and_comments(ctx: &mut ParseContext) {
	loop {
		if ctx.stream.take_while(|c| c.is_ascii_whitespace()).is_some() {
			continue;
		}

		if ctx.stream.take_if_byte(b'#') {
			let _ = ctx.stream.take_while(|c| c != b'\n');
			continue;
		}

		break;
	}
}

fn is_path_literal_character(c: char) -> bool {
	todo!()
}

impl Token {
	fn parse_within_path(ctx: &mut ParseContext) -> Result<Self, ParseError> {
		let mut buf = Vec::new();

		while let Some(c) = ctx.stream.take() {
			match c {
				// `$` escapes for cli values and env vars
				b'$' => {
					ctx.push_phase(Phase::DollarSignEscape);
					break;
				}

				// `{` escapes are for interpolation
				b'{' => {
					ctx.push_phase(Phase::BraceEscape);
					break;
				}

				// `\` escapes are for special strings
				b'\\' => append(&mut buf, parse_escape(ctx, true)?),

				// Whitespace as well as `,();&|` indicate end of a path.
				// In the future, I might expand what terminates a path
				_ if b",();&|".contains(&c) || c.is_ascii_whitespace() => {
					ctx.stream.untake();
					ctx.pop_phase(Phase::WithinPath);
					ctx.push_token(Token::EndPath);
					break;
				}

				_ => append(&mut buf, c as char),
			}
		}

		Ok(Self::Raw(buf))
	}

	fn parse_cli_arg(ctx: &mut ParseContext, braced: bool) -> Result<Self, ParseError> {
		let mut buf = String::new();

		if ctx.stream.take_if_byte(b'-') {
			buf.push('-');
		} else {
			let _ = ctx.stream.take_if_byte(b'+'); // ignore leading `+`
		}

		parse_integer(ctx, &mut buf);
		if braced && !ctx.stream.take_if_byte(b'}') {
			return Err(ParseError::MissingEndingBrace);
		}

		buf.parse::<isize>().map(Self::CliArg).or(Err(ParseError::CliArgTooLarge))
	}

	fn parse_env_var(ctx: &mut ParseContext, braced: bool) -> Result<Self, ParseError> {
		let buf = ctx.stream.take_while(|c| c.is_ascii_alphanumeric() || c == b'_').unwrap();

		if braced && !ctx.stream.take_if_byte(b'}') {
			return Err(ParseError::MissingEndingBrace);
		}

		Ok(Self::EnvVar(OsString::assert_from_raw_vec(buf)))
	}

	fn parse_dollar_sign_escape(ctx: &mut ParseContext) -> Result<Self, ParseError> {
		let braced = ctx.stream.take_if_byte(b'{');

		let result = match ctx.stream.peek().expect("called parse_dollar_sign_escape at eof") {
			x if x.is_ascii_digit() || x == b'-' || x == b'+' => Self::parse_cli_arg(ctx, braced),
			x if x.is_ascii_alphabetic() || x == b'_' => Self::parse_env_var(ctx, braced),
			_ => Err(ParseError::InvalidDollarSign),
		}?;

		ctx.pop_phase(Phase::DollarSignEscape);
		Ok(result)
	}

	fn parse_within_string(ctx: &mut ParseContext) -> Result<Self, ParseError> {
		let mut buf = Vec::new();
		while let Some(c) = ctx.stream.take() {
			match c {
				// `$` escapes for cli values and env vars
				b'$' => {
					ctx.push_phase(Phase::DollarSignEscape);
					break;
				}

				// `{` escapes are for interpolation
				b'{' => {
					ctx.push_phase(Phase::BraceEscape);
					break;
				}

				// `\` is for normal escapes
				b'\\' => append(&mut buf, parse_escape(ctx, false)?),

				// `"` ends the string.
				b'"' => {
					ctx.pop_phase(Phase::WithinString);
					ctx.push_token(Token::EndString);
					break;
				}

				_ => buf.push(c),
			}
		}

		Ok(Self::Raw(buf))
	}

	pub fn parse(ctx: &mut ParseContext) -> Result<Option<Self>, ParseError> {
		if ctx.stream.is_eof() {
			return Ok(None);
		}

		match ctx.phase() {
			Some(Phase::WithinPath) => Self::parse_within_path(ctx).map(Some),
			Some(Phase::WithinString) => Self::parse_within_string(ctx).map(Some),
			Some(Phase::WithinRegex) => todo!(),
			Some(Phase::DollarSignEscape) => Self::parse_dollar_sign_escape(ctx).map(Some),
			Some(Phase::BraceEscape) => {
				match Self::parse_normal(ctx) {
					Ok(None) => Err(ParseError::MissingEndingBrace),
					Ok(Some(Self::EndBrace)) => {
						ctx.pop_phase(Phase::BraceEscape);
						debug_assert_ne!(ctx.phase(), None); // brace escape is only within another phase
						Ok(Some(Token::EndBrace))
					}
					other => other,
				}
			}
			None => Self::parse_normal(ctx),
		}
	}

	fn parse_path_prefix(ctx: &mut ParseContext) -> Result<Option<Self>, ParseError> {
		if ctx.stream.take_if_starts_with(b"./") {
			return Ok(Some(Self::BeginPath(BeginPathKind::Pwd)));
		}

		if ctx.stream.take_if_starts_with(b"../") {
			return Ok(Some(Self::BeginPath(BeginPathKind::Parent)));
		}

		if ctx.stream.take_if_starts_with(b"~/") {
			return Ok(Some(Self::BeginPath(BeginPathKind::Home)));
		}

		if ctx.stream.take_if_starts_with(b"?/") {
			return Ok(Some(Self::BeginPath(BeginPathKind::Anywhere)));
		}

		if ctx.stream.take_if_byte(b'/') {
			if ctx.stream.peek() != Some(b'/') {
				return Ok(Some(Self::BeginPath(BeginPathKind::Root)));
			}

			ctx.stream.untake(); // remove the `take_if_byte`
		}

		Ok(None)
	}

	fn parse_normal(ctx: &mut ParseContext) -> Result<Option<Self>, ParseError> {
		// remove whitespace
		strip_whitespace_and_comments(ctx);

		// check to see if it's a path literal prefix
		if let Some(path) = Self::parse_path_prefix(ctx)? {
			ctx.push_phase(Phase::WithinPath);
			return Ok(Some(path));
		}

		// Otherwise, do this
		let Some(c) = ctx.stream.take() else { return Err(ParseError::Eof); };

		match c {
			// Start of compound literals
			b'"' => {
				ctx.push_phase(Phase::WithinString);
				Ok(Some(Self::BeginString))
			}
			b'$' if ctx.stream.take_if_byte(b'/') => {
				ctx.push_phase(Phase::WithinRegex);
				Ok(Some(Self::BeginRegex))
			}

			// Block and delims
			b'$' if ctx.stream.take_if_byte(b'{') => Ok(Some(Self::EndBlockStart)),
			b'^' if ctx.stream.take_if_byte(b'{') => Ok(Some(Self::BeginBlockStart)),
			b'}' => Ok(Some(Self::EndBrace)),
			b'(' => Ok(Some(Self::LeftParen)),
			b')' => Ok(Some(Self::RightParen)),

			_ => Err(ParseError::UnknownTokenStart(c as char)),
		}
	}
}

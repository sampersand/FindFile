use crate::filesize::{FileSize, Suffix};
use crate::parse::Stream;
use crate::parse::{LexContext, ParseError, Phase};
use os_str_bytes::OsStringBytes;
use std::ffi::OsString;
use std::fmt::{self, Debug, Formatter};

/*
IDEAS:::
	How do you use a `$` variable as:
		- regex: $/$x/
		- string: just $x, or ${x} and nothing else
		- plain number: 0${x} -- we don't use octal prefix
		- size: ${x}mb
	$/{x}/ - regex literal, nothing special

	just plain `$x` -- string
	"a${x}b" - lets you embed `x`
	${x} - also string
	#{x} - integer, possible suffix
	/{x} - path (?)

	or:
	$foo -> string
	$/foo -> path (no glob)
	$*foo -> path (with glob)
	$$foo -> perl regex
	$#foo -> number
	$.foo -> file size (?)

*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeginPathKind {
	Root,     // `/`
	Pwd,      // `./`
	Parent,   // `../`
	Home,     // `~/`
	Anywhere, // `?/`
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
	// misc
	Raw(Vec<u8>),
	CliArg(isize),
	EnvVar(OsString),
	Variable(String),
	Number(f64),
	DateTime(crate::DateTime),
	FileSize(crate::FileSize),

	// Begin / end pairs
	BeginPath(BeginPathKind),
	EndPath,
	BeginString,
	EndString,
	BeginRegex,
	EndRegex(Vec<u8>),
	BeginBraceEscape, //
	EndBraceEscape,   //

	// Block delims
	BeginBlockStart, // `^(`
	EndBlockStart,   // `$(`
	LeftParen,       // `(`
	RightParen,      // `)`

	// control characters
	Question,  // `?`
	Colon,     // `:`
	Comma,     // `,`
	Semicolon, // `;`
	And,       // `&&`
	Or,        // `||`
	Assign,    // `=`

	// Math
	Add,            // `+`
	AddAssign,      // `+=`
	Subtract,       // `-`
	SubtractAssign, // `-=`
	Multiply,       // `*`
	MultiplyAssign, // `*=`
	Divide,         // `//` or `/` followed by a space (todo: make it non-path char)
	DivideAssign,   // `/=` (for `/=` the path, do `/\=`)
	Modulo,         // `%`
	ModuloAssign,   // `%=`

	// logic
	Not,                // `!`
	Matches,            // `=~`
	Equal,              // `==`
	NotEqual,           // `!=`
	LessThanOrEqual,    // `<=`
	LessThan,           // `<`
	GreaterThanOrEqual, // `>=`
	GreaterThan,        // `>`
}

fn append(vec: &mut Vec<u8>, chr: char) {
	// let mut buf = if_windows!([0u16; 2], [0u8; 4]);

	// (if_windows!(char::encode_utf16, char::encode_utf8))(chr, &mut buf);

	// vec.push(buf.to_String().into());
	// TODO: don't use `.to_string()`
	vec.push(u8::try_from(chr).expect("todo: actually translate it over"));
}

fn parse_escape(stream: &mut Stream<'_>, is_path: bool) -> Result<char, ParseError> {
	fn parse_hex(byte: u8) -> Result<u32, ParseError> {
		(byte as char).to_digit(16).ok_or(ParseError::BadEscape("not a hex digit"))
	}

	match stream.take().ok_or(ParseError::BadEscape("nothing after backslash"))? {
		c @ (b'\\' | b'\"' | b'\'' | b'$' | b'{') => Ok(c as char),
		c @ (b'*' | b'?' | b'[') if is_path => Ok(c as char),
		b'n' => Ok('\n'),
		b't' => Ok('\t'),
		b'r' => Ok('\r'),
		b'0' => Ok('\0'),
		b'x' => {
			let [x1, x2] = stream.take_n().ok_or(ParseError::BadEscape("nothing after `x`"))?;

			let hexbyte = (parse_hex(x1)? << 4) | parse_hex(x2)?;
			char::from_u32(hexbyte).ok_or(ParseError::BadEscape("invalid `\\u` escape"))
		}
		b'u' => {
			let [x1, x2, x3, x4] =
				stream.take_n().ok_or(ParseError::BadEscape("nothing after `u`"))?;

			let hexnum = (parse_hex(x1)? << 12)
				| (parse_hex(x2)? << 8)
				| (parse_hex(x3)? << 4)
				| (parse_hex(x4)? << 0);

			char::from_u32(hexnum).ok_or(ParseError::BadEscape("invalid `\\u` escape"))
		}
		b'U' => todo!(),
		other => Err(ParseError::InvalidEscape(other as char)),
	}
}

fn parse_digits(lctx: &mut LexContext, into: &mut String) -> bool {
	if !lctx.stream.peek().map_or(false, |x| x.is_ascii_digit()) {
		return false;
	}

	while let Some(c) = lctx.stream.take() {
		match c {
			b'_' => continue,
			_ if c.is_ascii_digit() => into.push(c as char),
			_ => {
				lctx.stream.untake();
				break;
			}
		}
	}

	true
}

fn parse_base_integer(lctx: &mut LexContext) -> Option<u64> {
	if !lctx.stream.advance_if(b'0') {
		return None;
	}

	let Some(basechr) = lctx.stream.take() else {
		lctx.stream.untake();
		return None;
	};

	let radix = match basechr {
		b'x' | b'X' => 16,
		b'o' | b'O' => 8,
		b'b' | b'B' => 2,
		_ => {
			lctx.stream.untake();
			lctx.stream.untake();
			return None;
		}
	};

	let mut buf = String::new();
	while let Some(c) = lctx.stream.take() {
		if (c as char).to_digit(radix).is_none() {
			break;
		}

		buf.push(c as char);
	}

	u64::from_str_radix(&buf, radix).ok()
}

fn parse_float(lctx: &mut LexContext) -> Result<(f64, Option<u8>), ParseError> {
	let mut buf = String::new();

	if lctx.stream.advance_if(b'-') {
		buf.push('-');
	} else {
		let _ = lctx.stream.advance_if(b'+'); // omit leading `+`
	}

	if !parse_digits(lctx, &mut buf) {
		return Err(ParseError::BadFloat);
	}

	let precision;

	if lctx.stream.advance_if(b'.') {
		buf.push('.');
		if !parse_digits(lctx, &mut buf) {
			return Err(ParseError::BadFloat);
		}
		if buf.bytes().last() == Some(b'0') {
			precision = Some(buf.bytes().rev().take_while(|&x| x == b'0').count() as u8);
		} else {
			precision = Some(0);
		}
	} else {
		precision = Some(0);
	}

	if lctx.stream.advance_if(b'e') || lctx.stream.advance_if(b'E') {
		buf.push('e');
		if lctx.stream.advance_if(b'-') {
			buf.push('-');
		} else {
			let _ = lctx.stream.advance_if(b'+');
		}

		if !parse_digits(lctx, &mut buf) {
			return Err(ParseError::BadFloat);
		}
	}

	let float = buf.parse().or(Err(ParseError::BadFloat))?;
	Ok((float, precision))
}

fn strip_whitespace_and_comments(lctx: &mut LexContext) {
	loop {
		if !lctx.stream.take_while(u8::is_ascii_whitespace).is_empty() {
			continue;
		}

		if lctx.stream.advance_if(b'#') {
			let _ = lctx.stream.take_while(|&c| c != b'\n');
			continue;
		}

		break;
	}
}

fn is_ascii_alphanumeric_or_underscore(c: &u8) -> bool {
	c.is_ascii_alphanumeric() || *c == b'_'
}

fn is_path_literal_character(c: char) -> bool {
	todo!()
}

impl Token {
	fn parse_within_path(lctx: &mut LexContext) -> Result<Self, ParseError> {
		let mut buf = Vec::new();

		while let Some(c) = lctx.stream.take() {
			match c {
				// `$` escapes for cli values and env vars
				b'$' => {
					lctx.push_phase(Phase::DollarSignEscape);
					break;
				}

				// `{` escapes are for interpolation
				b'{' => {
					lctx.push_phase(Phase::BraceEscape);
					lctx.push_token(Token::BeginBraceEscape);
					break;
				}

				// `\` escapes are for special strings
				b'\\' => append(&mut buf, parse_escape(&mut lctx.stream, true)?),

				// Whitespace as well as `,();&|` indicate end of a path.
				// In the future, I might expand what terminates a path
				_ if b",();&|".contains(&c) || c.is_ascii_whitespace() => {
					lctx.stream.untake();
					lctx.pop_phase(Phase::WithinPath);
					lctx.push_token(Token::EndPath);
					break;
				}

				_ => append(&mut buf, c as char),
			}
		}

		Ok(Self::Raw(buf))
	}

	fn parse_cli_arg(lctx: &mut LexContext, braced: bool) -> Result<Self, ParseError> {
		let mut buf = String::new();

		if lctx.stream.advance_if(b'-') {
			buf.push('-');
		} else {
			let _ = lctx.stream.advance_if(b'+'); // ignore leading `+`
		}

		parse_digits(lctx, &mut buf);
		if braced && !lctx.stream.advance_if(b'}') {
			return Err(ParseError::MissingEndingBrace);
		}

		buf.parse::<isize>().map(Self::CliArg).or(Err(ParseError::CliArgTooLarge))
	}

	fn parse_env_var(lctx: &mut LexContext, braced: bool) -> Result<Self, ParseError> {
		let buf = lctx.stream.take_while(is_ascii_alphanumeric_or_underscore);
		debug_assert!(!buf.is_empty());

		if braced && !lctx.stream.advance_if(b'}') {
			return Err(ParseError::MissingEndingBrace);
		}

		Ok(Self::EnvVar(OsString::assert_from_raw_vec(buf.to_owned())))
	}

	fn parse_dollar_sign_escape(lctx: &mut LexContext) -> Result<Self, ParseError> {
		let result = Self::parse_dollar_sign(lctx)?;
		lctx.pop_phase(Phase::DollarSignEscape);
		Ok(result)
	}

	fn parse_dollar_sign(lctx: &mut LexContext) -> Result<Self, ParseError> {
		let braced = lctx.stream.advance_if(b'{');

		match lctx.stream.peek().expect("called parse_dollar_sign at eof") {
			x if x.is_ascii_digit() || x == b'-' || x == b'+' => Self::parse_cli_arg(lctx, braced),
			x if x.is_ascii_alphabetic() || x == b'_' => Self::parse_env_var(lctx, braced),
			_ => Err(ParseError::InvalidDollarSign),
		}
	}

	fn parse_within_string(lctx: &mut LexContext) -> Result<Self, ParseError> {
		let mut buf = Vec::new();
		while let Some(c) = lctx.stream.take() {
			match c {
				// `$` escapes for cli values and env vars
				b'$' => {
					lctx.push_phase(Phase::DollarSignEscape);
					break;
				}

				// `{` escapes are for interpolation
				b'{' => {
					lctx.push_phase(Phase::BraceEscape);
					lctx.push_token(Token::BeginBraceEscape);
					break;
				}

				// `\` is for normal escapes
				b'\\' => append(&mut buf, parse_escape(&mut lctx.stream, false)?),

				// `"` ends the string.
				b'"' => {
					lctx.pop_phase(Phase::WithinString);
					lctx.push_token(Token::EndString);
					break;
				}

				_ => buf.push(c),
			}
		}

		Ok(Self::Raw(buf))
	}

	fn parse_within_regex(lctx: &mut LexContext) -> Result<Self, ParseError> {
		let mut buf = Vec::new();
		while let Some(c) = lctx.stream.take() {
			match c {
				// `$` escapes for cli values and env vars
				b'$' => {
					lctx.push_phase(Phase::DollarSignEscape);
					break;
				}

				// `{` escapes are for interpolation
				b'{' => {
					lctx.push_phase(Phase::BraceEscape);
					lctx.push_token(Token::BeginBraceEscape);
					break;
				}

				// `/` ends the regex, with optional syntax vars at the end.
				b'/' => {
					let flags = lctx.stream.take_while(u8::is_ascii_alphabetic);
					lctx.pop_phase(Phase::WithinRegex);
					lctx.push_token(Token::EndRegex(flags.to_owned()));
					break;
				}

				_ => buf.push(c),
			}
		}

		Ok(Self::Raw(buf))
	}

	pub fn parse(lctx: &mut LexContext) -> Result<Option<Self>, ParseError> {
		if lctx.stream.is_eof() {
			return match lctx.pop_phase_unchecked() {
				Some(Phase::WithinPath) => Ok(Some(Self::EndPath)),
				Some(Phase::WithinString) => Err(ParseError::MissingEndQuote),
				Some(Phase::WithinRegex) => Err(ParseError::MissingEndRegex),
				None => Ok(None),
				other => unimplemented!("todo: what are the conditions that this can happen?"),
			};
		}

		match lctx.phase() {
			Some(Phase::WithinPath) => Self::parse_within_path(lctx).map(Some),
			Some(Phase::WithinString) => Self::parse_within_string(lctx).map(Some),
			Some(Phase::WithinRegex) => Self::parse_within_regex(lctx).map(Some),
			Some(Phase::DollarSignEscape) => Self::parse_dollar_sign_escape(lctx).map(Some),

			Some(Phase::BraceEscape) => {
				match Self::parse_normal(lctx) {
					Ok(None) => Err(ParseError::MissingEndingBrace),
					Ok(Some(Self::EndBraceEscape)) => {
						lctx.pop_phase(Phase::BraceEscape);
						debug_assert_ne!(lctx.phase(), None); // brace escape is only within another phase
						Ok(Some(Token::EndBraceEscape))
					}
					other => other,
				}
			}
			None => Self::parse_normal(lctx),
		}
	}

	fn parse_number(lctx: &mut LexContext) -> Result<Self, ParseError> {
		let (num, precision) = if let Some(integer) = parse_base_integer(lctx) {
			(integer as f64, None)
		} else {
			parse_float(lctx)?
		};

		let suffix = lctx.stream.take_while(u8::is_ascii_alphabetic);
		if suffix.is_empty() {
			return Ok(Self::Number(num));
		};

		if let Some(suffix) = Suffix::from_bytes(suffix) {
			return Ok(Self::FileSize(
				FileSize::new(num, suffix, precision).ok_or(ParseError::FileSizeLiteralTooLarge)?,
			));
		}
		todo!()
	}

	fn parse_path_prefix(lctx: &mut LexContext) -> Result<Option<Self>, ParseError> {
		if lctx.stream.advance_if(b"./") {
			return Ok(Some(Self::BeginPath(BeginPathKind::Pwd)));
		}

		if lctx.stream.advance_if(b"../") {
			return Ok(Some(Self::BeginPath(BeginPathKind::Parent)));
		}

		if lctx.stream.advance_if(b"~/") {
			return Ok(Some(Self::BeginPath(BeginPathKind::Home)));
		}

		if lctx.stream.advance_if(b"?/") {
			return Ok(Some(Self::BeginPath(BeginPathKind::Anywhere)));
		}

		if lctx.stream.advance_if(b'/') {
			if lctx.stream.peek() != Some(b'/') {
				return Ok(Some(Self::BeginPath(BeginPathKind::Root)));
			}

			lctx.stream.untake(); // remove the `advance_if`
		}

		Ok(None)
	}

	fn parse_normal(lctx: &mut LexContext) -> Result<Option<Self>, ParseError> {
		// remove whitespace
		strip_whitespace_and_comments(lctx);

		// check to see if it's a path literal prefix
		if let Some(path) = Self::parse_path_prefix(lctx)? {
			lctx.push_phase(Phase::WithinPath);
			return Ok(Some(path));
		}

		// Otherwise, do this
		let Some(c) = lctx.stream.take() else { return Err(ParseError::Eof); };

		macro_rules! ifeq {
			($if_eq:ident, $if_not:ident) => {
				Ok(Some(if lctx.stream.advance_if(b'=') { Self::$if_eq } else { Self::$if_not }))
			};
		}

		match c {
			// Start of compound literals
			b'"' => {
				lctx.push_phase(Phase::WithinString);
				Ok(Some(Self::BeginString))
			}
			b'$' if lctx.stream.advance_if(b'/') => {
				lctx.push_phase(Phase::WithinRegex);
				Ok(Some(Self::BeginRegex))
			}

			// Block and delims
			b'$' if lctx.stream.advance_if(b'{') => Ok(Some(Self::EndBlockStart)),
			b'^' if lctx.stream.advance_if(b'{') => Ok(Some(Self::BeginBlockStart)),
			b'}' => Ok(Some(Self::EndBraceEscape)),
			b'(' => Ok(Some(Self::LeftParen)),
			b')' => Ok(Some(Self::RightParen)),

			// Control Characters
			b'?' => Ok(Some(Self::Question)), // TODO: this can conflict with `?/`
			b':' => Ok(Some(Self::Colon)),
			b',' => Ok(Some(Self::Comma)),
			b';' => Ok(Some(Self::Semicolon)),
			b'&' => Ok(Some(Self::And)),
			b'|' => Ok(Some(Self::Or)),

			// Math
			b'+' => ifeq!(AddAssign, Add),
			b'-' => ifeq!(SubtractAssign, Subtract),
			b'*' => ifeq!(MultiplyAssign, Multiply),
			b'/' if lctx.stream.advance_if(b'/') => ifeq!(DivideAssign, Divide),
			b'/' if lctx.stream.advance_if(b'=') => Ok(Some(Self::DivideAssign)),
			b'/' if lctx.stream.peek().map_or(false, |c| c.is_ascii_whitespace()) => {
				Ok(Some(Self::Divide))
			}
			b'%' => ifeq!(ModuloAssign, Modulo),

			// Logic
			b'!' => ifeq!(NotEqual, Not),
			b'=' if lctx.stream.advance_if(b'~') => Ok(Some(Self::Matches)),
			b'=' => ifeq!(Equal, Assign),
			b'<' => ifeq!(LessThanOrEqual, LessThan),
			b'>' => ifeq!(GreaterThanOrEqual, GreaterThan),

			b'$' => Self::parse_dollar_sign(lctx).map(Some),
			x if x.is_ascii_alphabetic() || c == b'_' => {
				lctx.stream.untake();
				let mut was_last_questionmark = false;
				let buf = lctx.stream.take_while(|c| {
					if was_last_questionmark {
						false
					} else if *c == b'?' {
						was_last_questionmark = true;
						true
					} else {
						is_ascii_alphanumeric_or_underscore(c)
					}
				});

				Ok(Some(Self::Variable(
					String::from_utf8(buf.to_owned()).or(Err(ParseError::VariableIsntUtf8))?,
				)))
			}
			x if x.is_ascii_digit() => {
				lctx.stream.untake();
				Self::parse_number(lctx).map(Some)
			}

			// misc
			_ => Err(ParseError::UnknownTokenStart(c as char)),
		}
	}
}

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
	BeginPath,
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
	Divide,         // `//` or `/` followed by a space (todo: make it non-path char), should be `div`.
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

fn parse_hex(byte: u8) -> Result<u32, ParseError> {
	(byte as char).to_digit(16).ok_or(ParseError::BadEscape("not a hex digit"))
}

impl<'a> Stream<'a> {
	/// Parses out an escape sequence
	fn parse_escape(&mut self, is_path: bool) -> Result<char, ParseError> {
		match self.take().ok_or(ParseError::BadEscape("nothing after backslash"))? {
			c @ (b'\\' | b'\"' | b'\'' | b'$' | b'{') => Ok(c as char),
			c @ (b'*' | b'?' | b'[') if is_path => Ok(c as char),
			b'n' => Ok('\n'),
			b't' => Ok('\t'),
			b'r' => Ok('\r'),
			b'0' => Ok('\0'),
			b'x' => {
				let [a, b] = self.take_n().ok_or(ParseError::BadEscape("nothing after `x`"))?;

				let hexbyte = (parse_hex(a)? << 4) | parse_hex(b)?;
				char::from_u32(hexbyte).ok_or(ParseError::BadEscape("invalid `\\u` escape"))
			}
			b'u' => {
				let [a, b, c, d] = self.take_n().ok_or(ParseError::BadEscape("nothing after `u`"))?;

				let hexnum = (parse_hex(a)? << 12)
					| (parse_hex(b)? << 8)
					| (parse_hex(c)? << 4)
					| (parse_hex(d)? << 0);

				char::from_u32(hexnum).ok_or(ParseError::BadEscape("invalid `\\u` escape"))
			}
			b'U' => todo!(),
			other => Err(ParseError::InvalidEscape(other as char)),
		}
	}

	/// Parses out `[0-9][0-9_]*`
	fn parse_digits(&mut self, into: &mut String) -> bool {
		// Make sure we start with just a digit, not a `_`
		if !self.peek().map_or(false, |x| x.is_ascii_digit()) {
			return false;
		}

		into.extend(
			self
				.take_while(|c| c.is_ascii_digit() || c == b'_') // Take all digits or `_`s.
				.iter()
				.filter_map(|c| c.is_ascii_digit().then_some(*c as char)), // convert to char and insert
		);

		true
	}

	// Parses out a leading `-` or `+`, if given.
	fn parse_number_sign(&mut self, buf: &mut String) {
		if let Some(sign) = self.take_if(|c| c == b'-' || c == b'+') {
			buf.push(sign as char);
		}
	}

	fn parse_base_integer(&mut self) -> Result<Option<u64>, ParseError> {
		// If it doesn't start with `0`, it's not a base integer.
		if !self.advance_if(b'0') {
			return Ok(None);
		}

		// If the `0` is the very last character in the string, then it's also not base integer; we
		// have to undo the taking of the `0`.
		let Some(basechr) = self.take() else {
			self.untake();
			return Ok(None);
		};

		// Figure out the radix based on the digit given
		let radix = match self.take_if(|c| b"xXoObB".contains(&c)) {
			// Hexadecimal literal
			Some(b'x' | b'X') => 16,

			// Octal Literal
			Some(b'o' | b'O') => 8,

			// Binary literal
			// `0b` is a byte literal, so `0b<DIGIT>` is required for binary literal
			Some(b'b' | b'B') if self.peek().map_or(false, |c| c.is_ascii_digit()) => 2,
			Some(b'b' | b'B') => {
				self.untake();
				self.untake();
				return Ok(None);
			}

			Some(_) => unreachable!(),
			None => {
				self.untake();
				return Ok(None);
			}
		};

		let buf = self
			.take_while(|c| (c as char).to_digit(radix).is_some() || c == b'_')
			.iter()
			.filter_map(|&x| (x != b'_').then_some(x as char))
			.collect::<String>();

		u64::from_str_radix(&buf, radix).map(Some).or(Err(ParseError::BadFloat))
	}

	fn parse_float(&mut self) -> Result<(f64, Option<u8>), ParseError> {
		let mut buf = String::new();

		self.parse_number_sign(&mut buf);

		if !self.parse_digits(&mut buf) {
			return Err(ParseError::BadFloat);
		}

		// Parse trailing `.###`s.
		//
		// If trailing `0`s are given after the `.###` (eg `123.345000`), then those are used as the
		// precision of the number; If no `.` is given, or it is given and no trailing `0`s are given,
		// then the precision is zero. Precision is used with `=~` to roughly match filesizes.
		let precision = if self.advance_if(b'.') {
			buf.push('.');

			// Unable to parse digits after `.` means there's a problem.
			if !self.parse_digits(&mut buf) {
				return Err(ParseError::BadFloat);
			}

			Some(buf.bytes().rev().take_while(|&x| x == b'0').count() as u8)
		} else {
			Some(0)
		};

		// Parse out the exponent, if it's given.
		if self.advance_if(|c| c == b'e' || c == b'E') {
			buf.push('e');

			self.parse_number_sign(&mut buf);
			if !self.parse_digits(&mut buf) {
				return Err(ParseError::BadFloat);
			}
		}

		let float = buf.parse().or(Err(ParseError::BadFloat))?;
		Ok((float, precision))
	}

	fn strip_whitespace_and_comments(&mut self) {
		loop {
			// If any leading whitespace is present, stirp that.
			if !self.take_while(|c| c.is_ascii_whitespace()).is_empty() {
				continue;
			}

			// If the comment symbol, `#`, is supplied, then remove until end of line.
			if self.advance_if(b'#') {
				let _ = self.take_while(|c| c != b'\n');
				continue;
			}

			// If there was neither leading whitespace not a leading comment, then we're done.
			break;
		}
	}
}

fn is_ascii_alphanumeric_or_underscore(c: u8) -> bool {
	c.is_ascii_alphanumeric() || c == b'_'
}

fn is_path_literal_character(c: u8) -> bool {
	!c.is_ascii_whitespace() && !b",();&|".contains(&c)
}

fn is_path_start(byte: u8) -> bool {
	is_ascii_alphanumeric_or_underscore(byte) || b".+*[?".contains(&byte)
}

fn is_path_end(byte: u8) -> bool {
	b",();&|".contains(&byte) || byte.is_ascii_whitespace()
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
				b'\\' => append(&mut buf, lctx.stream.parse_escape(true)?),

				// Whitespace as well as `,();&|` indicate end of a path.
				// In the future, I might expand what terminates a path
				_ if !is_path_literal_character(c) => {
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

		if !lctx.stream.parse_digits(&mut buf) {
			return Err(ParseError::CliArgMissing);
		}

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
				b'\\' => append(&mut buf, lctx.stream.parse_escape(false)?),

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
					let flags = lctx.stream.take_while(|c| c.is_ascii_alphabetic());
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
		Self::_parse(lctx) // only for `dbg!`ing and logging in the future
	}

	fn _parse(lctx: &mut LexContext) -> Result<Option<Self>, ParseError> {
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
		let (num, precision) = if let Some(integer) = lctx.stream.parse_base_integer()? {
			(integer as f64, None)
		} else {
			lctx.stream.parse_float()?
		};

		let suffix = lctx.stream.take_while(|c| c.is_ascii_alphabetic());
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

	fn _is_path_next_token(stream: &mut Stream) -> bool {
		debug_assert!(!stream.is_eof()); // shouldnt have gotten here if it was.
		let rest = stream.remainder();

		// It didn't start with a path start character, abandon early.
		if !is_path_start(rest[0]) {
			return false;
		}

		// FIXME: total hack:
		// `*` on its own is a path.
		// `*` with whitespace on either side -> not a path
		// `*` preceded by a path-character -> not a path
		if b"*.+".contains(&rest[0])
			&& rest.get(1).map_or(true, |c| !c.is_ascii_whitespace())
			&& stream
				._remainder_minus_one()
				.map_or(true, |c| !c.is_ascii_whitespace() && !is_path_start(c))
		{
			return true;
		}
		// a `*` is a part of a path if it's the first character and is not followed by `*
		// `a*b` -> not a path
		// `a(*b` -> is a path
		// 'a* 3' -> not a path
		//

		for &byte in rest {
			if (byte as char) == std::path::MAIN_SEPARATOR {
				return true;
			}

			if is_path_end(byte) {
				break;
			}
		}
		false
	}

	// this is not terrific, it doesnt do legit parsing like it should. (eg what if
	// a `[]` has `/` in it, and we add `[]` for array literals in the future?)
	// It also doesn't account for interpolation with spaces in them. So yeah,
	// redo this in the future.
	fn parse_path_glob(lctx: &mut LexContext) -> Result<Option<Self>, ParseError> {
		if !Self::_is_path_next_token(&mut lctx.stream) {
			return Ok(None);
		}
		lctx.push_phase(Phase::WithinPath);
		Ok(Some(Self::BeginPath))
	}

	// fn parse_path_prefix(lctx: &mut LexContext) -> Result<Option<Self>, ParseError> {
	// 	if lctx.stream.advance_if(b"./") {
	// 		return Ok(Some(Self::BeginPath(BeginPathKind::Pwd)));
	// 	}

	// 	if lctx.stream.advance_if(b"../") {
	// 		return Ok(Some(Self::BeginPath(BeginPathKind::Parent)));
	// 	}

	// 	if lctx.stream.advance_if(b"~/") {
	// 		return Ok(Some(Self::BeginPath(BeginPathKind::Home)));
	// 	}

	// 	if lctx.stream.advance_if(b"?/") {
	// 		return Ok(Some(Self::BeginPath(BeginPathKind::Anywhere)));
	// 	}

	// 	if lctx.stream.advance_if(b'/') {
	// 		if lctx.stream.peek() != Some(b'/') {
	// 			return Ok(Some(Self::BeginPath(BeginPathKind::Root)));
	// 		}

	// 		lctx.stream.untake(); // remove the `advance_if`
	// 	}

	// 	Ok(None)
	// }

	fn parse_normal(lctx: &mut LexContext) -> Result<Option<Self>, ParseError> {
		// remove whitespace
		lctx.stream.strip_whitespace_and_comments();

		if lctx.stream.is_eof() {
			return Err(ParseError::Eof);
		}

		if let Some(pathglob) = Self::parse_path_glob(lctx)? {
			return Ok(Some(pathglob));
		}

		// // check to see if it's a path literal prefix
		// if let Some(path) = Self::parse_path_prefix(lctx)? {
		// 	lctx.push_phase(Phase::WithinPath);
		// 	return Ok(Some(path));
		// }

		// Otherwise, do this
		let c = lctx.stream.take().ok_or(ParseError::Eof)?;

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

			b'@' => todo!("parse `@` strings (like `%` strings in ruby)"),

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
					} else if c == b'?' {
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

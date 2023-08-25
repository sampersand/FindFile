use crate::parse2::parser::Phase;
use crate::parse2::{ErrorKind, Parser, Result, Stream};

// we could use the whole "span" scheme, but that seems overkill, as all we're doing
pub enum Token<'a> {
	// Compound Tokens
	RawData(&'a str),
	StartString,
	EndString,
	StartRegex,
	EndRegex(&'a str), // the value is the flags.
	StartPath,
	EndPath,
	StartInterpolation,
	EndInterpolation,

	// Simpler Tokens
	Identifier(&'a str),

	// Symbols
	Comma,        // `,`
	Semicolon,    // `;`
	LeftParen,    // `(`
	RightParen,   // `)`
	LeftBrace,    // `{`
	RightBrace,   // `}`
	LeftBracket,  // `[`
	RightBracket, // `]`
	Assign,       // `=`
	Question,     // `?`
	Colon,        // `:`

	// Control flow Keywords
	If,       // `if`
	Do,       // `do`
	Else,     // `else`
	Elif,     // `elif`
	While,    // `while`
	For,      // `for`
	Break,    // `break`
	Continue, // `continue`
	Return,   // `return`
	Fn,       // `fn`
	True,     // `true`
	False,    // `false`

	// Logic keywords
	Not,                // `!`
	Matches,            // `=~`
	NotMatches,         // `!~`
	Equal,              // `==`
	NotEqual,           // `!=`
	LessThanOrEqual,    // `<=`
	LessThan,           // `<`
	GreaterThanOrEqual, // `>=`
	GreaterThan,        // `>`

	// Short-circuit keywords.
	Undef,        // `//`
	UndefAssign,  // `//=`
	OrOr,         // `||`
	OrOrAssign,   // `||=`
	AndAnd,       // `&&`
	AndAndAssign, // `&&=`

	// Math keywords
	Add,      // `+`
	Subtract, // `-`
	Multiply, // `*`
	Divide,   // `/` (surrounded by non-token chars on each side)
	Modulo,   // `%`
	Pow,      // `^`

	// Math Assignment keywords
	Increment,      // `++`
	Decrement,      // `--`
	AddAssign,      // `+=`
	SubtractAssign, // `-=`
	MultiplyAssign, // `*=`
	DivideAssign,   // `/=` (surrounded by non-token chars on each side)
	ModuloAssign,   // `%=`
	PowAssign,      // `^=`
}

impl<'a> Token<'a> {
	pub fn parse_binary(parser: &mut Parser<'a>) -> Result<Option<Self>> {
		let mut stream = parser.stream_mut();
		Self::strip_whitespace_and_comments(stream);

		macro_rules! ifeq {
			($ifeq:ident, $ifneq:ident) => {
				Ok(Some(if stream.take_if('=').is_some() { Self::$ifeq } else { Self::$ifneq }))
			};
		}

		match stream.take() {
			// Matching
			Some('!') if stream.take_if('~').is_some() => Ok(Some(Self::NotMatches)),
			Some('=') if stream.take_if('~').is_some() => Ok(Some(Self::Matches)),

			// Increment/Decrement
			Some('+') if stream.take_if('+').is_some() => Ok(Some(Self::Increment)),
			Some('-') if stream.take_if('-').is_some() => Ok(Some(Self::Decrement)),

			// Short circuit assignment
			Some('/') if stream.take_if('/').is_some() => ifeq!(UndefAssign, Undef),
			Some('&') if stream.take_if('&').is_some() => ifeq!(AndAndAssign, AndAnd),
			Some('|') if stream.take_if('|').is_some() => ifeq!(OrOrAssign, OrOr),

			// Logic
			Some('!') => ifeq!(NotEqual, Not),
			Some('=') => ifeq!(Equal, Assign),
			Some('<') => ifeq!(LessThanOrEqual, LessThan),
			Some('>') => ifeq!(GreaterThanOrEqual, GreaterThan),

			// Math
			Some('+') => ifeq!(AddAssign, Add),
			Some('-') => ifeq!(SubtractAssign, Subtract),
			Some('*') => ifeq!(MultiplyAssign, Multiply),
			Some('/') => ifeq!(DivideAssign, Divide),
			Some('%') => ifeq!(ModuloAssign, Modulo),
			Some('^') => ifeq!(PowAssign, Pow),

			None => Ok(None),
			Some(other) => {
				stream.untake(other);
				Ok(None)
			}
		}
	}

	pub fn parse_non_binary(parser: &mut Parser<'a>) -> Result<Option<Self>> {
		match parser.phase() {
			Some(Phase::Text) => Self::parse_phase_text(parser).map(Some),
			Some(Phase::Regex) => Self::parse_phase_regex(parser).map(Some),
			Some(Phase::Path) => Self::parse_phase_path(parser),
			None => Self::parse_no_phase(parser),
		}
	}

	fn parse_phase_until(stream: &mut Stream<'a>, end: char) -> Option<Self> {
		// We only stop parsing when we hit a `{` or an `end`, without `\`s in front.
		let mut last_was_backslash = false;
		stream
			.take_while(|c| {
				if last_was_backslash {
					last_was_backslash = false;
					true
				} else if c == '\\' {
					last_was_backslash = true;
					true
				} else {
					c != end && c != '{'
				}
			})
			.map(Self::RawData)
	}

	pub fn parse_phase_text(parser: &mut Parser<'a>) -> Result<Self> {
		debug_assert_eq!(parser.phase(), Some(Phase::Text));
		if let Some(raw) = Self::parse_phase_until(parser.stream_mut(), '"') {
			return Ok(raw);
		}

		match parser.stream_mut().take() {
			Some('"') => {
				parser.leave_phase(Phase::Text);
				Ok(Self::EndString)
			}
			Some('{') => Ok(Self::StartInterpolation),
			Some(_) => unreachable!(),
			None => Err(parser.stream_mut().error(ErrorKind::UnterminatedString)),
		}
	}

	pub fn parse_phase_regex(parser: &mut Parser<'a>) -> Result<Self> {
		debug_assert_eq!(parser.phase(), Some(Phase::Regex));

		if let Some(raw) = Self::parse_phase_until(parser.stream_mut(), '"') {
			return Ok(raw);
		}

		match parser.stream_mut().take() {
			Some('/') => {
				parser.leave_phase(Phase::Regex);
				Ok(Self::EndRegex(
					parser.stream_mut().take_while(char::is_alphanumeric).unwrap_or_default(),
				))
			}
			Some('{') => Ok(Self::StartInterpolation),
			Some(_) => unreachable!(),
			None => Err(parser.stream_mut().error(ErrorKind::UnterminatedRegex)),
		}
	}

	pub fn parse_phase_path(parser: &mut Parser<'a>) -> Result<Option<Self>> {
		todo!();
	}

	// Regex for path start: `~?[-\w_*+.]*/`
	fn parse_path_start(stream: &Stream<'_>) -> bool {
		let mut peek_stream = stream.clone();

		// a leading `~` is allowed, but doesn't guarantee that we're at start of path.
		let _ = peek_stream.take_if('~');
		while let Some(c) = peek_stream.take() {
			match c {
				// All the allowed chars don't tell us if we're a path or not.
				'-' | '*' | '.' | '+' | '_' => continue,
				c if c.is_alphanumeric() => continue,

				// If we hit a `/`, we know we've found a path (as)
				'/' => return true,

				// Anything else before hitting a `/` tells us it's not a path start.
				_ => return false,
			}
		}

		false // we reached end of stream before finding a `/`.
	}

	// Compound literals are strings, regexes, and paths. They're more complicated than "simple"
	// literals, such as identifiers or filesizes.
	fn parse_compound_literal(parser: &mut Parser<'a>) -> Result<Option<Self>> {
		if parser.stream_mut().take_if('"').is_some() {
			parser.enter_phase(Phase::Text);
			return Ok(Some(Self::StartString));
		}

		if parser.stream_mut().take_if("$/").is_some() {
			parser.enter_phase(Phase::Regex);
			return Ok(Some(Self::StartRegex));
		}

		if Self::parse_path_start(parser.stream()) {
			parser.enter_phase(Phase::Path);
			return Ok(Some(Self::StartPath));
		}

		Ok(None) // no compound literal found
	}

	fn parse_integer(stream: &mut Stream<'_>) -> Result<(u64, bool)> {
		let mut int: u64 = 0;
		let mut underscore = false;

		while let Some(c) = stream.take_if(|c: char| c.is_ascii_digit()) {
			if let Some(new) =
				int.checked_mul(10).and_then(|int| int.checked_add((c as u8 - b'0') as u64))
			{
				int = new;
			} else {
				return Err(stream.error(ErrorKind::IntegerLiteralTooLarge));
			}

			underscore |= stream.take_while(|c| c == '_').is_some();
		}

		Ok((int, underscore))
	}

	fn parse_number(stream: &mut Stream<'a>) -> Result<Self> {
		let (integer, underscore) = Self::parse_integer(steram)?;


		if integer <= 9999 && 
		todo!()
	}

	fn parse_identifier(stream: &mut Stream<'a>) -> Result<Self> {
		let name = stream.take_while(|c| c.is_alphanumeric() || c == '_').unwrap();

		match name {
			"if" => Ok(Self::If),
			"do" => Ok(Self::Do),
			"else" => Ok(Self::Else),
			"elif" => Ok(Self::Elif),
			"while" => Ok(Self::While),
			"for" => Ok(Self::For),
			"break" => Ok(Self::Break),
			"continue" => Ok(Self::Continue),
			"return" => Ok(Self::Return),
			"fn" => Ok(Self::Fn),
			"true" => Ok(Self::True),
			"false" => Ok(Self::False),
			"div" => Ok(if stream.take_if("=").is_some() { Self::DivideAssign } else { Self::Divide }),
			_ => Ok(Self::Identifier(name)),
		}
	}

	fn strip_whitespace_and_comments(stream: &mut Stream<'_>) {
		loop {
			// Strip leading whitespace, if any.
			let _ = stream.take_while(char::is_whitespace);

			// If it's a comment, parse that out, otherwise exit.
			if stream.take_if('#').is_some() {
				let _ = stream.take_while(|c| c != '\n');
			} else {
				break;
			}
		}
	}

	pub fn parse_no_phase(parser: &mut Parser<'a>) -> Result<Option<Self>> {
		Self::strip_whitespace_and_comments(parser.stream_mut());

		if let Some(compound) = Self::parse_compound_literal(parser)? {
			return Ok(Some(compound));
		}

		let stream = parser.stream_mut();

		macro_rules! ifeq {
			($ifeq:ident, $ifneq:ident) => {
				Ok(Some(if stream.take_if('=').is_some() { Self::$ifeq } else { Self::$ifneq }))
			};
		}

		match stream.take() {
			Some(',') => Ok(Some(Token::Comma)),
			Some(';') => Ok(Some(Self::Semicolon)),
			Some('(') => Ok(Some(Self::LeftParen)),
			Some(')') => Ok(Some(Self::RightParen)),
			Some('{') => Ok(Some(Self::LeftBrace)),
			Some('}') => Ok(Some(Self::RightBrace)),
			Some('[') => Ok(Some(Self::LeftBracket)),
			Some(']') => Ok(Some(Self::RightBracket)),

			Some('?') => Ok(Some(Self::Question)),
			Some(':') => Ok(Some(Self::Colon)),

			// Matching
			Some('!') if stream.take_if('~').is_some() => Ok(Some(Self::NotMatches)),
			Some('=') if stream.take_if('~').is_some() => Ok(Some(Self::Matches)),

			// Increment/Decrement
			Some('+') if stream.take_if('+').is_some() => Ok(Some(Self::Increment)),
			Some('-') if stream.take_if('-').is_some() => Ok(Some(Self::Decrement)),

			// Short circuit assignment
			Some('/') if stream.take_if('/').is_some() => ifeq!(UndefAssign, Undef),
			Some('&') if stream.take_if('&').is_some() => ifeq!(AndAndAssign, AndAnd),
			Some('|') if stream.take_if('|').is_some() => ifeq!(OrOrAssign, OrOr),

			// Logic
			Some('!') => ifeq!(NotEqual, Not),
			Some('=') => ifeq!(Equal, Assign),
			Some('<') => ifeq!(LessThanOrEqual, LessThan),
			Some('>') => ifeq!(GreaterThanOrEqual, GreaterThan),

			// Math
			Some('+') => ifeq!(AddAssign, Add),
			Some('-') => ifeq!(SubtractAssign, Subtract),
			Some('*') => ifeq!(MultiplyAssign, Multiply),
			Some('/') => ifeq!(DivideAssign, Divide),
			Some('%') => ifeq!(ModuloAssign, Modulo),
			Some('^') => ifeq!(PowAssign, Pow),

			Some(c @ '0'..='9') => {
				stream.untake(c);
				Self::parse_number(stream).map(Some)
			}

			Some(c) if c.is_alphabetic() => {
				stream.untake(c);
				Self::parse_identifier(stream).map(Some)
			}

			None => Ok(None),
			Some(other) => {
				stream.untake(other);
				Ok(None)
			}
		}
	}
}
/*

\w+/...
[~.*+\w]?[^/]*\/


*/
/*
	// misc
	Raw(Vec<u8>),
	CliArg(isize),
	EnvVar(OsString),
	Variable(String),
	Number(f64),
	DateTime(crate::DateTime),
	FileSize { fs: crate::FileSize, precision: u8 },

	// Keywords


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
*/

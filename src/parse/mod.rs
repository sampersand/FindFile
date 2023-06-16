mod lex_context;
mod stream;
pub(crate) mod token; // tmp

pub use lex_context::{LexContext, Phase};
pub use stream::Stream;
pub use token::Token;

#[derive(Debug)]
pub enum ParseError {
	Eof,
	VariableIsntUtf8,
	UnknownTokenStart(char),
	BadEscape(&'static str),
	InvalidEscape(char),
	CliArgTooLarge,
	InvalidDollarSign,
	BadFloat,
	MissingEndingBrace,
	InvalidCliPosition(isize),
	MissingEnvVar(std::ffi::OsString),
	BadPath(crate::pathglob::PathParseError),
	MissingEndQuote,
	NotAndEndOfExpression,
	MissingEndRegex,
	NoClosingRightParen,
	MissingRhsToAssignment,
	MissingRhsToOp,
	MissingRhsToLogicOp,
	FileSizeLiteralTooLarge,
	CliArgMissing,
	AssignToNonVariable,
	InvalidRegex(crate::regex::RegexParseError),
}

impl From<crate::regex::RegexParseError> for ParseError {
	fn from(err: crate::regex::RegexParseError) -> Self {
		Self::InvalidRegex(err)
	}
}

mod parse_context;
mod stream;
mod token;

pub use parse_context::{ParseContext, Phase};
pub use stream::Stream;
pub use token::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
	Eof,
	UnknownTokenStart(char),
	BadEscape(&'static str),
	InvalidEscape(char),
	CliArgTooLarge,
	InvalidDollarSign,
	MissingEndingBrace,
}

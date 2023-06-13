pub mod ast;
mod ast_context;
mod lex_context;
mod stream;
pub(crate) mod token; // tmp

pub use ast::Ast;

pub use ast_context::AstContext;
pub use lex_context::{LexContext, Phase};
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
	BadFloat,
	MissingEndingBrace,
	InvalidCliPosition(isize),
	MissingEnvVar(std::ffi::OsString),
	BadPath(crate::PathParseError),
	MissingEndQuote,
	NotAndEndOfExpression,
	MissingEndRegex,
	NoClosingRightParen,
	MissingRhsToAssignment,
	MissingRhsToOp,
	MissingRhsToLogicOp,
	AssignToNonVariable,
}

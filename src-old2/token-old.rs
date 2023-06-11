use crate::filesize::FileSize;
use crate::pcre::{PcreRegex, PcreRegexParseError};
use crate::posix::{PosixRegex, PosixRegexParseError};
use logos::Logos;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum LexingError {
	NumberParseError,
	PosixRegexParseError(PosixRegexParseError),
	PcreRegexParseError(PcreRegexParseError),
	#[default]
	Other,
}

impl From<std::num::ParseIntError> for LexingError {
	fn from(_: std::num::ParseIntError) -> Self {
		LexingError::NumberParseError
	}
}
impl From<PosixRegexParseError> for LexingError {
	fn from(err: PosixRegexParseError) -> Self {
		Self::PosixRegexParseError(err)
	}
}

impl From<PcreRegexParseError> for LexingError {
	fn from(err: PcreRegexParseError) -> Self {
		Self::PcreRegexParseError(err)
	}
}

impl From<std::num::ParseFloatError> for LexingError {
	fn from(_: std::num::ParseFloatError) -> Self {
		LexingError::NumberParseError
	}
}

#[derive(Logos, Debug, PartialEq)]
#[logos(error = LexingError)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"#[^\n]*")]
#[logos(subpattern digits = r"\d[\d_]*")]
#[logos(subpattern alpha = r"[a-zA-Z_]+")]
#[logos(subpattern alnum = r"[a-zA-Z_][_\w]*")]
#[logos(subpattern ident = r"[a-zA-Z_][_\w]*")]
#[rustfmt::skip]
pub enum Token<'a> {
	#[token("^{")] BeginBlockStart,
	#[token("${")] EndBlockStart,
	#[token("}")] BlockEnd,

	#[token("?")] Question,
	#[token(":")] Colon,
	#[token("(")] Open,
	#[token(")")] Close,
	#[token(",")] Comma,
	#[token("&")] And,
	#[token("|")] Or,

	#[token("+")] Add,
	#[token("-")] Sub,
	#[token("*")] Mul,
	#[token("//")] Div,
	#[token("^")] Pow,

	#[token("!")]   Not,
	#[token(":=")]  Assign,
	#[regex("==?")] Eql,
	#[token("!=")]  Neq,
	#[token("<")]   Lth,
	#[token("<=")]  Leq,
	#[token(">")]   Gth,
	#[token(">=")]  Geq,

	#[regex(r"\$(?&ident)", |lex| OsStr::new(&lex.slice()[1..]))]
	EnvVar(&'a OsStr),

	#[regex(r"\$-?(?&digits)", |lex| lex.slice()[1..].parse())]
	CliArg(isize),

	#[regex(r"(?&ident)")]
	Identifier(&'a str),

	#[regex(r"(?i)(?&digits)[kmgtpe]i?b?", |lex| lex.slice().parse::<FileSize>().expect("it should always work"))]
	FileSize(FileSize),

	// TODO: strip `\`s from path
	#[regex(r"([.~]?/)([^ ]|\\.)*", |lex| PathBuf::from(lex.slice()))]
	Path(PathBuf),

	// #[regex(r"(?i)(?&digits)[kmgtpe]i?b?", |lex| lex.slice().parse::<FileSize>().expect("it should always work"))]
	// DateTime(FileSize),

	#[regex(r"(?&digits)(.?(&digits))?([eE][-+]?(?&digits))", |lex| lex.slice().parse())]
	Number(f64),

	#[regex(r#""(\\.|[^"])*""#, |lex| lex.slice())]
	String(&'a str),

	#[regex(r"\$/(\\.|[^/])*/", |lex| lex.slice().parse())]
	PerlRegex(PcreRegex),

	#[regex(r"x/(\\.|[^/])*/", |lex| lex.slice().parse())]
	PosixRegex(PosixRegex)
}

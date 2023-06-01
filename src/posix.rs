use logos::Logos;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct PosixRegex<'a>(Vec<PosixRegexToken<'a>>);

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(error = PosixRegexError)]
enum PosixRegexToken<'a> {
	#[token("?")]
	AnyCharacter,

	#[token("*")]
	Wildcard,

	#[regex(r"\[.*\]", |lex| lex.slice().parse())]
	CharacterRange(CharRange),

	#[regex(r"[^*?\[\]]")]
	Characters(&'a str),
}

#[derive(Debug, PartialEq, Clone)]
pub enum PosixRegexError {
	Todo,
}

impl<'a> FromStr for PosixRegex<'a> {
	type Err = PosixRegexError;

	fn from_str(input: &str) -> Result<Self, Self::Err> {
		todo!()
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CharRange {
	x: i32,
}

impl FromStr for CharRange {
	type Err = PosixRegexError;

	fn from_str(input: &str) -> Result<Self, Self::Err> {
		todo!()
	}
}

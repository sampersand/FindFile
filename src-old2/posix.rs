#![allow(unused)]

#[derive(Debug, PartialEq)]
pub struct PosixRegex(Vec<PosixRegexToken>);

#[derive(Debug, PartialEq)]
enum PosixRegexToken {
	AnyCharacter,
	OneOrMore,
	Characters(String),
	CharacterRange(CharRange),
}

#[derive(Debug, PartialEq)]
enum CharRange {
	Foo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PosixRegexParseError {
	BackslashAtEndOfString,
}

impl std::str::FromStr for PosixRegex {
	type Err = PosixRegexParseError;

	fn from_str(source: &str) -> Result<Self, Self::Err> {
		let mut tokens = Vec::new();
		let mut current = String::new();

		let mut chars = source.chars();

		while let Some(c) = chars.next() {
			if c == '\\' {
				current.push(chars.next().ok_or(PosixRegexParseError::BackslashAtEndOfString)?);
				continue;
			}

			if c != '*' && c != '?' && c != '[' {
				current.push(c);
				continue;
			}

			if !current.is_empty() {
				tokens.push(PosixRegexToken::Characters(current));
				current = String::new();
			}

			match c {
				'?' => tokens.push(PosixRegexToken::AnyCharacter),
				'*' => tokens.push(PosixRegexToken::OneOrMore),
				'[' => {
					todo!()
				}
				_ => unreachable!(),
			}
		}

		Ok(Self(tokens))
	}
}

// use logos::Logos;
// use std::str::FromStr;

// #[derive(Debug, PartialEq)]
// pub struct PosixRegex<'a>(Vec<PosixRegexToken<'a>>);

// #[derive(Logos, Debug, Clone, PartialEq, Eq)]
// #[logos(PosixRegexParseError = PosixRegexParseError)]
// enum PosixRegexToken<'a> {
// 	#[token("?")]
// 	AnyCharacter,

// 	#[token("*")]
// 	Wildcard,

// 	#[regex(r"\[.*\]", |lex| lex.slice().parse())]
// 	CharacterRange(CharRange),

// 	#[regex(r"[^*?\[\]]")]
// 	Characters(&'a str),
// }

// #[derive(Debug, PartialEq, Clone)]
// pub enum PosixRegexParseError {
// 	Todo,
// }

// impl<'a> FromStr for PosixRegex<'a> {
// 	type Err = PosixRegexParseError;

// 	fn from_str(input: &str) -> Result<Self, Self::Err> {
// 		todo!()
// 	}
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// struct CharRange {
// 	x: i32,
// }

// impl FromStr for CharRange {
// 	type Err = PosixRegexParseError;

// 	fn from_str(input: &str) -> Result<Self, Self::Err> {
// 		todo!()
// 	}
// }

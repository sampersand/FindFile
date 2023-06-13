use crate::ast::Block;
use crate::parse::{LexContext, ParseError, Token};
use os_str_bytes::OsStrBytes;

#[derive(Debug, Clone, PartialEq)]
pub struct Interpolated {
	pub parts: Vec<(Vec<u8>, Block)>,
	pub tail: Vec<u8>,
}

impl Interpolated {
	pub fn parse_until(lctx: &mut LexContext, end: Token) -> Result<Self, ParseError> {
		let mut parts = Vec::new();
		let mut current = Vec::new();

		loop {
			match lctx.next()?.expect("this should be an error in the lexer") {
				Token::CliArg(pos) => {
					let cli = lctx.get_cli(pos).ok_or(ParseError::InvalidCliPosition(pos))?;
					current.extend(cli.to_raw_bytes().iter());
				}
				Token::EnvVar(var) => {
					let env = lctx.get_env(&var).ok_or(ParseError::MissingEnvVar(var))?;
					current.extend(env.to_raw_bytes().iter());
				}
				Token::Raw(data) => current.extend(&data),
				Token::BeginBraceEscape => {
					let expr = Block::parse_until(lctx, Token::EndBraceEscape)?;
					parts.push((std::mem::take(&mut current), expr));
				}
				x if x == end => break,
				token => unreachable!("invalid token in interpolation: {token:?}"),
			}
		}

		Ok(Self { parts, tail: current })
	}
}

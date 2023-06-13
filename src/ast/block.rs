use crate::ast::{Expression, Precedence};
use crate::parse::{LexContext, ParseError, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct Block(Vec<Expression>);

impl Block {
	pub fn parse_until(lctx: &mut LexContext, end: Token) -> Result<Self, ParseError> {
		let mut args = Vec::new();

		while let Some(expr) = Expression::parse(lctx, true, Precedence::default())? {
			args.push(expr);
			if !lctx.take_if(Token::Semicolon)? {
				break;
			}
		}

		if !lctx.take_if(end)? {
			return Err(ParseError::NoClosingRightParen);
		}

		Ok(Self(args))
	}
}

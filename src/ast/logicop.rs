use crate::parse::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicOperator {
	Equal,
	NotEqual,
	LessThan,
	LessThanOrEqual,
	GreaterThan,
	GreaterThanOrEqual,
}

impl LogicOperator {
	pub fn from_token(token: &Token) -> Option<Self> {
		match token {
			Token::Equal => Some(Self::Equal),
			Token::NotEqual => Some(Self::NotEqual),
			Token::LessThan => Some(Self::LessThan),
			Token::LessThanOrEqual => Some(Self::LessThanOrEqual),
			Token::GreaterThan => Some(Self::GreaterThan),
			Token::GreaterThanOrEqual => Some(Self::GreaterThanOrEqual),
			_ => None,
		}
	}
}

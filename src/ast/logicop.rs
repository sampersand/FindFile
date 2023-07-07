use crate::parse::Token;
use crate::vm::{Builder, Opcode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicOperator {
	Matches,
	NotMatches,
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
			Token::Matches => Some(Self::Matches),
			Token::NotMatches => Some(Self::NotMatches),
			Token::Equal => Some(Self::Equal),
			Token::NotEqual => Some(Self::NotEqual),
			Token::LessThan => Some(Self::LessThan),
			Token::LessThanOrEqual => Some(Self::LessThanOrEqual),
			Token::GreaterThan => Some(Self::GreaterThan),
			Token::GreaterThanOrEqual => Some(Self::GreaterThanOrEqual),
			_ => None,
		}
	}

	pub fn compile(self, builder: &mut Builder) {
		match self {
			Self::Matches => builder.opcode(Opcode::Matches),
			Self::NotMatches => builder.opcode(Opcode::NotMatches),
			Self::Equal => builder.opcode(Opcode::Equal),
			Self::NotEqual => builder.opcode(Opcode::NotEqual),
			Self::LessThan => builder.opcode(Opcode::LessThan),
			Self::LessThanOrEqual => builder.opcode(Opcode::LessThanOrEqual),
			Self::GreaterThan => builder.opcode(Opcode::GreaterThan),
			Self::GreaterThanOrEqual => builder.opcode(Opcode::GreaterThanOrEqual),
		}
	}
}

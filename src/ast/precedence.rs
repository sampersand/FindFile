use crate::parse::Token;

/// Order of operations precedence. The lower it is, the less tight the binding is.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
	MulDivMod,
	AddSub,
	Logic,
	ShortCircuit,
	Assignment,
	Comma,

	#[default]
	TopLevel,
}

impl Precedence {
	pub fn of(token: &Token, comma_is_and: bool) -> Option<Self> {
		match token {
			Token::Add | Token::Subtract => Some(Self::AddSub),
			Token::Multiply | Token::Divide | Token::Modulo => Some(Self::MulDivMod),

			Token::AddAssign
			| Token::SubtractAssign
			| Token::MultiplyAssign
			| Token::DivideAssign
			| Token::ModuloAssign
			| Token::Assign => Some(Self::Assignment),

			Token::Matches
			| Token::NotMatches
			| Token::Equal
			| Token::NotEqual
			| Token::LessThan
			| Token::LessThanOrEqual
			| Token::GreaterThan
			| Token::GreaterThanOrEqual => Some(Self::Logic),

			Token::And | Token::Or => Some(Self::ShortCircuit),
			Token::Comma if comma_is_and => Some(Self::Comma),
			_ => None,
		}
	}
}

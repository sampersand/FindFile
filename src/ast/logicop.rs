use crate::parse::Token;
use crate::play::PlayResult;
use crate::Value;
use std::cmp::Ordering;

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
	pub fn run(self, lhs: &Value, rhs: &Value) -> PlayResult<bool> {
		if self == Self::Matches || self == Self::NotMatches {
			// we do it backwards because `contents =~ $/foo/` is normal usage, but it's really
			// `$/foo/.matches(contents)`
			return Ok(rhs.matches(lhs)? == (self == Self::Matches));
		}

		let cmp = lhs.compare(rhs)?;
		match self {
			Self::Matches => unreachable!(),
			Self::NotMatches => unreachable!(),
			Self::Equal => Ok(cmp == Ordering::Equal),
			Self::NotEqual => Ok(cmp != Ordering::Equal),
			Self::LessThan => Ok(cmp < Ordering::Equal),
			Self::LessThanOrEqual => Ok(cmp <= Ordering::Equal),
			Self::GreaterThan => Ok(cmp > Ordering::Equal),
			Self::GreaterThanOrEqual => Ok(cmp >= Ordering::Equal),
		}
	}

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
}

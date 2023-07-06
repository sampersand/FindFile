use crate::ast::Precedence;
use crate::parse::Token;
use crate::play::PlayResult;
use crate::vm::Builder;
use crate::vm::Opcode;
use crate::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathOperator {
	Add,
	Subtract,
	Multiply,
	Divide,
	Modulo,
}
impl MathOperator {
	pub fn precedence(self) -> Precedence {
		match self {
			Self::Add | Self::Subtract => Precedence::AddSub,
			Self::Multiply | Self::Divide | Self::Modulo => Precedence::MulDivMod,
		}
	}

	pub fn run(self, lhs: &Value, rhs: &Value) -> PlayResult<Value> {
		match self {
			Self::Add => lhs.add(rhs).map_err(From::from),
			Self::Subtract => lhs.subtract(rhs).map_err(From::from),
			Self::Multiply => lhs.multiply(rhs).map_err(From::from),
			Self::Divide => lhs.divide(rhs).map_err(From::from),
			Self::Modulo => lhs.modulo(rhs).map_err(From::from),
		}
	}

	// the bool is whether it was an assignment
	pub fn from_token(token: &Token) -> Option<(Self, bool)> {
		match token {
			Token::Add => Some((Self::Add, false)),
			Token::AddAssign => Some((Self::Add, true)),
			Token::Subtract => Some((Self::Subtract, false)),
			Token::SubtractAssign => Some((Self::Subtract, true)),
			Token::Multiply => Some((Self::Multiply, false)),
			Token::MultiplyAssign => Some((Self::Multiply, true)),
			Token::Divide => Some((Self::Divide, false)),
			Token::DivideAssign => Some((Self::Divide, true)),
			Token::Modulo => Some((Self::Modulo, false)),
			Token::ModuloAssign => Some((Self::Modulo, true)),
			_ => None,
		}
	}

	pub fn compile(self, builder: &mut Builder) {
		match self {
			Self::Add => builder.opcode(Opcode::Add),
			Self::Subtract => builder.opcode(Opcode::Subtract),
			Self::Multiply => builder.opcode(Opcode::Multiply),
			Self::Divide => builder.opcode(Opcode::Divide),
			Self::Modulo => builder.opcode(Opcode::Modulo),
		}
	}
}

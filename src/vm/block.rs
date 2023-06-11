use crate::vm::Opcode;
use crate::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
	// stuff
}

#[derive(Default)]
pub struct BlockBuilder {
	code: Vec<Opcode>,
	consts: Vec<Value>,
	nlocals: usize,
}

impl BlockBuilder {
	pub fn build(self) -> Block {
		todo!()
	}

	fn opcode(&mut self, op: Opcode) {
		self.code.push(op);
	}

	pub fn load_const(&mut self, value: Value) {
		for (idx, constant) in self.consts.iter().enumerate() {
			if *constant == value {
				self.opcode(Opcode::LoadConstant(idx));
				return;
			}
		}

		self.opcode(Opcode::LoadConstant(self.consts.len()));
		self.consts.push(value);
	}
}

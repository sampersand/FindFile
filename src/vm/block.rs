use crate::vm::Opcode;
use crate::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
	// stuff
}

pub struct Builder<'a> {
	code: Vec<Opcode>,
	consts: Vec<Value>,
	arguments: Vec<String>,
	global_vars: &'a mut HashMap<String, usize>,
}

impl<'a> Builder<'a> {
	pub fn new<T>(args: T, global_vars: &'a mut HashMap<String, usize>) -> Self
	where
		T: IntoIterator<Item = String>,
	{
		Self {
			code: Vec::new(),
			consts: Vec::new(),
			arguments: args.into_iter().collect(),
			global_vars,
		}
	}

	pub fn build(self) -> Block {
		todo!()
	}

	pub fn opcode(&mut self, op: Opcode) {
		self.code.push(op);
	}

	pub fn load_variable(&mut self, name: &str) {
		if let Some(idx) = self.arguments.iter().position(|x| x == name) {
			self.opcode(Opcode::LoadArgument(idx));
			return;
		}

		let index;

		if let Some(&idx) = self.global_vars.get(name) {
			index = idx;
		} else {
			index = self.global_vars.len();
			self.global_vars.insert(name.to_owned(), index);
		};

		self.opcode(Opcode::LoadVariable(index))
	}

	pub fn load_constant(&mut self, value: Value) {
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

use super::Block;
use crate::vm::Opcode;
use crate::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildContext {
	TopLevel,
	Logical,
	Normal,
}

pub struct Builder<'a> {
	code: Vec<Opcode>,
	consts: Vec<Value>,
	loops: Vec<(LoopToken, Vec<JumpIndex>)>, // (loop start, jump_to_ends)
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
			loops: Vec::new(),
			consts: Vec::new(),
			arguments: args.into_iter().collect(),
			global_vars,
		}
	}

	pub fn build(self) -> Block {
		assert!(self.loops.is_empty());
		Block { code: self.code, consts: self.consts, args: self.arguments }
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

	pub fn store_variable(&mut self, name: &str) {
		if let Some(idx) = self.arguments.iter().position(|x| x == name) {
			self.opcode(Opcode::StoreArgument(idx));
			return;
		}

		let index;

		if let Some(&idx) = self.global_vars.get(name) {
			index = idx;
		} else {
			index = self.global_vars.len();
			self.global_vars.insert(name.to_owned(), index);
		};

		self.opcode(Opcode::StoreVariable(index))
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

	pub fn defer_jump(&mut self) -> JumpIndex {
		let idx = self.code.len();
		self.opcode(Opcode::Illegal);
		JumpIndex(idx)
	}

	pub fn position(&mut self) -> Position {
		Position(self.code.len())
	}

	pub fn jump_unconditional(&mut self, pos: Position) {
		self.opcode(Opcode::Jump(pos.0));
	}

	pub fn enter_loop(&mut self) -> LoopToken {
		let token = LoopToken(self.code.len());
		self.loops.push((token, vec![]));
		token
	}

	pub fn exit_loop(&mut self, token: LoopToken) {
		let (top_token, jump_indices) = self.loops.pop().unwrap();
		assert_eq!(top_token, token);

		for jump_index in jump_indices {
			jump_index.jump_unconditional(self);
		}
	}

	pub fn jump_to_loop_start(&mut self) -> Result<(), crate::parse::ParseError> {
		let Some((pos, _)) = self.loops.last() else {
			return Err(crate::parse::ParseError::Message("continue out of loop"));
		};

		self.jump_unconditional(Position(pos.0));
		Ok(())
	}

	pub fn jump_to_loop_end(&mut self) -> Result<(), crate::parse::ParseError> {
		let deferred = self.defer_jump();
		if let Some((_, jumps)) = self.loops.last_mut() {
			jumps.push(deferred);
			Ok(())
		} else {
			Err(crate::parse::ParseError::Message("break out of loop"))
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopToken(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpIndex(usize);

impl JumpIndex {
	pub fn jump_unconditional(self, builder: &mut Builder) {
		builder.code[self.0] = Opcode::Jump(builder.code.len());
	}

	pub fn jump_if(self, builder: &mut Builder) {
		builder.code[self.0] = Opcode::JumpIf(builder.code.len());
	}

	pub fn jump_unless(self, builder: &mut Builder) {
		builder.code[self.0] = Opcode::JumpUnless(builder.code.len());
	}
}

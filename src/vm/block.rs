use crate::vm::{Opcode, RunError, Vm};
use crate::Value;
use core::cmp::Ordering;
use std::collections::HashMap;

mod builder;
pub use builder::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
	code: Vec<Opcode>,
	consts: Vec<Value>,
	args: Vec<String>, // names are given for fun.
}

#[derive(Debug)]
struct Stackframe<'b, 'v> {
	block: &'b Block,
	vm: &'v mut Vm,
	ip: usize,
	stack: Vec<Value>,
	args: Vec<Value>,
}

impl Block {
	pub fn run(&self, vm: &mut Vm) -> Result<Value, RunError> {
		Stackframe { block: self, vm, ip: 0, stack: Vec::new(), args: vec![] }.run()
	}
}

impl Stackframe<'_, '_> {
	fn jump_to(&mut self, idx: usize) {
		debug_assert!(idx <= self.block.code.len(), "{self:?}");
		self.ip = idx;
	}

	fn next_opcode(&mut self) -> Option<Opcode> {
		let &op = self.block.code.get(self.ip)?;
		self.ip += 1;
		Some(op)
	}

	fn run(mut self) -> Result<Value, RunError> {
		while let Some(opcode) = self.next_opcode() {
			if let Some(return_value) = self.run_opcode(opcode)? {
				return Ok(return_value);
			}
		}

		Ok(self.stack.pop().unwrap_or_default())
	}

	fn push(&mut self, value: Value) {
		self.stack.push(value);
	}

	fn pop(&mut self) -> Value {
		self.stack.pop().expect("<internal error: popped from end of stack>")
	}

	fn run_opcode(&mut self, opcode: Opcode) -> Result<Option<Value>, RunError> {
		use Opcode::*;
		let mut args = (0..opcode.arity()).map(|_| self.pop()).collect::<Vec<_>>();

		let topush = match opcode {
			Illegal => unreachable!(),
			LoadConstant(idx) => self.block.consts[idx].clone(),
			LoadArgument(idx) => self.args[idx].clone(),
			LoadVariable(idx) => self.vm.get_variable(idx).unwrap_or_default(), // todo: is default correct?
			StoreArgument(idx) => {
				self.args[idx] = args.remove(0);
				return Ok(None);
			}
			StoreVariable(idx) => {
				self.vm.store_variable(idx, args.remove(0));
				return Ok(None);
			}

			Dup => {
				self.push(args[0].clone());
				args.remove(0)
			}
			Pop => return Ok(None),

			GenericCall(amnt) => args[0].call(&args[1..])?,

			CreatePath(_usize) => todo!(),
			CreateRegex(_usize) => todo!(),
			CreateString(_usize) => todo!(),

			Return => return Ok(Some(args.remove(0))),
			Jump(position) => {
				self.jump_to(position);
				return Ok(None);
			}
			JumpIf(position) | JumpUnless(position) => {
				if args[0].is_truthy(self.vm)? == matches!(opcode, Opcode::JumpIf(_)) {
					self.jump_to(position);
				}
				return Ok(None);
			}

			Not => (!args[0].is_truthy(self.vm)?).into(),
			Negate => todo!(),
			UPositive => todo!(),
			ForcedLogical => args[0].logical(self.vm)?.into(),

			Add => args[0].add(&args[1])?,
			Subtract => args[0].subtract(&args[1])?,
			Multiply => args[0].multiply(&args[1])?,
			Divide => args[0].divide(&args[1])?,
			Modulo => args[0].modulo(&args[1])?,

			Matches => todo!(),
			NotMatches => todo!(),
			Equal => (args[0].compare(&args[1])? == Ordering::Equal).into(),
			NotEqual => (args[0].compare(&args[1])? != Ordering::Equal).into(),
			LessThan => (args[0].compare(&args[1])? < Ordering::Equal).into(),
			LessThanOrEqual => (args[0].compare(&args[1])? <= Ordering::Equal).into(),
			GreaterThan => (args[0].compare(&args[1])? > Ordering::Equal).into(),
			GreaterThanOrEqual => (args[0].compare(&args[1])? >= Ordering::Equal).into(),

			// Querying
			IsFile { implicit } => todo!(),
			IsDirectory { implicit } => todo!(),
			IsExecutable { implicit } => todo!(),
			IsSymlink { implicit } => todo!(),
			IsBinary { implicit } => todo!(),
			IsHidden { implicit } => todo!(),
			IsGitIgnored { implicit } => todo!(),
			IsOk(_usize) => todo!(),

			// Path-related funcitons
			FileSize { implicit: true } => self.vm.info().content_size().into(),
			FileSize { implicit: false } => todo!(),
			PushRoot => todo!(),
			PushPath => todo!(),
			PushPwd => todo!(),
			Dirname { implicit } => todo!(),
			Extname { implicit } => todo!(),
			ExtnameDot { implicit } => todo!(),
			Basename { implicit } => todo!(),
			Stemname { implicit } => todo!(),

			// Misc
			Print(_usize) => todo!(),
			Write(_usize) => todo!(), // same as print just no newline at end
			Skip => todo!(),
			Quit { implicit } => todo!(),
			Depth { implicit } => todo!(),
			Sleep { implicit } => todo!(),

			// Interactiv => todo!()e
			Mv { implicit, force } => todo!(),
			Rm { implicit, force } => todo!(),
			RmR { implicit, force } => todo!(),
			Cp { implicit, force } => todo!(),
			Ln { implicit, force } => todo!(),
			LnS { implicit, force } => todo!(),
			Mkdir => todo!(),
			Touch { implicit } => todo!(),
		};

		self.push(topush);
		Ok(None)
	}
}

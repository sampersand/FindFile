use crate::play::PathInfo;
use crate::vm::{Opcode, RunError, Vm};
use crate::Value;
use core::cmp::Ordering;
use os_str_bytes::OsStrBytes;
use std::ffi::OsStr;

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

		macro_rules! info {
			($cond:ident) => {
				info!($cond, 0)
			};
			($cond:ident, $pos:literal) => {
				if $cond {
					std::borrow::Cow::Borrowed(self.vm.info())
				} else if let Value::Text(ref name) = args[$pos] {
					std::borrow::Cow::Owned(PathInfo::new(&std::path::Path::new(
						&OsStr::assert_from_raw_bytes(name.as_ref()),
					))?)
				} else {
					panic!("todo: error the argument isnt a path");
				}
				.as_ref()
			};
		}

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

			GenericCall(_amnt) => args[0].call(&args[1..])?,

			CreatePath(_usize) => todo!(),
			CreateRegex(_usize) => todo!(),
			CreateString(_usize) => todo!(),

			Return => return Ok(Some(args.remove(0))),
			Jump(position) => {
				self.jump_to(position);
				return Ok(None);
			}
			JumpIf(position) | JumpUnless(position) => {
				if args[0].logical(self.vm)? == matches!(opcode, Opcode::JumpIf(_)) {
					self.jump_to(position);
				}
				return Ok(None);
			}

			Not => (!args[0].logical(self.vm)?).into(),
			Negate => args[0].negate()?.into(),
			UPositive => todo!(),
			ForcedLogical => args[0].logical(self.vm)?.into(),

			Add => args[1].add(&args[0])?,
			Subtract => args[1].subtract(&args[0])?,
			Multiply => args[1].multiply(&args[0])?,
			Divide => args[1].divide(&args[0])?,
			Modulo => args[1].modulo(&args[0])?,

			Matches => args[1].matches(&args[0])?.into(),
			NotMatches => (!args[1].matches(&args[0])?).into(),
			Equal => (args[1].compare(&args[0])? == Ordering::Equal).into(),
			NotEqual => (args[1].compare(&args[0])? != Ordering::Equal).into(),
			LessThan => (args[1].compare(&args[0])? < Ordering::Equal).into(),
			LessThanOrEqual => (args[1].compare(&args[0])? <= Ordering::Equal).into(),
			GreaterThan => (args[1].compare(&args[0])? > Ordering::Equal).into(),
			GreaterThanOrEqual => (args[1].compare(&args[0])? >= Ordering::Equal).into(),

			// Querying
			IsFile { implicit } => info!(implicit).is_file().into(),
			IsDirectory { implicit } => info!(implicit).is_dir().into(),
			IsExecutable { implicit: _ } => todo!(),
			IsSymlink { implicit: _ } => todo!(),
			IsBinary { implicit: _ } => todo!(),
			IsHidden { implicit } => info!(implicit).is_hidden().into(),
			IsGitIgnored { implicit: _ } => todo!(),
			IsOk(_usize) => todo!(),

			// Path-related funcitons
			FileSize { implicit } => info!(implicit).content_size().into(),
			PushRoot => self.vm.root().clone().into(),
			PushPath => self.vm.info().path()._rc().clone().into(),
			PushPwd => todo!(),
			Dirname { implicit } => info!(implicit).path().parent().into(),
			Extname { implicit } => info!(implicit).path().extension().into(),
			ExtnameDot { implicit } => info!(implicit).extnamedot().into(),
			Basename { implicit } => info!(implicit).path().base().into(),
			Stemname { implicit } => info!(implicit).path().stem().into(),

			// Misc
			Print(_usize) | Write(_usize) => {
				for arg in args.iter().rev() {
					match arg {
						Value::Text(txt) => print!("{}", String::from_utf8_lossy(&txt)),
						other => print!("{other:?}"),
					}
				}
				if matches!(opcode, Print(_)) {
					println!(); // todo: println `\0`?
				}
				args.remove(0)
			}
			Skip => todo!(),
			Quit { implicit } => std::process::exit(if implicit { 0 } else { todo!("top to int") }),
			Depth { implicit: _ } => todo!(),
			Sleep { implicit: _ } => todo!(),

			// Interactive
			Mv { implicit: _, force: _ } => todo!(),
			Rm { implicit: _, force: _ } => todo!(),
			RmR { implicit: _, force: _ } => todo!(),
			Cp { implicit: _, force: _ } => todo!(),
			Ln { implicit: _, force: _ } => todo!(),
			LnS { implicit: _, force: _ } => todo!(),
			Mkdir => todo!(),
			Touch { implicit: _ } => todo!(),
		};

		self.push(topush);
		Ok(None)
	}
}

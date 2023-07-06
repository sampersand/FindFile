use crate::ast::Expression;
use crate::parse::ParseError;
use crate::play::PathInfo;
use crate::vm::Block;
use crate::vm::{self, RunResult};
use crate::Value;
use std::collections::HashMap;

mod config;
pub use config::Config;

#[derive(Debug)]
pub struct Vm {
	config: Config,
	vars: Vec<Option<Value>>,
	info: Option<PathInfo>,
}

impl Vm {
	pub fn compile(config: Config, expr: Expression) -> Result<(Self, Block), ParseError> {
		let mut map = HashMap::new();
		let mut builder = vm::block::Builder::new(vec![], &mut map);
		expr.compile(&mut builder)?;

		let block = builder.build();
		let vm = Self { config, vars: vec![None; map.len()], info: None };

		Ok((vm, block))
	}

	pub fn info(&self) -> &PathInfo {
		self.info.as_ref().expect("todo: when wouldnt we have one?")
	}

	pub fn info_mut(&mut self) -> &mut PathInfo {
		self.info.as_mut().expect("todo: when wouldnt we have one?")
	}

	pub fn get_variable(&self, idx: usize) -> Option<Value> {
		self.vars[idx].clone()
	}

	pub fn store_variable(&mut self, idx: usize, value: Value) {
		self.vars[idx] = Some(value);
	}

	pub fn set_pathinfo(&mut self, info: PathInfo) {
		self.info = Some(info);
	}
}

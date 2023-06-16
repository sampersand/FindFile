use crate::ast::Expression;
use crate::play::RunContext;
use crate::play::{PlayContext, PlayResult};
use crate::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct Program {
	exprs: Vec<Expression>,
	pub(crate) vars: HashMap<String, Value>,
}

impl Program {
	pub fn new(exprs: Vec<Expression>) -> Self {
		Self { exprs, vars: Default::default() }
	}

	fn _play<T: AsRef<Path> + ?Sized>(
		&mut self,
		exprs: &[Expression],
		start: &T,
		rctx: RunContext,
	) -> PlayResult<()> {
		// dbg!(start.display());
		for entry in std::fs::read_dir(start.as_ref())? {
			let mut ctx = PlayContext::new(self, entry?)?;

			let mut matched = true;
			for expr in exprs {
				if !expr.run(&mut ctx, rctx).map_or(false, |x| x.is_truthy()) {
					matched = false;
					break;
				}
			}

			if matched {
				println!("{}", ctx.path().display());
			}

			if ctx.is_dir() {
				let path = ctx.take_path();
				self._play(exprs, &path, rctx)?;
			}
		}

		Ok(())
	}

	pub fn play<T: AsRef<Path> + ?Sized>(mut self, start: &T) -> PlayResult<()> {
		let start = start.as_ref();

		debug_assert!(start.is_dir());
		let exprs = std::mem::take(&mut self.exprs);

		self._play(&exprs, start, RunContext::Logical)
	}
}
// ::fs::read_dir

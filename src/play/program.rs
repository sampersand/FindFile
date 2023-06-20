use crate::ast::Expression;
use crate::play::RunContext;
use crate::play::{PlayContext, PlayResult};
use crate::Value;
use std::collections::HashMap;
use std::path::Path;

// mod config;
// use config::Config;

#[derive(Debug)]
pub struct Program {
	exprs: Vec<Expression>,
	// config: Config,
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
				if let Err(err) = self._play(exprs, &path, rctx) {
					eprintln!("err: {err:?}");
				}
			}
		}

		Ok(())
	}

	pub fn play(mut self) -> PlayResult<()> {
		let mut start_pos = vec![];
		// Todo: unify multiple exprs starting positions
		for expr in &self.exprs {
			start_pos.extend(expr.begin_position());
		}

		if start_pos.is_empty() {
			start_pos.push(".".into());
		}

		let exprs = std::mem::take(&mut self.exprs);

		for start in start_pos {
			self._play(&exprs, &start, RunContext::Logical)?;
		}

		Ok(())
	}
}
// ::fs::read_dir

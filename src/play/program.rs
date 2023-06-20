use crate::ast::Expression;
use crate::play::RunContext;
use crate::play::{PlayContext, PlayResult};
use crate::Value;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

mod config;
use config::Config;

#[derive(Default, Debug)]
pub struct Program {
	config: Config,
	// env: HashMap<OsString, Option<OsString>>,
	pub(crate) vars: HashMap<String, Value>,
}

impl Program {
	pub fn new(config: Config) -> Self {
		Self { config, vars: Default::default() }
	}

	pub fn cli(&self) -> &[std::ffi::OsString] {
		&self.config.cli
	}

	pub fn run_file(&mut self, path: &Path) -> PlayResult<()> {
		let contents = std::fs::read_to_string(path)?;
		let mut lctx = crate::parse::LexContext::new(&contents, self);
		let expr = Expression::parse_toplevel(&mut lctx)?;
		self.play(&expr)
	}

	fn _play<T: AsRef<Path> + ?Sized>(&mut self, expr: &Expression, start: &T) -> PlayResult<usize> {
		for entry in std::fs::read_dir(start.as_ref())? {
			let mut ctx = PlayContext::new(self, entry?)?;

			let mut matched = expr.run(&mut ctx, RunContext::Logical).map_or(false, |x| x.is_truthy());
			let (is_dir, path) = (ctx.is_dir(), ctx.take_path());

			if self.config.invert {
				matched = !matched;
			}
			let matched = matched;

			if matched && config.{
				println!("{}", path.display());
			}

			if is_dir {
				if let Err(err) = self._play(expr, &path) {
					eprintln!("err: {err:?}");
				}
			}
		}

		Ok(())
	}

	pub fn play(&mut self, expr: &Expression) -> PlayResult<()> {
		let start_positions = match expr.begin_position() {
			x if x.is_empty() => vec![".".into()],
			x => x,
		};

		let mut num_matches = 0;
		for start in start_positions {
			num_matches += self._play(expr, &start)?;
		}

		if self.config.count {
			println!("{num_matches}");
		}

		Ok(())
	}
}
// ::fs::read_dir

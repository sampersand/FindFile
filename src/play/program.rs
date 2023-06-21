use crate::ast::Expression;
use crate::play::context::FileInfo;
use crate::play::Env;
use crate::play::PlayError;
use crate::play::RunContext;
use crate::play::{PlayContext, PlayResult};
use crate::Value;
use os_str_bytes::OsStrBytes;
use std::collections::HashMap;
use std::io::{ErrorKind as IoErrorKind, Write};
use std::path::Path;
use std::path::PathBuf;

mod config;
pub use config::Config;

#[derive(Default, Debug)]
pub struct Program {
	config: Config,
	env: Env,
	vars: HashMap<String, Value>,
}

impl Program {
	pub fn new(config: Config, env: Env) -> Self {
		Self { config, env, vars: Default::default() }
	}

	pub fn assign_var(&mut self, name: &str, value: Value) {
		self.vars.insert(name.into(), value);
	}

	pub fn get_var(&mut self, name: &str) -> Option<Value> {
		self.vars.get(name).cloned()
	}

	pub fn config(&self) -> &Config {
		&self.config
	}

	pub fn env(&self) -> &Env {
		&self.env
	}

	pub fn run_file(&mut self, path: &Path) -> PlayResult<()> {
		self.play_expr(&std::fs::read_to_string(path)?)
	}

	fn _play<T: AsRef<Path> + ?Sized>(&mut self, expr: &Expression, start: &T) -> PlayResult<usize> {
		let mut num_matches = 0;

		for entry in std::fs::read_dir(start.as_ref())? {
			let mut ctx = PlayContext::new(self, entry?)?;
			let matched = expr.run(&mut ctx, RunContext::Logical).map_or(false, |x| x.is_truthy());
			let fileinfo = ctx.file_info;

			// Invert `matched` if given the `!` flag.
			let matched = if self.config.is_inverted() { !matched } else { matched };

			if matched {
				num_matches += 1;
			}

			if matched && self.config.should_print() {
				let mut stdout = std::io::stdout().lock();
				stdout.write_all(&fileinfo.path.as_ref().as_os_str().to_raw_bytes())?;
				self.config.write_line_ending(stdout)?;
			}

			if fileinfo.file_type.is_dir() {
				let path = fileinfo.path;
				match self._play(expr, &path) {
					Ok(match_count) => num_matches += match_count,
					Err(err) => self.config.handle_error(err)?,
				}
			}
		}

		Ok(num_matches)
	}

	pub fn play_expr(&mut self, source: &str) -> PlayResult<()> {
		let mut lctx = crate::parse::LexContext::new(source, self);
		let expr = Expression::parse_toplevel(&mut lctx)?;
		self.play(&expr)
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

		if self.config.is_counting() {
			println!("{num_matches}");
		}

		Ok(())
	}
}

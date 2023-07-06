use crate::ast::Expression;
use crate::play::Env;
use crate::play::PathInfo;
use crate::play::PlayError;
use crate::play::{PlayContext, PlayResult};
use crate::Value;
use os_str_bytes::OsStrBytes;
use std::collections::HashMap;
use std::ffi::OsString;
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

	pub fn play_expr(&mut self, source: &str) -> PlayResult<()> {
		let mut lctx = crate::parse::LexContext::new(source, self);
		let expr = Expression::parse_toplevel(&mut lctx)?;
		self.play(&expr)
	}

	fn _play<T: AsRef<Path> + ?Sized>(
		&mut self,
		vm: &mut crate::vm::Vm,
		block: &crate::vm::Block,
		start: &T,
	) -> PlayResult<usize> {
		let mut num_matches = 0;

		for entry in std::fs::read_dir(start.as_ref())? {
			num_matches += self.handle(entry?.path(), vm, block, true)?;
		}

		Ok(num_matches)
	}

	fn handle(
		&mut self,
		name: PathBuf,
		vm: &mut crate::vm::Vm,
		block: &crate::vm::Block,
		recur: bool,
	) -> PlayResult<usize> {
		let mut ctx = PlayContext::new(self, name)?;
		let pathinfo = ctx.into_pathinfo();
		vm.set_pathinfo(pathinfo.clone());
		let matched = block.run(vm).map_or(false, |x| x.is_truthy());

		// Invert `matched` if given the `!` flag.
		let matched = if self.config.is_inverted() { !matched } else { matched };

		let mut num_matches = 0;
		if matched {
			num_matches += 1;
		}

		if matched && self.config.should_print() {
			let mut stdout = std::io::stdout().lock();
			stdout.write_all(&pathinfo.path()._rc().as_os_str().to_raw_bytes())?;
			self.config.write_line_ending(stdout)?;
		}

		if pathinfo.is_dir() && recur {
			// ensure we take it so the rest of the `pathinfo` struct can be dropped
			let path = pathinfo.path()._rc().clone();
			drop(pathinfo);
			match self._play(vm, block, &path) {
				Ok(match_count) => num_matches += match_count,
				Err(err) => self.config.handle_error(err)?,
			}
		}

		Ok(num_matches)
	}

	pub fn play(&mut self, expr: &Expression) -> PlayResult<()> {
		let start_positions = match expr.begin_position() {
			x if x.is_empty() => vec![".".into()],
			x => x,
		};

		let (mut vm, block) =
			crate::vm::Vm::compile(Default::default(), expr.clone()).expect("bad expr");

		let mut num_matches = 0;
		for start in start_positions {
			num_matches += self.handle(start.clone(), &mut vm, &block, false)?;
			num_matches += self._play(&mut vm, &block, &start)?;
		}

		if self.config.is_counting() {
			println!("{num_matches}");
		}

		Ok(())
	}
}

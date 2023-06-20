use crate::ast::Expression;
use crate::play::context::FileInfo;
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
pub use config::{Config, When};

#[derive(Default, Debug)]
pub struct Program {
	config: Config,
	// env: HashMap<OsString, Option<OsString>>,
	vars: HashMap<String, Value>,
}

impl Program {
	pub fn new(config: Config) -> Self {
		Self { config, vars: Default::default() }
	}

	pub fn assign_var(&mut self, name: &str, value: Value) {
		self.vars.insert(name.into(), value);
	}

	pub fn get_var(&mut self, name: &str) -> Option<Value> {
		self.vars.get(name).cloned()
	}

	pub fn cli(&self) -> &[std::ffi::OsString] {
		&self.config.cli
	}

	pub fn run_file(&mut self, path: &Path) -> PlayResult<()> {
		self.play_expr(&std::fs::read_to_string(path)?)
	}

	fn line_ending(&self) -> u8 {
		if self.config.print0 {
			b'\0'
		} else {
			b'\n'
		}
	}

	fn _play<T: AsRef<Path> + ?Sized>(&mut self, expr: &Expression, start: &T) -> PlayResult<usize> {
		let mut num_matches = 0;

		for entry in std::fs::read_dir(start.as_ref())? {
			let mut ctx = PlayContext::new(self, entry?)?;
			let matched =
				dbg!(expr.run(&mut ctx, RunContext::Logical)).map_or(false, |x| x.is_truthy());
			let fileinfo = ctx.file_info;

			// Invert `matched` if given the `!` flag.
			let matched = if self.config.invert { !matched } else { matched };

			if matched {
				num_matches += 1;
			}

			if matched && !self.config.dont_print {
				let mut stdout = std::io::stdout().lock();
				stdout.write_all(&fileinfo.path.as_ref().as_os_str().to_raw_bytes());
				stdout.write(&[self.line_ending()]);
			}

			if fileinfo.file_type.is_dir() {
				let FileInfo { path, .. } = fileinfo;

				// todo: ignore_errors_subcommand
				match self._play(expr, &path) {
					Ok(amnt) => num_matches += amnt,
					Err(PlayError::Io(_)) if self.config.ignore_errors_os => {}
					Err(PlayError::Io(err))
						if self.config.ignore_errors_traversal
							&& err.kind() == IoErrorKind::PermissionDenied => {} // if let Err(err) = self._play(expr, &path) {
					Err(other) => return Err(other),
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

		if self.config.count {
			println!("{num_matches}");
		}

		Ok(())
	}
}
// ::fs::read_dir

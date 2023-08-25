#![allow(unused)]
use clap::error::ErrorKind;
use clap::CommandFactory;
use clap::Parser;
use findfile::ast::Expression;
use findfile::cli::{Args, Colour, IgnoreErrors, Prompt};
use findfile::parse::LexContext;
use findfile::play::program::{Config, Program};
use findfile::play::Env;
use findfile::play::{PlayError, PlayResult};
use std::process::ExitCode;

fn main() -> ExitCode {
	match _main() {
		Ok(status) => status,
		Err(err) => {
			eprintln!("uncaught error: {err:?}");
			ExitCode::FAILURE
		}
	}
}

fn _main() -> PlayResult<ExitCode> {
	let mut args = Args::parse();
	let source = if let Some(ref file) = args.file {
		std::fs::read_to_string(file)?
	} else {
		let mut expr = std::mem::take(&mut args.expression).unwrap_or_else(|| ".".into());
		while expr.ends_with(',') {
			if args.args.is_empty() {
				return Err(PlayError::Other("`,` at the end of an expression"));
			}

			expr.push_str(
				&args.args.remove(0).into_string().expect("todo: error for cli arg not a string"),
			);
		}
		expr
	};

	let env = Env::new(std::mem::take(&mut args.args));

	let mut program = Program::new(Config::from(&args), env);

	// Run all imported files
	for imported_file in args.import {
		program.run_file(&imported_file)?;
	}

	program.play_expr(&source)?;
	return Ok(ExitCode::SUCCESS);

	// Args::command()
	// 	.error(ErrorKind::ArgumentConflict, "Can't do relative and absolute version change").a
	// 	.exit()
	// 	;

	// Args::new().print_help();

	// let pattern = args.expression.unwrap_or_else(|| {
	// 	let f = args.file.as_ref().expect("todo: error for not supplying an expression or `-f`");
	// 	std::fs::read_to_string(f).expect("unable to read file contents")
	// });

	// let mut lctx = LexContext::new(&pattern);
	// let pattern = Expression::parse(&mut lctx, true, Default::default()).unwrap().unwrap();
	// let program = Program::new(vec![pattern]);
	// program.play().unwrap();
}

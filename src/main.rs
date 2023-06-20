#![allow(unused)]
use clap::error::ErrorKind;
use clap::CommandFactory;
use clap::Parser;
use findfile::ast::Expression;
use findfile::parse::LexContext;
use findfile::play::PlayResult;
use findfile::play::Program;
use findfile::PathRegex;
use std::process::ExitCode;

mod cli;
use cli::Args;

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
	let args = Args::parse();
	let mut program = Program::default();

	// Run all imported files
	for imported_file in &args.import {
		program.run_file(&imported_file);
	}

	// let expressions = fetch_expressions(&mut args);

	// Program::new(vec![]).play().unwrap();
	dbg!(&args);
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

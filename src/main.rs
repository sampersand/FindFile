#![allow(unused)]
use clap::error::ErrorKind;
use clap::CommandFactory;
use clap::Parser;
use findfile::ast::Expression;
use findfile::parse::LexContext;
use findfile::play::program::{Config, Program, When};
use findfile::play::PlayResult;
use findfile::PathRegex;
use std::process::ExitCode;

mod cli;
use self::cli::{Args, Colour, IgnoreErrors, Prompt};

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

	let mut program = Program::new(
		Config {
			cli: args.args,
			dont_print: args.dont_print || args.count,
			count: args.count,
			print0: args.print0,
			invert: args.invert,
			stable: args.stable,
			jobs: args.jobs,
			ignore_errors_traversal: args.ignored_errors.contains(&IgnoreErrors::Traversal),
			ignore_errors_os: args.ignored_errors.contains(&IgnoreErrors::Os),
			ignore_errors_subcommand: args.ignored_errors.contains(&IgnoreErrors::Subcommand),
			prompt: match (args.prompt, args.interactive, args.force) {
				(Prompt::Auto, true, false) => When::Always,
				(Prompt::Auto, false, true) => When::Never,
				(Prompt::Auto, false, false) => When::Auto,
				(Prompt::Always, false, false) => When::Always,
				(Prompt::Never, false, false) => When::Never,
				_ => unreachable!(),
			},
			color: match args.color {
				Colour::Auto => When::Auto,
				Colour::Always => When::Always,
				Colour::Never => When::Never,
			},
		}
		.check_for_unimplemented_features(),
	);

	// Run all imported files
	for imported_file in &args.import {
		program.run_file(imported_file)?;
	}

	let source = if let Some(file) = args.file {
		std::fs::read_to_string(&file)?
	} else {
		args.expression.unwrap_or_else(|| ".".into())
	};

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

#![allow(unused)]
use clap::error::ErrorKind;
use clap::CommandFactory;
use clap::Parser;
use findfile::ast::Expression;
use findfile::parse::LexContext;
use findfile::play::Program;
use findfile::PathRegex;

mod cli;
use cli::Args;

fn main() {
	let args = Args::parse();

	// Args::command()
	// 	.error(ErrorKind::ArgumentConflict, "Can't do relative and absolute version change").a
	// 	.exit()
	// 	;

	// Args::new().print_help();
	dbg!(&args);

	let pattern = args.expression.unwrap_or_else(|| {
		let f = args.file.as_ref().expect("todo: error for not supplying an expression or `-f`");
		std::fs::read_to_string(f).expect("unable to read file contents")
	});

	let mut lctx = LexContext::new(&pattern);
	let pattern = Expression::parse(&mut lctx, true, Default::default()).unwrap().unwrap();
	let program = Program::new(vec![pattern]);
	program.play().unwrap();
}

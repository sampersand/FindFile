#![allow(unused)]
use findfile::ast::Expression;
use findfile::parse::LexContext;
use findfile::play::Program;
use findfile::PathRegex;

use clap::Parser;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// The string containing the code
	expr: Option<String>,

	/// Positional arguments accessible via `$1`, `$2`, etc in the code.
	args: Vec<OsString>,

	/// Don't print out matching lines by default
	#[arg(short = 'n', long = "dont-print")]
	dont_print: bool,

	/// Print out how many things matched at the end; implies `-n`
	#[arg(short = 'c', long = "count")]
	count: bool,

	/// Emit `\0` instead of `\n` after each match
	#[arg(short = '0', long = "print0")]
	print0: bool,

	/// File to load code from; omit `EXPR`
	#[arg(short = 'f', long = "file")]
	file: Option<PathBuf>,
}

fn main() {
	let args = Args::parse();

	let expr = args.expr.unwrap_or_else(|| {
		let f = args.file.as_ref().expect("todo: error for not supplying an expression or `-f`");
		std::fs::read_to_string(f).expect("unable to read file contents")
	});

	let mut lctx = LexContext::new(&expr);
	let expr = Expression::parse(&mut lctx, true, Default::default()).unwrap().unwrap();
	let program = Program::new(vec![expr]);
	program.play().unwrap();

	// // 	// let mut lctx = LexContext::new("\"${PATH}\" 2 * 3 + 4");
	// // 	let mut args = std::env::args();
	// // 	let matchstr = args.skip(1).next().unwrap();

	// // 	let mut lctx = LexContext::new(&matchstr);
	// // 	let expr = Expression::parse(&mut lctx, true, Default::default()).unwrap().unwrap();

	// // 	let program = Program::new(vec![expr]);
	// // 	program.play().unwrap();

	// // 	// let regex = PathRegex::new("foo/*/*.txt").unwrap();
	// // 	// dbg!(regex);
	// // 	// dbg!(reg.matches("foo/bar/baz.txt"));
	// // }
}

// // fn main() {
// // 	// let mut lctx = LexContext::new("\"${PATH}\" 2 * 3 + 4");
// // 	let mut args = std::env::args();
// // 	let matchstr = args.skip(1).next().unwrap();

// // 	let mut lctx = LexContext::new(&matchstr);
// // 	let expr = Expression::parse(&mut lctx, true, Default::default()).unwrap().unwrap();

// // 	let program = Program::new(vec![expr]);
// // 	program.play().unwrap();

// // 	// let regex = PathRegex::new("foo/*/*.txt").unwrap();
// // 	// dbg!(regex);
// // 	// dbg!(reg.matches("foo/bar/baz.txt"));
// // }

// // // 	// let mut lctx = LexContext::new(r#""./a{"A"}$+1X", $2"#.as_ref());
// // // 	dbg!(Ast::parse(&mut lctx));
// // // 	// let mut lctx = LexContext::new(r"./a${foo}bc,d".as_ref());
// // // 	// let mut stream = Stream::new(r"${x > 10g --$3} && ~/ls\ -al".as_ref());

// // // 	// while let Ok(x) = lctx.next().unwrap() {
// // // 	// 	println!("{:?} {:?}", x, lctx.phase());
// // // 	// }
// // // 	// while let Some(next) = lex.next() {
// // // 	// 	println!("{:?}", next)
// // // 	// }
// // // }

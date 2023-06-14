#![allow(unused)]
use findfile::ast::Expression;
use findfile::parse::LexContext;
use findfile::play::Program;
use findfile::PathRegex;

fn main() {
	// let mut lctx = LexContext::new("\"${PATH}\" 2 * 3 + 4");
	let mut args = std::env::args();
	let matchstr = args.skip(1).next().unwrap();

	let mut lctx = LexContext::new(&matchstr);
	let expr = Expression::parse(&mut lctx, true, Default::default()).unwrap().unwrap();

	let program = Program::new(vec![expr]);

	program.play(".").unwrap();

	// let regex = PathRegex::new("foo/*/*.txt").unwrap();
	// dbg!(regex);
	// dbg!(reg.matches("foo/bar/baz.txt"));
}

// 	// let mut lctx = LexContext::new(r#""./a{"A"}$+1X", $2"#.as_ref());
// 	dbg!(Ast::parse(&mut lctx));
// 	// let mut lctx = LexContext::new(r"./a${foo}bc,d".as_ref());
// 	// let mut stream = Stream::new(r"${x > 10g --$3} && ~/ls\ -al".as_ref());

// 	// while let Ok(x) = lctx.next().unwrap() {
// 	// 	println!("{:?} {:?}", x, lctx.phase());
// 	// }
// 	// while let Some(next) = lex.next() {
// 	// 	println!("{:?}", next)
// 	// }
// }

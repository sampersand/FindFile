#![allow(unused)]
use findfile::parse::LexContext;
use findfile::parse::{ast::Expression, Ast};
use findfile::PathRegex;

fn main() {
	let mut lctx = LexContext::new("\"${PATH}\" 2 * 3 + 4");
	// let mut lctx = LexContext::new("1 * 2 + 3");
	// let mut lctx = LexContext::new("foo, bar(), a += (b; c)(!3, 4 > 5)");

	dbg!(Expression::parse(&mut lctx, true, Default::default()));

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

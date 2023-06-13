#![allow(unused)]
use findfile::parse::Ast;
use findfile::parse::LexContext;
use findfile::PathRegex;

fn main() {
	let regex = PathRegex::new("foo/*/*.txt").unwrap();
	dbg!(regex);
	// dbg!(reg.matches("foo/bar/baz.txt"));
}

// 	let mut lctx = LexContext::new(
// 		r#"
// size > $3

// "#
// 		.as_ref(),
// 	);
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

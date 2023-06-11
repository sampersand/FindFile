#![allow(unused)]
use findfile::parse::Ast;
use findfile::parse::LexContext;

fn main() {
	let mut lexctx = LexContext::new(
		r#"
size > $3

"#
		.as_ref(),
	);
	// let mut lexctx = LexContext::new(r#""./a{"A"}$+1X", $2"#.as_ref());
	dbg!(Ast::parse(&mut lexctx));
	// let mut lexctx = LexContext::new(r"./a${foo}bc,d".as_ref());
	// let mut stream = Stream::new(r"${x > 10g --$3} && ~/ls\ -al".as_ref());

	// while let Ok(x) = lexctx.next().unwrap() {
	// 	println!("{:?} {:?}", x, lexctx.phase());
	// }
	// while let Some(next) = lex.next() {
	// 	println!("{:?}", next)
	// }
}

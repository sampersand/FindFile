use findfile::parse::ParseContext;

fn main() {
	let mut ctx = ParseContext::new(r#""./a{"A"}$+1X""#.as_ref());
	// let mut ctx = ParseContext::new(r"./a${foo}bc,d".as_ref());
	// let mut stream = Stream::new(r"${x > 10g --$3} && ~/ls\ -al".as_ref());

	while let Some(x) = ctx.next().unwrap() {
		println!("{:?} {:?}", x, ctx.phase());
	}
	// while let Some(next) = lex.next() {
	// 	println!("{:?}", next)
	// }
}

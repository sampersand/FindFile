use findfile::{Stream, token::Token};

fn main() {
	let mut stream = Stream::new(r"./a${foo}bc,d".as_ref());
	// let mut stream = Stream::new(r"${x > 10g --$3} && ~/ls\ -al".as_ref());

	loop {
		let x = Token::next(&mut stream, &Default::default());
		if dbg!(x).is_err() {
			break;
		}
	}
	// while let Some(next) = lex.next() {
	// 	println!("{:?}", next)
	// }
}

use findfile::token::Token;
use logos::Logos;

fn main() {
	let mut lex = Token::lexer(r"${x > 10g --$3 }");

	while let Some(next) = lex.next() {
		println!("{:?}", next)
	}
}

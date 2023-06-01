use crate::token::Token;

#[derive(Debug)]
pub struct Parser<'a> {
	begin: &'a str,
	current: &'a str,
}

impl<'a> Parser<'a> {
	pub fn new(source: &'a str) -> Self {
		Self { begin: source, current: source }
	}

	pub fn lineno(&self) -> usize {
		let _ = (self.begin, self.current);
		todo!()
	}

	fn peek(&self) -> Option<char> {
		self.current.chars().next()
	}

	fn advance(&mut self) -> Option<char> {
		let mut chars = self.current.chars();
		let ret = chars.next()?;
		self.current = chars.as_str();
		Some(ret)
	}

	fn take_while(&mut self, cond: impl FnMut(char) -> bool) -> Option<&'a str> {
		let start = self.current;

		while self.peek().map_or(false, cond) {
			let _ = self.advance();
		}

		if self.current == start {
			return None;
		}


		// todo: clean up
		Some(start.strip_suffix(self.current).unwrap())
	}

	pub fn next(&mut self) -> Result<Option<Token<'a>>, String> {
		let Some(chr) = self.advance() else { return Ok(None) };

		match chr {
			'$' => {
				if let Some(number) = self.take_while(char::is_ascii_digit) {
					return Ok(Some(Token))
				}
				self.next
		}

		Ok(None)
	}
}

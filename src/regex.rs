use fancy_regex::Regex as FancyRegex;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct Regex(FancyRegex);

impl Eq for Regex {}
impl PartialEq for Regex {
	fn eq(&self, rhs: &Self) -> bool {
		self.0.as_str() == rhs.0.as_str()
	}
}

#[derive(Debug)]
pub struct RegexParseError(fancy_regex::Error);

impl Regex {
	pub fn new(source: &[u8], flags: &[u8]) -> Result<Self, RegexParseError> {
		let source = std::str::from_utf8(source).expect("todo: non-utf8 regexes");
		if flags.is_empty() {
			return FancyRegex::new(source).map_err(RegexParseError).map(Self);
		}
		let mut src = String::new();
		src.push_str("(?");
		src.push_str(std::str::from_utf8(flags).expect("todo: non-utf8 regexes"));
		src.push(')');
		src.push_str(source);
		FancyRegex::new(&src).map_err(RegexParseError).map(Self)
	}

	pub fn is_match(&self, source: &[u8]) -> bool {
		let Ok(source) = std::str::from_utf8(source) else {
			return false; // todo: handle non-`&[u8]` regexes
		};

		self.0.is_match(source).unwrap_or(false)
	}

	// pub fn capture_names(&self) -> impl Iterator<Item = &str> {
	// 	self.0.capture_names().skip(1).filter_map(std::convert::identity)
	// }

	// pub fn matches(&self, source: )

	pub fn source(&self) -> &[u8] {
		self.0.as_str().as_bytes()
	}
}

impl Display for Regex {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.write_str(std::str::from_utf8(self.source()).expect("todo: non-utf8 regexes"))
	}
}

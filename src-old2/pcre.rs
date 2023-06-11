#[derive(Debug, Clone, PartialEq)]
pub struct PcreRegex(u8);

#[derive(Debug, Clone, PartialEq)]
pub enum PcreRegexParseError {
	#[allow(unused)]
	Todo,
}

impl std::str::FromStr for PcreRegex {
	type Err = PcreRegexParseError;

	fn from_str(inp: &str) -> Result<Self, Self::Err> {
		let _ = inp;
		todo!()
	}
}

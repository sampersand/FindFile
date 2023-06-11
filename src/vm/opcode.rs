#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opcode {
	LoadConstant(usize),
	ConcatString,
}

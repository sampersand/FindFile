use crate::parse::ast::{Block, Expression};

#[derive(Debug, Clone, PartialEq)]
pub struct AstContext {
	before: Vec<Block>,
	after: Vec<Block>,
	conditions: Vec<Expression>,
}

impl AstContext {
	pub fn add_before(&mut self, block: Block) {
		self.before.push(block);
	}

	pub fn add_after(&mut self, block: Block) {
		self.after.push(block);
	}
}

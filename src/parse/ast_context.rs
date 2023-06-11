use crate::parse::ast::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
	// todo: this is the compiled output of a block
}

#[derive(Debug, Clone, PartialEq)]
pub struct AstContext {
	before: Vec<Block>,
	after: Vec<Block>,
	conditions: Vec<Expression>,
}

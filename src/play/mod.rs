mod context;
mod env;
mod error;

pub mod program;

pub use context::PlayContext;
pub use env::Env;
pub use error::{PlayError, PlayResult};
pub use program::Program;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunContext {
	Logical,

	#[default]
	Any,
}

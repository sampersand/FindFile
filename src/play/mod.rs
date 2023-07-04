mod context;
mod env;
mod error;
mod pathinfo;

pub mod program;

pub use context::PlayContext;
pub use env::Env;
pub use error::{PlayError, PlayResult};
pub use pathinfo::PathInfo;
pub use program::Program;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunContextOld {
	Logical,
	#[default]
	Any, // todo: rename to normal
}

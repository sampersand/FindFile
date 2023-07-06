pub mod block;
mod opcode;

pub use block::{Block, Builder};
pub use opcode::Opcode;

pub mod vm;
pub use vm::Vm;

#[derive(Debug)]
pub enum RunError {
	Io(std::io::Error),
}

pub type RunResult<T> = Result<T, RunError>;

impl From<std::io::Error> for RunError {
	fn from(err: std::io::Error) -> Self {
		Self::Io(err)
	}
}

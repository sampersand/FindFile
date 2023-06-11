#![allow(unused)]

// pub mod parser;
mod context;
pub mod filesize;
mod parse;
mod pathregex;
mod pcre;
mod posix;
mod stream;
pub mod token;

#[derive(Debug, Clone, PartialEq)]
pub struct Regex;

pub use context::Context;
pub use filesize::FileSize;
pub use pathregex::PathRegex;
pub use posix::PosixRegex;
pub use stream::Stream;

struct InterpolatedString;
fn parse_interop(string: &std::ffi::OsStr) -> InterpolatedString {
	let _ = string;
	todo!()
}

#![allow(unused)]

pub mod ast;
pub mod parse;
pub mod pathregex;
pub mod play;
mod value;
pub mod vm;
pub use pathregex::{PathParseError, PathRegex};
pub use value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct DateTime;

#[derive(Debug, Clone, PartialEq)]
pub struct FileSize;

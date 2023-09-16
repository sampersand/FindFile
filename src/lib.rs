// #![allow(unused)]
extern crate static_assertions as sa;

pub mod ast;
pub mod cli;
pub mod filesize;
pub mod parse;
// pub mod parse2;
pub mod pathglob;
pub mod play;
mod regex;
mod value;
pub mod vm;
pub use regex::Regex;
pub use value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct DateTime;
pub use pathglob::PathGlob;

pub use filesize::FileSize;

fn slice_contains(haystack: &[u8], needle: &[u8]) -> bool {
	haystack.windows(needle.len()).any(|c| c == needle)
}

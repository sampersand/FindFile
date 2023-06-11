#![allow(unused)]

use crate::context::Context;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display, Formatter};
use std::path::{Component, Components, Path, PathBuf, PrefixComponent};

#[derive(Debug, PartialEq, Clone)]
pub struct PathRegex {
	start: PathBuf,
	remainder: Vec<DirKind>,
}

pub enum ParseError {
	InvalidEscape,
	InvalidDollarSign,
}

#[derive(Debug, PartialEq, Clone)]
enum DirKind {
	Literal(PathBuf),
	Parent,
	Regex(ComponentRegex),
	AnyDirs,
}

#[derive(Debug, PartialEq, Clone)]
struct ComponentRegex(Vec<ComponentRegexPart>);

#[derive(Debug, PartialEq, Clone)]
enum ComponentRegexPart {
	Literal(OsString),
	AnyCharacter,
	ZeroOrMoreChars,
	CharRange(Vec<CharRange>),
}

type CharRange = std::ops::RangeInclusive<char>;

fn expand_source(source: &OsStr, context: &Context) -> Result<OsString, ParseError> {
	let mut src = OsString::new();
	let mut iter = source.to_str().expect("todo: handle non-str things").chars();

	while let Some(chr) = iter.next() {
		if chr != '$' {
			src.push((chr as char).to_string()); // todo: remove to_string
			continue;
		}

		// let next = chr.next().ok_or(ParseError::InvalidDollarSign)?;
		// let1 mut isalnum
	}

	Ok(src)
}
impl PathRegex {
	pub fn parse(source: &OsStr, context: &Context) -> Result<Self, ParseError> {
		let source = expand_source(source, context)?;
		let mut components = Path::new(&source).components();
		let mut dirs = Vec::<DirKind>::new();

		while let Some(component) = components.next() {
			match component {
				Component::Prefix(prefix) => todo!("add support in for windows prefix: {prefix:?}"),
				root @ Component::RootDir => {
					debug_assert!(dirs.is_empty(), "shoulda been taken care of my `components`");
					dirs.push(DirKind::Literal(root.as_os_str().into()));
				}
				Component::CurDir => {
					debug_assert!(dirs.is_empty(), "shoulda been taken care of my `components`");
					dirs.push(DirKind::Literal(std::env::current_dir().expect("cant get pwd")));
				}
				Component::ParentDir => {
					dirs.push(DirKind::Parent);
				}
				Component::Normal(path) => {
					let mut current = OsString::new();
					let mut chars = path.to_str().expect("path isnt os str").chars();

					// while let Some(chr) = chars.next() {
					// 	match chr {

					// 	}
					// }
					// parse_path(path, context);
				}
			}
		}

		todo!()
	}
}

// impl ComponentRegex {
// 	fn parse(source: &OsStr, context: &Context) -> Result<Self, ParseError> {
// 		debug_assert!(!source.is_empty());

// 		let mut current = OsString::new();
// 		let mut parts = Vec::new();
// 		let mut iter = source.to_str().expect("todo: non-ascii sources").chars().peekable();

// 		while let Some(chr) = iter.next() {
// 			match chr {
// 				'\\' => match iter.next().ok_or(ParseError::InvalidEscape)? {
// 					' ' | '$' | '{' => current.push(chr.to_string()), // todo: not `to-string`
// 					_ => return Err(ParseError::InvalidEscape)
// 				},
// 				'$' => current.push(fetch_env_var)
// 				_ => current.push(chr.to_string()), // TODO: not `to-string`
// 			}
// 		}

// 		Ok(Self(parts))
// 	}
// }

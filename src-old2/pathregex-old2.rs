#![allow(unused)]

use std::fmt::{self, Display, Formatter};
use crate::context::Context;
use std::ffi::{OsStr, OsString};
use std::path::{Component, Components, Path, PathBuf, PrefixComponent};

/// Represents a path regex, such as `./foo/*/bar`.
#[derive(Debug, Clone, PartialEq)]
pub struct PathRegex {
	start: PathBuf,
	dirs: Vec<DirKind>,
}

impl PathRegex {
	/// Parse out a [`PathRegex`], returning an error if it cant.
	pub fn parse(source: &OsStr, context: &Context) -> Result<Self, PathParseError> {
		let mut components = Path::new(source).components();

		let (start, next) = fetch_starting_directory(components)?;
		let mut dirs = vec![];

		for component in components {
			dirs.push(DirKind::parse(component, context)?);
		}

		Ok(Self { start, dirs })
	}

	pub fn matches(&self, path: &Path) -> bool {
		let _ = path;
		todo!()
	}

	pub fn display(&self) -> impl Display {
		struct Disp<'a>(&'a PathRegex);

		impl Display for Disp<'_> {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				for path in path
				Display::fmt(&self.0, f)
			}
		}
	}
}

#[cfg(test)]
mod test {
	#[test]
	fn the_default_is_pwd() {
		assert_eq!(PathRegex::default().display().to_string(), "/");
	}
}

// /// The point for the [`PathRegex`].
// #[derive(Debug, Clone, PartialEq)]
// enum StartDir {
// 	/// Start at the root folder, ie `/`.
// 	Root,

// 	/// Start at the current folder, ie `./`
// 	Current,

// 	/// Start at the parent folder, ie `../`
// 	Parent,

// 	// Start with `~/`
// 	Home,

// 	// Only on windows, `C:` or whatever
// 	Prefix(PrefixComponent<'static>),

// 	// Starts with `*` -- base of a file.
// 	Basename,
// }

// impl StartDir {
// 	fn parse(component: Component) -> Result<Self, PathParseError> {
// 		match component {
// 			Component::Prefix(_prefix) => todo!(),
// 			Component::RootDir => Ok(Self::Root),
// 			Component::CurDir => Ok(Self::Current),
// 			Component::ParentDir => Ok(Self::Parent),
// 			Component::Normal(_) => Err(PathParseError::StartsWithNormalDir),
// 		}
// 	}
// }

#[derive(Debug, Clone, PartialEq)]
enum DirKind {
	Literal(OsString),
	Parent,
	Regex(Vec<Part>),
	AnyDirs,
}

// fn parse_path_escape(source: &[u8]) -> Option<(char, &[u8])> {
// 	let first = *source.get(0)? as char;

// 	// If it's a custom escape for a path, then use that.
// 	if " *?[]".contains(first) {
// 		// TODO: add additional escapes.
// 		return Some((first, &source[1..]));
// 	}

// 	crate::parse::parse_string_escape(source)
// }

impl DirKind {
	fn parse(component: Component, context: &Context) -> Result<Self, PathParseError> {
		let _ = (component, context);
		todo!();
	}
}

// 		let path = match component {
// 			Component::Prefix(_) | Component::RootDir | Component::CurDir => {
// 				unreachable!("there are only emitted at the start")
// 			}
// 			Component::ParentDir => return Ok(Self::Parent), 
// 			Component::Normal(path) if path == "**" => return Ok(Self::AnyDirs),
// 			Component::Normal(path) => path,
// 		};

// 		let mut parts = Vec::new();
// 		let mut current = OsString::with_capacity(path.len());
// 		let mut chars = path.to_str().expect("todo: handle non-ascii paths").chars();

// 		while let Some(chr) = chars.next() {
// 			match chr {
// 				'\\' => {
// 					let Some((escaped, remainder)) = parse_path_escape(chars.as_str().as_bytes()) else {
// 						return Err(PathParseError::InvalidEscape);
// 					};

// 					current.push(&escaped.to_string());
// 					chars = std::str::from_utf8(remainder).expect("we just converted as bytes").chars();
// 				}
// 				'{' => todo!("string interop"),
// 				'$' => todo!("embed env var: {context:?}"),
// 				'/' => unreachable!(),
// 				other => current.push(chr.to_string()),
// 			}
// 		}

// 		// if parts.is_empty() {
// 		// 	Ok(DirKind::Literal(current))
// 		// } else {
// 		// 	Ok(DirKind::Regex(todo!()))
// 		// }

// 		// while let Some(part) = Part::parse() {
// 		// 	parts.push(current);
// 		// }

// 		// while let Some(chr) = chars.next() {
// 		// 	if chr == '\\' {
// 		// 		// TODO: don't use `.to_string()`
// 		// 		// todo: allow for escapes and `{`s?
// 		// 		let next = chars.next().ok_or(PathParseError::BackslashAtEndOfString);
// 		// 		// current.push(next&?.to_string(),);
// 		// 		continue;
// 		// 	}

// 		// 	if !"?*[]".contains(chr) {
// 		// 		// TODO: don't use `.to_string()`
// 		// 		current.push(&chr.to_string());
// 		// 		continue;
// 		// 	}

// 		// 	if !current.is_empty() {
// 		// 		parts.push(Part::Literal(std::mem::take(&mut current)));
// 		// 	}

// 		// 	match chr {
// 		// 		_ => todo!(), // '?' => parts.push(current),
// 		// 	}
// 		// }

// 		if parts.is_empty() {
// 			Ok(Self::Literal(current))
// 		} else {
// 			if !current.is_empty() {
// 				parts.push(Part::Literal(current));
// 			}

// 			Ok(Self::Regex(parts))
// 		}
// 	}
// }

#[derive(Debug, Clone, PartialEq)]
enum Part {
	Literal(OsString),
	AnyCharacter,
	ZeroOrMoreChars,
	CharRange(CharRange),
}

#[derive(Debug, Clone, PartialEq)]
struct CharRange;

#[derive(Debug, Clone, PartialEq)]
pub enum PathParseError {
	NothingGiven,
	InvalidEscape,
	StartsWithNormalDir,
}

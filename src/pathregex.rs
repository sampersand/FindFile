use os_str_bytes::{OsStrBytes, OsStringBytes};
use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct PathRegex {
	start: Option<PathStart>,
	dirs: Vec<DirKind>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PathStart {
	Root,
	Pwd,
	Parent,
	None,
}

#[derive(Debug, Clone, PartialEq)]
enum DirKind {
	Normal(OsString),
	Regex(Vec<Part>),
	Parent,
	AnyDirs,
}

#[derive(Debug, Clone, PartialEq)]
enum Part {
	Literal(OsString),
	ZeroOrMore,
	AnyCharcater,
	CharRange(Vec<u8>), // todo: do this better than just a char range.
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathParseError {
	NothingGiven,
}

impl PathStart {
	fn parse(component: Component<'_>) -> Option<Self> {
		match component {
			Component::Prefix(_) => todo!("handle prefixes on windows"),
			Component::RootDir => Some(PathStart::Root),
			Component::CurDir => Some(PathStart::Pwd),
			Component::ParentDir => Some(PathStart::Parent),
			Component::Normal(_) => None,
		}
	}
}

impl DirKind {
	fn parse(component: Component<'_>) -> Result<Self, PathParseError> {
		let source = match component {
			Component::Normal(x) if x == "**" => return Ok(Self::AnyDirs),
			Component::Normal(osstr) => osstr,
			Component::ParentDir => return Ok(Self::Parent),
			_ => unreachable!("others are only yielded at the start components"),
		};

		let mut parts = Vec::new();
		let mut literal = Vec::new();
		let raw = source.to_raw_bytes();
		let mut iter = raw.iter();

		for &byte in iter {
			if !b"?*[".contains(&byte) {
				literal.push(byte);
				continue;
			}

			if !literal.is_empty() {
				parts.push(Part::Literal(OsString::assert_from_raw_vec(std::mem::take(&mut literal))));
			}

			match byte {
				b'?' => parts.push(Part::AnyCharcater),
				b'*' => parts.push(Part::ZeroOrMore),
				b'[' => todo!("start a char match"),
				_ => unreachable!("already handled by the if statement above"),
			}
		}

		// if there's no regex, then dont do anything special
		if parts.is_empty() {
			return Ok(Self::Normal(OsString::assert_from_raw_vec(literal)));
		}

		if !literal.is_empty() {
			parts.push(Part::Literal(OsString::assert_from_raw_vec(literal)))
		}

		Ok(Self::Regex(parts))
	}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathMatch {
	/// The entire [`PathRegex`] matched.
	Yes,

	/// The given path was a directory which, while itself doesn't completely match, does
	/// match with a prefix portion of the [`PathRegex`]. Thus, you should continue searching.
	PartialDirMatch,

	/// The pattern doesn't match. Either the path supplied is a dir which none of its children
	/// will ever match (eg given `foo/bar/*.txt`, `foo/baz` will never have any children match),
	/// or it's a file which doesn't match.
	No,
}

impl PathRegex {
	pub fn new<T: AsRef<Path> + ?Sized>(source: &T) -> Result<Self, PathParseError> {
		let path = source.as_ref();
		let start = PathStart::parse(path.components().next().ok_or(PathParseError::NothingGiven)?);

		let mut components = path.components();
		if start.is_some() {
			let _ = components.next();
		}

		let dirs = components.map(DirKind::parse).collect::<Result<_, _>>()?;

		Ok(Self { start, dirs })
	}

	pub fn parse(
		begin: crate::parse::token::BeginPathKind,
		source: &OsStr,
	) -> Result<Self, PathParseError> {
		todo!()
	}

	// nb: this function needs a lot of work. it doesn't properly handle htings like
	// `self: "./foo/bar/baz" ` and `path: $PWD/foo/bar/baz` being equivalent...
	/*
	algorithm idea
		1. if `self` has a "starting point", make sure that both `self` and `path`
			begin at that starting point. if so, then just make `path` the remainder.

	*/
	pub fn matches<T: AsRef<Path> + ?Sized>(&self, path: &T) -> PathMatch {
		// 	let path = path.as_ref();
		// 	let mut components = path.components();

		// 	if let Some(start) = self.start {
		// 		let Some(head) = components.next() else {
		// 			return false;
		// 		};
		// 		if head != start {
		// 			return false;
		// 		}
		// 	}

		// 	for dirkind in &self.dirs {
		// 		let Some(component) = components.next() else {
		// 			return false;
		// 		};

		// 		match component {

		// Normal(OsString),
		// Regex(Vec<Part>),
		// Parent,
		// AnyDirs,
		// 		}
		// 	}
		// 	// for component in self.components().zip()
		// 	dbg!(self, path);
		// 	todo!();

		// for (src, given) in self.path.components().zip(path.components()) {
		// 	if src == given {
		// 		continue;
		// 	}

		// 	dbg!(src, given);
		// }
		// dbg!(self, path);
		todo!();
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn foo() {
		let path = Path::new("./foo/bar/baz");
		// let reg = PathRegex::new("./foo")
		// dbg!(PathRegex::new() {}.matches(path));

		// unimplemented!();
	}
}

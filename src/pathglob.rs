/*(* no whitespace is allowed between any of these thigns *)

path-literal := percent-path-literal | normal-path-literal ;
percent-path-literal (* `p` doesn't do interpolation in the body *)
 := '%' ('P' | 'p') (? any char ?) {(? anything but that char ?)} (? that char ?) ;
normal-path-literal := [prefix] {'/' directory} '/' [suffix] ;
prefix := start normal ;

start := (ALPHA | DIGIT | '.') | env-var | glob ;
glob := '*' | '?' | '[' character-range ']' ;

normal := start | interoplate | (? anything but end-path or '/' ?) ;
end-path := ',' | '(' | ')' | ';' | '&' | '|' | WHITESPACE ;

directory := '**' | normal {normal};
suffix := {normal};
*/
use os_str_bytes::{OsStrBytes, OsStringBytes};
use std::ffi::{OsStr, OsString};
use std::ops::RangeInclusive;
use std::path::{Component, Path, PathBuf};

// ff a/b
// ff ./a/b

#[derive(Debug, Clone, PartialEq)]
pub struct PathGlob {
	start: PathBuf,
	parts: Vec<PathPart>,
	is_dir: bool, // if it ends in `/`
}

impl PathGlob {
	pub fn begin_position(&self) -> PathBuf {
		let mut begin = self.start.clone();
		for part in &self.parts {
			if let PathPart::Normal(pathpart) = part {
				begin = begin.join(pathpart);
			} else {
				break;
			}
		}
		begin
	}

	// parses any path, doesn't care about special characters.
	pub fn parse(source: &Path) -> Result<Self, PathParseError> {
		assert_ne!(source, Path::new(""));
		let home = home::home_dir().ok_or(PathParseError::CantGetHomeDir)?;
		let mut components = source.components();

		let (start, iter, first_component) =
			match components.next().ok_or(PathParseError::NoPathGiven)? {
				Component::Prefix(prefix) => (prefix.as_os_str().into(), None, None),
				Component::RootDir => ("/".into(), None, None),
				Component::CurDir => (".".into(), None, None),
				Component::ParentDir => ("..".into(), None, None),
				Component::Normal(x) if x.to_raw_bytes() == b"~".as_slice() => {
					("/".into(), Some(home.components().skip(1)), None)
				}
				norm @ Component::Normal(_) => (".".into(), None, Some(norm)),
			};

		let parts = iter
			.into_iter()
			.flatten()
			.chain(first_component)
			.chain(components)
			.map(|comp| PathPart::parse(comp.as_os_str()))
			.collect::<Result<_, _>>()?;

		let is_dir = source.as_os_str().to_string_lossy().bytes().last()
			== Some(std::path::MAIN_SEPARATOR as u8);
		Ok(Self { start, parts, is_dir })
	}

	pub fn is_match(&self, given: &Path) -> bool {
		// note: `.is_dir()` follows symlinks, so in the future we might not want to.
		if self.is_dir && !given.is_dir() {
			return false;
		}

		let mut components = given.components();

		if components.next().map_or(true, |x| x.as_os_str() != self.start) {
			// todo: this might return false positives, b/c of `../` etc.
			return false;
		}

		// optimizationf or `**/foo.txt`
		// if self.parts[0] == PathPart::AnyDirs && self.parts.len() == 2 {
		// 	return match_globbed_parts(self.parts[1], self.parts.
		// }

		match_globbed_dirs(&self.parts, &components.map(Component::as_os_str).collect::<Vec<_>>())
	}
}

fn match_globbed_dirs(parts: &[PathPart], components: &[&OsStr]) -> bool {
	if parts.is_empty() || components.is_empty() {
		return parts.is_empty(); // && components.is_empty();
	}

	match parts[0] {
		PathPart::Normal(ref os) => {
			components[0] == os && match_globbed_dirs(&parts[1..], &components[1..])
		}
		PathPart::Globbed(ref glob) => {
			match_globbed_parts(&glob, &components[0].to_raw_bytes())
				&& match_globbed_dirs(&parts[1..], &components[1..])
		}
		PathPart::AnyDirs => (0..components.len())
			.rev()
			.map(|i| &components[i..])
			.any(|rest| match_globbed_dirs(&parts[1..], rest)),
	}
}

// return given.file_name().map_or(false, |last| {
// 	self.parts.last().expect("we should always have at least 1").is_match(last)
// });

#[derive(Debug)]
pub enum PathParseError {
	NoPathGiven,
	NotAPathStart(char),
	PrematureRangeEnd,
	CantGetHomeDir,
	InvalidEscape(char),
	PrematureAlternateEnd,
	CantGetPwd(std::io::Error),
}

#[derive(Debug, Clone, PartialEq)]
enum PathPart {
	AnyDirs,
	Normal(OsString),
	Globbed(Vec<Glob>),
}

impl PathPart {
	fn parse(source: &OsStr) -> Result<Self, PathParseError> {
		if source.to_str() == Some("**") {
			return Ok(Self::AnyDirs);
		}

		let mut parts = Vec::new();
		let mut bytes = source.to_raw_bytes();
		let mut iter = bytes.iter().copied();
		let mut current = Vec::new();

		while let Some(byte) = iter.next() {
			if !b"*?[{".contains(&byte) {
				current.push(byte);
				continue;
			}

			if !current.is_empty() {
				parts.push(Glob::Raw(std::mem::take(&mut current)));
			}

			parts.push(match byte {
				b'*' => Glob::ZeroOrMore,
				b'?' => Glob::SingleChar,
				b'{' => Glob::Alternative({
					// todo: support more than just literal text
					let mut globs = vec![];
					let mut current = vec![];
					loop {
						match iter.next().ok_or(PathParseError::PrematureAlternateEnd)? {
							b'}' => break,
							b'\\' => match iter.next().ok_or(PathParseError::PrematureAlternateEnd)? {
								c @ (b',' | b'}') => current.push(c),
								_ => todo!("other escapes"),
							},
							b',' => {
								assert!(!current.is_empty(), "todo error for when `,,` appears");
								globs.push(Glob::Raw(std::mem::take(&mut current)));
							}
							c => current.push(c),
						}
					}
					if !current.is_empty() {
						globs.push(Glob::Raw(current));
					}
					globs
				}),
				b'[' => Glob::Range(GlobRange::parse(&mut iter)?),
				_ => unreachable!(),
			});
		}

		if parts.is_empty() {
			debug_assert!(!current.is_empty());
			return Ok(Self::Normal(OsString::assert_from_raw_vec(current)));
		}

		if !current.is_empty() {
			parts.push(Glob::Raw(current));
		}

		Ok(Self::Globbed(parts))
	}

	fn is_match(&self, given: &OsStr) -> bool {
		match self {
			Self::AnyDirs => true,
			Self::Normal(lhs) => lhs == given,
			Self::Globbed(ref parts) => match_globbed_parts(parts, &given.to_raw_bytes()),
		}
	}
}

fn match_globbed_parts(parts: &[Glob], given: &[u8]) -> bool {
	if parts.is_empty() || given.is_empty() {
		return parts.is_empty() && given.is_empty();
	}

	match parts[0] {
		Glob::Alternative(ref alts) => alts.iter().any(|alt| match alt {
			Glob::Raw(ref raw) => given
				.strip_prefix(raw.as_slice())
				.map_or(false, |rest| match_globbed_parts(&parts[1..], rest)),
			_ => todo!("support other alternatives"),
		}),
		Glob::Raw(ref raw) => given
			.strip_prefix(raw.as_slice())
			.map_or(false, |rest| match_globbed_parts(&parts[1..], rest)),
		Glob::SingleChar => {
			given.get(1..).map_or(false, |rest| match_globbed_parts(&parts[1..], rest))
		}
		Glob::Range(ref range) => given.split_first().map_or(false, |(chr, rest)| {
			range.is_match(*chr as char) && match_globbed_parts(&parts[1..], rest)
		}),
		Glob::ZeroOrMore => (0..given.len())
			.rev()
			.map(|i| &given[i..])
			.any(|rest| match_globbed_parts(&parts[1..], rest)),
	}
}

#[derive(Debug, Clone, PartialEq)]
enum Glob {
	Raw(Vec<u8>),
	ZeroOrMore,
	SingleChar,
	Range(GlobRange),
	Alternative(Vec<Glob>),
}

#[derive(Default, Debug, Clone, PartialEq)]
struct GlobRange {
	negated: bool,
	solitary: Vec<char>,
	ranges: Vec<RangeInclusive<char>>,
}

impl GlobRange {
	fn is_match(&self, given: char) -> bool {
		self.negated != self._is_match(given)
	}

	fn _is_match(&self, given: char) -> bool {
		self.solitary.iter().any(|&c| c == given)
			|| self.ranges.iter().any(|rng| rng.contains(&given))
	}

	fn parse(iter: &mut impl Iterator<Item = u8>) -> Result<Self, PathParseError> {
		let mut byte = iter.next().ok_or(PathParseError::PrematureRangeEnd)?;
		let negated = byte == b'^';
		let mut solitary = Vec::new();
		let mut ranges = Vec::new();

		let mut iter = if negated { None } else { Some(byte) }.into_iter().chain(iter);

		loop {
			match iter.next().ok_or(PathParseError::PrematureRangeEnd)? as char {
				']' => return Ok(Self { negated, solitary, ranges }),
				'-' if solitary.is_empty() => todo!(),
				'-' if !solitary.is_empty() => {
					let begin = solitary.pop().unwrap();
					let end = iter.next().ok_or(PathParseError::PrematureRangeEnd)? as char;
					ranges.push(begin..=end);
				}

				'&' if solitary.last() == Some(&'&')
					|| ranges.last().map_or(false, |r| *r.end() == '&') =>
				{
					todo!("`&&` within char ranges")
				}

				'[' => todo!("posix-style ranges"),

				'\\' => match iter.next().ok_or(PathParseError::PrematureRangeEnd)? as char {
					c @ ('\\' | '[' | ']' | '-' | '^') => solitary.push(c),
					'W' | 'S' | 'D' => todo!("negated regexes (will be used with `&&`)"),
					'd' => {
						ranges.push('0'..='9');
					}
					'w' => {
						ranges.push('a'..='z');
						ranges.push('A'..='Z');
						ranges.push('0'..='9');
						solitary.push('_');
					}
					's' => {
						ranges.push('\x09'..='\x13'); // \t \n \v \f \r
						solitary.push(' ');
					}
					'x' | 'u' | 'U' => todo!("escape for `\\x`, `\\u`, and `\\U`."),
					'0' => solitary.push('\0'),
					'n' => solitary.push('\n'),
					'r' => solitary.push('\r'),
					't' => solitary.push('\t'),
					other => return Err(PathParseError::InvalidEscape(other)),
				},
				other => solitary.push(other),
			}
		}
	}
}

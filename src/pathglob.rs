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
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct PathGlob {
	pub prefix: Option<Component>,
	pub dirs: Vec<Directory>,
	pub suffix: Option<Component>,
}

// impl PathGlob {
// 	pub fn parse_from(mut source: &[u8]) -> Result<Self, PathParseError> {
// 		let prefix = match *source.get(0).ok_or(PathParseError::EmptySource)? as char {
// 			std::path::MAIN_SEPARATOR => None, // ie it's an absolute path
// 			c if PATH_START_CHARACTERS.contains(c) => todo!(),
// 			other => return Err(PathParseError::NotAPathStart(other)),
// 		};

// 		todo!()

// 		// if !PATH_START_CHARACTERS.containssource
// 	}
// }

#[derive(Debug)]
pub enum PathParseError {
	EmptySource,
	NotAPathStart(char),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Directory {
	AnyDirs,
	Normal(Component),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component(pub Vec<ComponentPart>);

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentPart {
	Raw(Vec<u8>),
	Glob(Glob),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Glob {
	Glob,
	SingleChar,
	Range(Vec<Range<char>>),
}

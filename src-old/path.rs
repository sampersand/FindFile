use std::borrow::Cow;
use std::path::Path as StdPath;

pub struct Path<'a> {
	path: Cow<'a, StdPath>,
}

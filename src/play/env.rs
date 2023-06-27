use std::ffi::{OsStr, OsString};

/// A type representing the outside environment.
///
/// Both `$environment` variables and `$123` cli args use this.
#[derive(Debug, Default)]
pub struct Env {
	cli: Vec<OsString>,
}

impl Env {
	/// Creates a new [`Env`] with the given command line arguments
	pub fn new(cli: Vec<OsString>) -> Self {
		Self { cli }
	}

	/// Gets the command line argument at position `position`.
	pub fn get_cli(&self, position: usize) -> Option<&OsStr> {
		self.cli.get(position).map(|x| &**x)
	}

	/// How many args were passed in the cli.
	pub fn cli_len(&self) -> usize {
		self.cli.len()
	}

	// Here's a fun little secret: It's actually more performant (when I tested) to fetch from the
	// environment each time than to store it in a hashmap!
	pub fn get_env(&self, name: &OsStr) -> Option<OsString> {
		std::env::var_os(name)
	}
}

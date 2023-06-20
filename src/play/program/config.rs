#[derive(Default, Debug, Clone)]
pub struct Config {
	/// Don't print out matching lines by default
	dont_print: bool,

	/// Print out how many things matched at the end; implies `-n`
	count: bool,

	/// Emit `\0` instead of `\n` after each match; disables `-n`
	print0: bool,
}

impl Config {
	pub fn dont_print(&self) -> bool {
		self.dont_print
	}

	pub fn set_dont_print(&mut self, value: bool) -> bool {
		self.dont_print = value;

		self.dont_print
	}
}

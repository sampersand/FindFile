use std::ffi::OsString;

#[derive(Default, Debug, Clone)]
pub struct Config {
	pub cli: Vec<OsString>,
	pub dont_print: bool,
	pub count: bool,
	pub print0: bool,
	pub invert: bool,
	pub stable: bool,
	pub jobs: Option<usize>,
	pub ignore_errors_traversal: bool,
	pub ignore_errors_os: bool,
	pub ignore_errors_subcommand: bool,
	pub prompt: When,
	pub color: When,
}

#[derive(Default, Debug, Clone)]
pub enum When {
	#[default]
	Auto,
	Always,
	Never,
}

// impl Config {
// 	pub fn dont_print(&self) -> bool {
// 		self.dont_print
// 	}

// 	pub fn set_dont_print(&mut self, value: bool) -> bool {
// 		self.dont_print = value;

// 		self.dont_print
// 	}
// 	/// Dangerous actions are when any of `mv, cp, rm, rmr, ln` would end up deleting or overwriting
// 	/// a file. Note that each of those actions have an "interactive" variant (`mvi`, `cpi`, etc)
// 	/// and a forceful variant (`mvf`, `cpf`, etc) which don't use this field.
// 	#[arg(
// 		short,
// 		long,
// 		hide_short_help = true,
// 		value_enum,
// 		value_name = "WHEN",
// 		default_value_t = Default::default(),
// 		overrides_with_all = ["interactive", "force"]
// 	)]
// 	pub prompt: Prompt,

// 	/// Always ask before doing anything destructive.
// 	///
// 	/// This is identical to `--prompt=always`.
// 	#[arg(short, long, overrides_with_all=["prompt", "force"])]
// 	pub interactive: bool,

// 	/// Never ask before doing anything destructive.
// 	///
// 	/// This is identical to `--prompt=never`.
// 	#[arg(short, long, overrides_with_all=["prompt", "interactive"])]
// 	pub force: bool,

// 	/// When to print colors
// 	#[arg(long, value_enum, value_name = "WHEN", default_value_t = Default::default())]
// 	pub color: Colour,
// }

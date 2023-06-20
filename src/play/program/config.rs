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

#[derive(Default, Debug, Clone, PartialEq)]
pub enum When {
	#[default]
	Auto,
	Always,
	Never,
}

impl Config {
	#[rustfmt::skip]
	pub fn check_for_unimplemented_features(self) -> Self {
		if self.stable { unimplemented!("unimplemented option: stable"); }
		if self.jobs.is_some() { unimplemented!("unimplemented option: jobs"); }
		if self.ignore_errors_subcommand { unimplemented!("unimplemented option: ignore subcommands"); }
		if self.prompt != Default::default() { unimplemented!("unimplemented option: prompt") ;}
		if self.color != Default::default() { unimplemented!("unimplemented option: color") ;}
		self
	}
}

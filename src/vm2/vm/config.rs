use crate::play::{PlayError, PlayResult};
use std::io::{self, Write};

#[derive(Default, Debug)]
pub struct Config {
	dont_print: bool,
	count: bool,
	print0: bool,
	invert: bool,
	stable: bool,
	jobs: usize,
	ignore_permission_errors: bool,
	ignore_os_errors: bool,
	ignore_subcommand_errors: bool,
	prompt: bool,
	colour: bool,
}

fn check_for_unimplemented_features(args: &crate::cli::Args) {
	macro_rules! check {
		($($t:ident)*) => {$(
			if args.$t != Default::default() {
				unimplemented!("unimplemented option: {}", stringify!($t));
			}
		)*};
	}
	check!(stable jobs prompt interactive force color);
	if args.ignored_errors.contains(&crate::cli::IgnoreErrors::Subcommand) {
		unimplemented!("unimplemented option: ignore subcommands");
	}
}

impl From<&crate::cli::Args> for Config {
	fn from(args: &crate::cli::Args) -> Self {
		use crate::cli::{Colour, IgnoreErrors, Prompt};
		check_for_unimplemented_features(args);

		Self {
			dont_print: args.dont_print || args.count,
			count: args.count,
			print0: args.print0,
			invert: args.invert,
			stable: args.stable,
			jobs: args.jobs.unwrap_or(1),
			ignore_os_errors: args.ignored_errors.contains(&IgnoreErrors::Os),
			ignore_permission_errors: args.ignored_errors.contains(&IgnoreErrors::Permission),
			ignore_subcommand_errors: args.ignored_errors.contains(&IgnoreErrors::Subcommand),
			prompt: match args.prompt {
				Prompt::Auto => atty::is(atty::Stream::Stdin),
				Prompt::Always => true,
				Prompt::Never => true,
			},
			colour: match args.color {
				Colour::Auto => atty::is(atty::Stream::Stdout),
				Colour::Always => true,
				Colour::Never => true,
			},
		}
	}
}

impl Config {
	#[must_use]
	pub fn should_print(&self) -> bool {
		!self.dont_print
	}

	#[must_use]
	pub fn is_counting(&self) -> bool {
		self.count
	}

	#[must_use]
	pub fn is_inverted(&self) -> bool {
		self.invert
	}

	#[must_use]
	pub fn is_stable(&self) -> bool {
		self.stable
	}

	#[must_use]
	pub fn should_prompt(&self) -> bool {
		self.prompt
	}

	#[must_use]
	pub fn should_colour(&self) -> bool {
		self.colour
	}

	#[must_use]
	pub fn ignore_subcommand_errors(&self) -> bool {
		self.ignore_subcommand_errors
	}

	/// how many jobs to spawn
	#[must_use]
	pub fn jobs(&self) -> usize {
		debug_assert_ne!(self.jobs, 0);
		self.jobs
	}

	pub fn write_line_ending(&self, mut out: impl Write) -> io::Result<()> {
		debug_assert!(!self.dont_print);

		if self.print0 {
			return out.write_all(&[b'\0']);
		}

		if cfg!(windows) {
			out.write_all(&[b'\r', b'\n'])
		} else {
			out.write_all(&[b'\n'])
		}
	}

	pub fn handle_error(&self, err: PlayError) -> PlayResult<()> {
		match err {
			PlayError::Io(_) if self.ignore_os_errors => Ok(()),
			PlayError::Io(err)
				if self.ignore_permission_errors && err.kind() == io::ErrorKind::PermissionDenied =>
			{
				Ok(())
			}
			other => Err(other),
		}
	}
}

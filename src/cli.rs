use clap::{Parser, ValueEnum};
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
	author,
	version,
	about,
	long_about = None,
	args_override_self = true,
	max_term_width = 98,
)]
pub struct Args {
	/// The search expression. [default: .]
	///
	/// The return value of the expression will be used in determining whether the path matches.
	#[arg(group = "input", value_name = "expression", allow_hyphen_values = true)]
	pub expression: Option<String>,

	/// File to load code from; omit `<expression>`
	///
	/// This acts the same as if you had done `ff "$(cat PATH)"`. This allows for easy reuse of
	/// queries.
	#[arg(long, group = "input", value_name = "PATH")]
	pub file: Option<PathBuf>,

	/// A list of files to import first.
	///
	/// These will only be executed once, at the beginning, before the expression is even parsed.
	/// This allows for easy reuse of custom user-defined functions. To import more than one file,
	/// simply supply `-I` multiple times.
	#[arg(short = 'I', long, short_alias = 'r')] // `-r` is because of ruby ;-p
	pub import: Vec<PathBuf>,

	/// Positional arguments accessible via `$1`, `$2`, etc in the code.
	///
	/// Any arguments after the expression are interpreted verbatim (except for a leading `--` which
	/// is stripped), and are accessible in the source code via `$1`, `$2`, etc.
	#[arg(trailing_var_arg = true)]
	pub args: Vec<OsString>,

	/// Invert matches: Match when the script is false.
	///
	/// This is equivalent to doing surrounding your script in `!(...)`.
	#[arg(short = 'v', long)]
	pub invert: bool,

	/// Ensure files are traversed in a stable manner; (implies `-j1`)
	///
	/// Note that this requires sorting all the files before traversing, so it'll be slower than a
	/// non-stable traversal. Since this requires stable traversal, only one job can be running.
	#[arg(short, long, overrides_with("jobs"))]
	pub stable: bool,

	/// Emit `\0` instead of `\n` after each match.
	///
	/// This is especially useful if you want to use findfile in conjunction
	/// with `xargs`. Do note, however, that findfile can do pretty much
	/// everything you'd need `xargs` for, such as:
	///   ff -n 'foo/**/*.old, rm(path)'
	#[arg(short = '0', long, verbatim_doc_comment, conflicts_with_all(["count", "dont_print"]))]
	pub print0: bool,

	/// Don't print out lines that match.
	///
	/// (Note: You can print arbitrary things from within an expression via
	/// `print()`.) dont-print is especially useful when used in conjunction
	/// with executable functions, such as `cp`:
	///    ff -n '*.txt, cp(path, "{path}.bak")'
	#[arg(short = 'n', long, verbatim_doc_comment, conflicts_with("print0"))]
	pub dont_print: bool,

	/// Print out how many things matched at the end; implies `-n`
	#[arg(short, long, conflicts_with("print0"))]
	pub count: bool,

	/// How many jobs to use. Defaults to the number of cores.
	///
	/// Note that each of these jobs will be completely independent of one another; as such, any
	/// variables assigned in a process won't be visible in others. (In the future, we may have
	/// ways around that.)
	///
	/// Many things conflict with a job count of more than 1. All the prompt commands (`mvi, cpi,
	/// rmi, rmri, lni, ok?`) currently require a single thread to use, as well as the
	/// `--interactive` flag. (In the future, this restriction may be relaxed). Additionally, the
	/// `--stable` is not compatible.
	#[arg(short, long, conflicts_with_all(["stable", "interactive"]))]
	pub jobs: Option<usize>,

	/// Which errors to ignore when running code.
	#[arg(short = 'e', long, value_enum, value_name = "ERRS")]
	pub ignored_errors: Vec<IgnoreErrors>,

	/// Run the expression only once, with no current file.
	#[arg(long)]
	pub run_once: bool,

	/// When to prompt for dangerous actions.
	///
	/// Dangerous actions are when any of `mv, cp, rm, rmr, ln` would end up deleting or overwriting
	/// a file. Note that each of those actions have an "interactive" variant (`mvi`, `cpi`, etc)
	/// and a forceful variant (`mvf`, `cpf`, etc) which don't use this field.
	#[arg(
		short,
		long,
		hide_short_help = true,
		value_enum,
		value_name = "WHEN",
		default_value_t = Default::default(),
		overrides_with_all = ["interactive", "force"]
	)]
	pub prompt: Prompt,

	/// Always ask before doing anything destructive.
	///
	/// This is identical to `--prompt=always`.
	#[arg(short, long, overrides_with_all=["prompt", "force"])]
	pub interactive: bool,

	/// Print out changes that'd happen but don't execute them. Implies `-n`
	#[arg(short, long)]
	pub dry: bool,

	/// Never ask before doing anything destructive.
	///
	/// This is identical to `--prompt=never`.
	#[arg(short, long, overrides_with_all=["prompt", "interactive"])]
	pub force: bool,

	/// When to print colors
	#[arg(long, value_enum, value_name = "WHEN", default_value_t = Default::default())]
	pub color: Colour,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum IgnoreErrors {
	/// Ignore permission errors for when interacting with dirs & files.
	#[value(aliases = ["p", "perms"])]
	Permission,

	/// Any os error that's not covered by permission.
	Os,

	/// If ignored, the stderr from subcommands is closed.
	#[value(alias = "s")]
	Subcommand,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Prompt {
	/// Prompt for destructive actions only when connected to a TTY.
	#[default]
	Auto,

	/// Always prompt for destructive actions.
	#[value(aliases = ["a", "true", "t", "y", "yes"])]
	Always,

	/// Never prompt for destructive actions.
	#[value(aliases = ["n", "false", "f", "no"])]
	Never,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Colour {
	/// Print out colors only when connected to a TTY.
	#[default]
	Auto,

	/// Always print out colors.
	#[value(aliases = ["a", "true", "t", "y", "yes"])]
	Always,

	/// Never print out colors.
	#[value(aliases = ["n", "false", "f", "no"])]
	Never,
}

use std::ffi::OsString;

#[derive(Debug, Default)]
pub struct Context {
	cli_args: Vec<OsString>,
	env_vars: (),
}

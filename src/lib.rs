#![allow(unused)]

#[cfg(windows)]
macro_rules! if_windows {
	($windows:expr, $_unix:expr) => {
		$windows
	};
}

#[cfg(not(windows))]
macro_rules! if_windows {
	($_windows:expr, $unix:expr) => {
		$unix
	};
}

pub mod parse;

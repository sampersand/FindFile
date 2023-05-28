// use std::fmt::{self, Display, Formatter};
// // use std::math::ops;

// #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct FileSize(u64);

// impl FileSize {
// 	pub const KILOBTYE: Self = Self(1000u64.pow(1));
// 	pub const MEGABYTE: Self = Self(1000u64.pow(2));
// 	pub const GIGABYTE: Self = Self(1000u64.pow(3));
// 	pub const TERABYTE: Self = Self(1000u64.pow(4));
// 	pub const PETABYTE: Self = Self(1000u64.pow(5));

// 	pub const KIBIBTYE: Self = Self(1024u64.pow(1));
// 	pub const MEBIBYTE: Self = Self(1024u64.pow(2));
// 	pub const GIBIBYTE: Self = Self(1024u64.pow(3));
// 	pub const TEBIBYTE: Self = Self(1024u64.pow(4));
// 	pub const PEBIBYTE: Self = Self(1024u64.pow(5));

// 	pub const fn from_bytes(num: u64) -> Self {
// 		Self(num)
// 	}

// 	pub const fn bytes(self) -> u64 {
// 		self.0
// 	}

// 	pub const fn mul(self, rhs: u64) -> Self {
// 		Self(self.0 * rhs)
// 	}
// }

// impl Display for FileSize {
// 	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
// 		if f.alternate() {
// 			return Display::fmt(&self.bytes(), f);
// 		}

// 		// Display::fmt(&self.0, f)
// 	}
// }

// fn main() {
// 	println!("{:?}",);
// }

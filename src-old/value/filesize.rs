use std::fmt::{self, Display, Formatter};
// use std::math::ops;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileSize(u64);

impl FileSize {
	pub const KILOBTYE: Self = Self(1000u64.pow(1));
	pub const MEGABYTE: Self = Self(1000u64.pow(2));
	pub const GIGABYTE: Self = Self(1000u64.pow(3));
	pub const TERABYTE: Self = Self(1000u64.pow(4));
	pub const PETABYTE: Self = Self(1000u64.pow(5));
	pub const EXABYTE: Self = Self(1000u64.pow(6));
	// modern computers can't handle anything larger than yottabytes

	pub const KIBIBTYE: Self = Self(1024u64.pow(1));
	pub const MEBIBYTE: Self = Self(1024u64.pow(2));
	pub const GIBIBYTE: Self = Self(1024u64.pow(3));
	pub const TEBIBYTE: Self = Self(1024u64.pow(4));
	pub const PEBIBYTE: Self = Self(1024u64.pow(5));
	pub const EXIBYTE: Self = Self(1024u64.pow(6));

	pub const fn from_bytes(num: u64) -> Self {
		Self(num)
	}

	pub const fn bytes(self) -> u64 {
		self.0
	}

	pub const fn mul(self, rhs: u64) -> Self {
		Self(self.0 * rhs)
	}

	pub fn fdiv(self, rhs: Self) -> f64 {
		(self.0 as f64) / (rhs.0 as f64)
	}
}

impl PartialEq<u64> for FileSize {
	fn eq(&self, rhs: &u64) -> bool {
		self.0 == *rhs
	}
}

impl PartialOrd<u64> for FileSize {
	fn partial_cmp(&self, rhs: &u64) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(rhs)
	}
}

impl Display for FileSize {
	// even tho we're using floating point numbers, there shouldn't be any errors until you hit
	// 1.0/f64::EPSILON, which will be in the yottabyte range.
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let base = if f.alternate() { Self::KIBIBTYE } else { Self::KILOBTYE };
		let mut pow = 1;

		while pow <= 6 && *self > base.0.pow(pow) {
			pow += 1;
		}

		// assert_eq!(self.0.ilog2() / 10, pow - 1);

		pow -= 1;
		let ratio = self.fdiv(FileSize(base.0.pow(pow)));
		let unit = match pow {
			0 => "", // nothing
			1 => {
				// `KiB` vs `kB`
				if f.alternate() {
					"K"
				} else {
					"k"
				}
			}
			2 => "M",
			3 => "G",
			4 => "T",
			5 => "P",
			6 => "E",
			_ => unreachable!("power too large? {pow}"),
		};

		write!(
			f,
			"{ratio:.*}{unit}{binary}B",
			std::cmp::min(f.precision().unwrap_or(0), 3 * (pow as usize)),
			binary = if f.alternate() { "i" } else { "" },
		)
	}
}

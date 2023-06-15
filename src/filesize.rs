use std::fmt::{self, Display, Formatter};

mod suffix;
pub use suffix::Suffix;

// Note: by using `f64` instead of `u64` as the backing type, we _technically_
// could have rounding errors. However, `f64` can successfully store `u52`s natively, so unless
// a filesize is larger than 4,503,599,627,370,495 bytes (4.5 quadrillion, ie 4.5e15) it doesn't
// really matter.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileSize {
	bytes: u64,
	precision: Option<u8>,
}

impl FileSize {
	pub fn new(amount: f64, suffix: Suffix, precision: Option<u8>) -> Option<Self> {
		let integer = (amount * (suffix as u64 as f64)) as u64;

		assert!(
			10u64.checked_pow(precision.unwrap_or(0) as _).unwrap() <= integer,
			"Todo: check for precision"
		);
		/* n.ilog10()-p >= 2?  */

		// dbg!(integer as u64 as f64, integer, integer as u64);
		// if (integer as u64 as f64) != integer {
		// 	return None;
		// }

		Some(Self::from_bytes(integer, precision))
	}

	pub const fn from_bytes(num: u64, precision: Option<u8>) -> Self {
		Self { bytes: num, precision }
	}

	pub const fn bytes(self) -> u64 {
		self.bytes
	}

	pub const fn precision(self) -> Option<u8> {
		self.precision
	}

	pub const fn mul(self, rhs: u64) -> Self {
		Self::from_bytes(self.bytes() * rhs, self.precision)
	}

	pub fn fuzzy_matches(self, tomatch: Self) -> bool {
		let precision = match self.precision {
			Some(0..=9) | None => return self == tomatch,
			Some(precision) => precision,
		};

		let bytes = self.bytes();
		let offset = 10u64.pow(bytes.ilog10() - (precision as u32) - 1);
		(bytes..bytes + offset).contains(&tomatch.bytes())
	}
}

impl PartialEq<u64> for FileSize {
	fn eq(&self, rhs: &u64) -> bool {
		self.bytes() == *rhs
	}
}

impl PartialOrd<u64> for FileSize {
	fn partial_cmp(&self, rhs: &u64) -> Option<std::cmp::Ordering> {
		self.bytes().partial_cmp(rhs)
	}
}

impl Display for FileSize {
	// even tho we're using floating point numbers, there shouldn't be any errors until you hit
	// 1.0/f64::EPSILON, which will be in the yottabyte range.
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let is_byte = f.alternate();
		let bytes = self.bytes();

		let base = if is_byte { Suffix::KibiByte } else { Suffix::KiloByte } as u64;
		let mut pow = 0;
		while pow < Suffix::MAX_EXPONENT && base.pow(pow + 1) < bytes {
			pow += 1;
		}

		let ratio = (bytes as f64) / (base.pow(pow) as f64);
		let unit = Suffix::unit_for(pow, is_byte);
		let binary = if is_byte { "i" } else { "" };

		write!(
			f,
			"{ratio:.*}{unit}{binary}B",
			std::cmp::min(f.precision().unwrap_or(0), 3 * (pow as usize))
		)
	}
}

// impl Display for FileSize {
// 	// even tho we're using floating point numbers, there shouldn't be any errors until you hit
// 	// 1.0/f64::EPSILON, which will be in the yottabyte range.
// 	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
// 		let base = if f.alternate() {
// 			Self::new(1.0, Suffix::KibiByte).unwrap()
// 		} else {
// 			Self::new(1.0, Suffix::KiloByte).unwrap()
// 		};

// 		let mut pow = 1;

// 		while pow <= 6 && *self > base.0.pow(pow) {
// 			pow += 1;
// 		}

// 		// assert_eq!(self.0.ilog2() / 10, pow - 1);

// 		pow -= 1;
// 		let ratio = self.fdiv(FileSize(base.0.pow(pow)));
// 		let unit = match pow {
// 			0 => "", // nothing
// 			1 => {
// 				// `KiB` vs `kB`
// 				if f.alternate() {
// 					"K"
// 				} else {
// 					"k"
// 				}
// 			}
// 			2 => "M",
// 			3 => "G",
// 			4 => "T",
// 			5 => "P",
// 			6 => "E",
// 			_ => unreachable!("power too large? {pow}"),
// 		};

// 		write!(
// 			f,
// 			"{ratio:.*}{unit}{binary}B",
// 			std::cmp::min(f.precision().unwrap_or(0), 3 * (pow as usize)),
// 			binary = if f.alternate() { "i" } else { "" },
// 		)
// 	}
// }

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn foo() {
		for x in 1..10_001 {
			let _ = FileSize::new(x as f64, Suffix::KiloByte, None).unwrap().to_string();
		}
		assert_eq!("12kB", FileSize::new(12.0, Suffix::KiloByte, None).unwrap().to_string());
		assert_eq!("1B", FileSize::new(1.0, Suffix::None, None).unwrap().to_string());
	}
}

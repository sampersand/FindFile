use std::fmt::{self, Display, Formatter};
// use std::math::ops;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileSize(u64);

impl FileSize {
	pub const KILOBYTE: Self = Self(1000u64.pow(1));
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

	pub fn from_float_and_suffix(float: f64, suffix: Suffix) -> Option<Self> {
		(float as u64).checked_mul(suffix as u64).map(Self)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u64)]
pub enum Suffix {
	None = 1000u64.pow(0),
	KiloByte = 1000u64.pow(1),
	MegaByte = 1000u64.pow(2),
	GigaByte = 1000u64.pow(3),
	TeraByte = 1000u64.pow(4),
	PetaByte = 1000u64.pow(5),
	ExaByte = 1000u64.pow(6),

	KibiByte = 1024u64.pow(1),
	MebiByte = 1024u64.pow(2),
	GibiByte = 1024u64.pow(3),
	TebiByte = 1024u64.pow(4),
	PebiByte = 1024u64.pow(5),
	ExiByte = 1024u64.pow(6),
}

impl Display for Suffix {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::None => f.write_str("B"),
			Self::KiloByte => f.write_str("kB"),
			Self::MegaByte => f.write_str("MB"),
			Self::GigaByte => f.write_str("GB"),
			Self::TeraByte => f.write_str("TB"),
			Self::PetaByte => f.write_str("PB"),
			Self::ExaByte => f.write_str("EB"),
			Self::KibiByte => f.write_str("KiB"),
			Self::MebiByte => f.write_str("MiB"),
			Self::GibiByte => f.write_str("GiB"),
			Self::TebiByte => f.write_str("TiB"),
			Self::PebiByte => f.write_str("PiB"),
			Self::ExiByte => f.write_str("EiB"),
		}
	}
}

impl Suffix {
	// lowercase allowed
	pub fn from_bytes(source: &[u8]) -> Option<Self> {
		let mut iter = source.iter();
		let power = iter.next()?;

		let (is_byte, bsufix_found) = match iter.next() {
			Some(b'i' | b'I') => (true, false),
			Some(b'b' | b'B') | None => (false, true),
			_ => return None,
		};

		if !bsufix_found && !matches!(iter.next(), Some(b'b' | b'B')) {
			return None;
		}

		match (power.to_ascii_lowercase(), is_byte) {
			(b'k', false) => Some(Self::KiloByte),
			(b'm', false) => Some(Self::MegaByte),
			(b'g', false) => Some(Self::GigaByte),
			(b't', false) => Some(Self::TeraByte),
			(b'p', false) => Some(Self::PetaByte),
			(b'e', false) => Some(Self::ExaByte),
			(b'k', true) => Some(Self::KibiByte),
			(b'm', true) => Some(Self::MebiByte),
			(b'g', true) => Some(Self::GibiByte),
			(b't', true) => Some(Self::TebiByte),
			(b'p', true) => Some(Self::PebiByte),
			(b'e', true) => Some(Self::ExiByte),
			(_, _) => None,
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum FileSizeParseError {
	NoIntPrefix,
	InvalidSuffix,
}

// impl std::str::FromStr for FileSize {
// 	type Err = FileSizeParseError;

// 	fn from_str(source: &str) -> Result<Self, Self::Err> {
// 		let (prefix, suffix) = source
// 			.find(|c: char| !c.is_ascii_digit())
// 			.map(|idx| source.split_at(idx))
// 			.ok_or(FileSizeParseError::NoIntPrefix)?;

// 		let value: u64 = prefix.parse().or(Err(FileSizeParseError::NoIntPrefix))?;

// 		// delete trailing `b` / `B` if given
// 		let Ok(base) = Suffix::from_str(suffix.to_str()) else {
// 			return Err(FileSizeParseError::InvalidSuffix);
// 		};

// 		let _ = (value, base);
// 		todo!()
// 		// Ok(base.mul(value))
// 	}
// }

impl Display for FileSize {
	// even tho we're using floating point numbers, there shouldn't be any errors until you hit
	// 1.0/f64::EPSILON, which will be in the yottabyte range.
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let base = if f.alternate() { Self::KIBIBTYE } else { Self::KILOBYTE };
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

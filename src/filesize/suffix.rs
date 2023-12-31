use std::fmt::{self, Display, Formatter};

// note: modern computers can't handle anything larger than an exabyte.
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
		f.write_str(self.to_str())
	}
}

impl Suffix {
	// note: modern computers can't handle anything larger than an exabyte.
	pub const MAX_EXPONENT: u32 = 6;

	/// Converts the suffix to its string representation.
	pub const fn to_str(self) -> &'static str {
		match self {
			Self::None => "B",
			Self::KiloByte => "kB",
			Self::MegaByte => "MB",
			Self::GigaByte => "GB",
			Self::TeraByte => "TB",
			Self::PetaByte => "PB",
			Self::ExaByte => "EB",
			Self::KibiByte => "KiB",
			Self::MebiByte => "MiB",
			Self::GibiByte => "GiB",
			Self::TebiByte => "TiB",
			Self::PebiByte => "PiB",
			Self::ExiByte => "EiB",
		}
	}

	pub fn unit_for(pow: u32, is_byte: bool) -> &'static str {
		match pow {
			0 => "",             // nothing
			1 if is_byte => "K", // SI defines kilobyte as `kB` but kibibyte as `KiB`/`KB`.
			1 if !is_byte => "k",
			2 => "M",
			3 => "G",
			4 => "T",
			5 => "P",
			6 => "E",
			_ => unreachable!("power too large? {pow}"),
		}
	}

	// lowercase allowed

	pub fn from_bytes(source: &[u8]) -> Option<Self> {
		let mut iter = source.iter().copied().peekable();
		let power = iter.next()?;

		if power == b'b' {
			return iter.next().is_none().then_some(Self::None);
		}

		let is_byte = iter.next_if_eq(&b'i').is_some() || iter.next_if_eq(&b'I').is_some();
		let _ = iter.next_if_eq(&b'b').is_some() || iter.next_if_eq(&b'B').is_some(); // ignore `b` and `B`

		if iter.next().is_some() {
			return None;
		};

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

			_ => None,
		}
	}
}

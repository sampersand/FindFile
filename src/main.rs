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
    pub const ZETTABYTE: Self = Self(1000u64.pow(7));
    pub const YOTTABYTE: Self = Self(1000u64.pow(8));
    // modern computers can't handle anything larger than yottabytes

    pub const KIBIBTYE: Self = Self(1024u64.pow(1));
    pub const MEBIBYTE: Self = Self(1024u64.pow(2));
    pub const GIBIBYTE: Self = Self(1024u64.pow(3));
    pub const TEBIBYTE: Self = Self(1024u64.pow(4));
    pub const PEBIBYTE: Self = Self(1024u64.pow(5));
    pub const EXIBYTE: Self = Self(1024u64.pow(6));
    pub const ZEBIBYTE: Self = Self(1024u64.pow(7));
    pub const YOBIBYTE: Self = Self(1024u64.pow(8));

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
        let mut pow: usize = 1;

        while *self >= base.0.pow(pow as u32) {
            pow += 1;
        }

        pow -= 1;
        let ratio = self.fdiv(FileSize(base.0.pow(pow as u32)));
        let unit = match pow {
            0 => "", // nothing
            1 => if f.alternate() { "K" } else { "k" }, // `KiB` vs `kB`
            2 => "M",
            3 => "G",
            4 => "T",
            5 => "P",
            6 => "E",
            7 => "Z",
            8 => "Y",
            _ => unreachable!("power too large? {pow}")
        };

        write!(f,
            "{ratio:.*}{unit}{binary}B",
            std::cmp::min(f.precision().unwrap_or(0), 3 * pow),
            binary = if f.alternate() { "i" } else { "" },
        )
    }
    // fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    //     let base = if f.alternate() { Self::KIBIBTYE } else { Self::KILOBTYE };
    //     let mut pow: usize = 1;

    //     while *self >= base.0.pow(pow as u32) {
    //         pow += 1;
    //     }

    //     pow -= 1;
    //     let ratio = self.fdiv(FileSize(base.0.pow(pow as u32)));
    //     let unit = match pow {
    //         0 => "", // nothing
    //         1 => if f.alternate() { "K" } else { "k" }, // `KiB` vs `kB`
    //         2 => "M",
    //         3 => "G",
    //         4 => "T",
    //         5 => "P",
    //         6 => "E",
    //         7 => "Z",
    //         8 => "Y",
    //         _ => unreachable!("power too large? {pow}")
    //     };

    //     write!(f,
    //         "{ratio:.*}{unit}{binary}B",
    //         std::cmp::min(f.precision().unwrap_or(0), 3 * pow),
    //         binary = if f.alternate() { "i" } else { "" },
    //     )
    // }
}

fn main() {
    println!("{:.9}", FileSize::from_bytes(1));
    println!("{:.9}", FileSize::from_bytes(12));
    println!("{:.9}", FileSize::from_bytes(123));
    println!("{:.9}", FileSize::from_bytes(1234));
    println!("{:.9}", FileSize::from_bytes(12345));
    println!("{:.9}", FileSize::from_bytes(123456));
    println!("{:.9}", FileSize::from_bytes(1234567));
    println!("{:.9}", FileSize::from_bytes(12345678));
    println!("{:.9}", FileSize::from_bytes(123456789));
    println!("{:.4}", FileSize::from_bytes(1234567890));
}

// Bitcoin Hashes Library
// Written in 2018 by
//   Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Hex encoding and decoding
//!

use std::fmt;
use {Error, Hash};

/// Trait for objects that can be serialized as hex strings
pub trait ToHex {
    /// Hex representation of the object
    fn to_hex(&self) -> String;
}

/// Trait for objects that can be deserialized from hex strings
pub trait FromHex: Sized {
    /// Produce an object from a hex string
    fn from_hex(s: &str) -> Result<Self, Error>;
}

impl<T: fmt::LowerHex> ToHex for T {
    /// Outputs the hash in hexadecimal form
    fn to_hex(&self) -> String {
        format!("{:x}", self)
    }
}

impl<T: Hash> FromHex for T {
    /// Parses a hex string as a hash object
    fn from_hex(s: &str) -> Result<Self, Error> {
        if s.len() != 2 * Self::LEN {
            return Err(Error::InvalidLength(2 * Self::LEN, s.len()));
        }

        let mut vec = Vec::<u8>::from_hex(s)?;
        if Self::DISPLAY_BACKWARD {
            vec.reverse();
        }
        Self::from_slice(&vec)
    }
}

/// Iterator over a hex-encoded string slice which decodes hex and yields bytes.
pub struct HexIterator<'a> {
    /// The slice whose first two characters will be decoded to yield the next byte
    pub sl: &'a str
}

impl<'a> Iterator for HexIterator<'a> {
    type Item = Result<u8, Error>;

    fn next(&mut self) -> Option<Result<u8, Error>> {
        if self.sl.len() % 2 == 1 {
            Some(Err(Error::OddLengthString(self.sl.len())))
        } else if self.sl.is_empty() {
            None
        } else {
            let (hi, lo) = {
                let mut iter = self.sl.chars();
                let hi = iter.next().unwrap();
                let lo = iter.next().unwrap();
                match (hi.to_digit(16), lo.to_digit(16)) {
                    (Some(hi), Some(lo)) => (hi, lo),
                    (None, _) => return Some(Err(Error::InvalidChar(hi))),
                    (_, None) => return Some(Err(Error::InvalidChar(lo))),
                }
            };
            let ret = (hi << 4) + lo;
            self.sl = &self.sl[2..];
            Some(Ok(ret as u8))
        }
    }
}

/// Output hex into an object implementing `fmt::Write`, which is usually more
/// efficient than going through a `String` using `ToHex`.
pub fn format_hex<T: fmt::Write>(data: &[u8], mut fmt: T) -> fmt::Result {
    for ch in data {
        write!(fmt, "{:02x}", *ch)?;
    }
    Ok(())
}

/// Output hex in reverse order; used for Sha256dHash whose standard hex encoding
/// has the bytes reversed.
pub fn format_hex_reverse<T: fmt::Write>(data: &[u8], mut fmt: T) -> fmt::Result {
    for ch in data.iter().rev() {
        write!(fmt, "{:02x}", *ch)?;
    }
    Ok(())
}

impl ToHex for [u8] {
    fn to_hex(&self) -> String {
        let mut ret = String::with_capacity(2 * self.len());
        format_hex(self, &mut ret).expect("format to string");
        ret
    }
}

impl FromHex for Vec<u8> {
    fn from_hex(s: &str) -> Result<Vec<u8>, Error> {
        if s.len() % 2 == 1 {
            return Err(Error::OddLengthString(s.len()));
        }

        let mut vec = Vec::with_capacity(s.len() / 2);
        let iter = HexIterator {
            sl: s
        };
        for byte in iter {
            vec.push(byte?);
        }
        Ok(vec)
    }
}

macro_rules! impl_fromhex_array {
    ($len:expr) => {
        impl FromHex for [u8; $len] {
            fn from_hex(s: &str) -> Result<[u8; $len], Error> {
                if s.len() == 2 * $len {
                    let mut ret = [0; $len];
                    let iter = HexIterator {
                        sl: s,
                    };
                    for (n, byte) in iter.enumerate() {
                        ret[n] = byte?;
                    }
                    Ok(ret)
                } else {
                    if s.len() % 2 == 1 {
                        Err(Error::OddLengthString(s.len()))
                    } else {
                        Err(Error::InvalidLength(2 * $len, s.len()))
                    }
                }
            }
        }
    }
}

impl_fromhex_array!(2);
impl_fromhex_array!(4);
impl_fromhex_array!(6);
impl_fromhex_array!(8);
impl_fromhex_array!(10);
impl_fromhex_array!(12);
impl_fromhex_array!(14);
impl_fromhex_array!(16);
impl_fromhex_array!(20);
impl_fromhex_array!(24);
impl_fromhex_array!(28);
impl_fromhex_array!(32);
impl_fromhex_array!(33);
impl_fromhex_array!(64);
impl_fromhex_array!(65);
impl_fromhex_array!(128);
impl_fromhex_array!(256);
impl_fromhex_array!(384);
impl_fromhex_array!(512);

#[cfg(test)]
mod tests {
    use super::{ToHex, FromHex};
    use Error;

    #[test]
    fn hex_roundtrip() {
        let expected = "0123456789abcdef";
        let expected_up = "0123456789ABCDEF";

        let parse: Vec<u8> = FromHex::from_hex(expected).expect("parse lowercase string");
        assert_eq!(parse, vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
        let ser = parse.to_hex();
        assert_eq!(ser, expected);

        let parse: Vec<u8> = FromHex::from_hex(expected_up).expect("parse uppercase string");
        assert_eq!(parse, vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
        let ser = parse.to_hex();
        assert_eq!(ser, expected);

        let parse: [u8; 8] = FromHex::from_hex(expected_up).expect("parse uppercase string");
        assert_eq!(parse, [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);
        let ser = parse.to_hex();
        assert_eq!(ser, expected);
    }

    #[test]
    fn hex_error() {
        let oddlen = "0123456789abcdef0";
        let badchar1 = "Z123456789abcdef";
        let badchar2 = "012Y456789abcdeb";
        let badchar3 = "«23456789abcdef";

        assert_eq!(
            Vec::<u8>::from_hex(oddlen),
            Err(Error::OddLengthString(17))
        );
        assert_eq!(
            <[u8; 4]>::from_hex(oddlen),
            Err(Error::OddLengthString(17))
        );
        assert_eq!(
            <[u8; 8]>::from_hex(oddlen),
            Err(Error::OddLengthString(17))
        );
        assert_eq!(
            Vec::<u8>::from_hex(badchar1),
            Err(Error::InvalidChar('Z'))
        );
        assert_eq!(
            Vec::<u8>::from_hex(badchar2),
            Err(Error::InvalidChar('Y'))
        );
        assert_eq!(
            Vec::<u8>::from_hex(badchar3),
            Err(Error::InvalidChar('«'))
        );
    }
}


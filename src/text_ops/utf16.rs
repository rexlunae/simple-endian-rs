//! UTF-16 helper types.

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::fmt;
use core::ops::Deref;

use crate::{BigEndian, LittleEndian, SpecificEndianOwned};

/// Errors returned when decoding UTF-16.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Utf16Error {
    /// UTF-16 input contained an invalid surrogate sequence.
    InvalidUtf16,
}

impl fmt::Display for Utf16Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Utf16Error::InvalidUtf16 => write!(f, "invalid UTF-16"),
        }
    }
}

#[cfg(any(feature = "io-std", feature = "io"))]
impl std::error::Error for Utf16Error {}

/// Borrowed UTF-16 code units (big-endian encoded `u16`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Utf16StrBE<'a>(pub &'a [BigEndian<u16>]);

impl<'a> From<&'a [BigEndian<u16>]> for Utf16StrBE<'a> {
    fn from(v: &'a [BigEndian<u16>]) -> Self {
        Self(v)
    }
}

impl AsRef<[BigEndian<u16>]> for Utf16StrBE<'_> {
    fn as_ref(&self) -> &[BigEndian<u16>] {
        self.0
    }
}

impl Deref for Utf16StrBE<'_> {
    type Target = [BigEndian<u16>];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Borrowed UTF-16 code units (little-endian encoded `u16`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Utf16StrLE<'a>(pub &'a [LittleEndian<u16>]);

impl<'a> From<&'a [LittleEndian<u16>]> for Utf16StrLE<'a> {
    fn from(v: &'a [LittleEndian<u16>]) -> Self {
        Self(v)
    }
}

impl AsRef<[LittleEndian<u16>]> for Utf16StrLE<'_> {
    fn as_ref(&self) -> &[LittleEndian<u16>] {
        self.0
    }
}

impl Deref for Utf16StrLE<'_> {
    type Target = [LittleEndian<u16>];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Owned UTF-16 code units (big-endian `u16`).
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Utf16StringBE(pub Vec<BigEndian<u16>>);

impl AsRef<[BigEndian<u16>]> for Utf16StringBE {
    fn as_ref(&self) -> &[BigEndian<u16>] {
        self.0.as_slice()
    }
}

impl Deref for Utf16StringBE {
    type Target = [BigEndian<u16>];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

/// Owned UTF-16 code units (little-endian `u16`).
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Utf16StringLE(pub Vec<LittleEndian<u16>>);

impl AsRef<[LittleEndian<u16>]> for Utf16StringLE {
    fn as_ref(&self) -> &[LittleEndian<u16>] {
        self.0.as_slice()
    }
}

impl Deref for Utf16StringLE {
    type Target = [LittleEndian<u16>];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl<'a> IntoIterator for Utf16StrBE<'a> {
    type Item = &'a BigEndian<u16>;
    type IntoIter = core::slice::Iter<'a, BigEndian<u16>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for Utf16StrLE<'a> {
    type Item = &'a LittleEndian<u16>;
    type IntoIter = core::slice::Iter<'a, LittleEndian<u16>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a Utf16StringBE {
    type Item = &'a BigEndian<u16>;
    type IntoIter = core::slice::Iter<'a, BigEndian<u16>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a Utf16StringLE {
    type Item = &'a LittleEndian<u16>;
    type IntoIter = core::slice::Iter<'a, LittleEndian<u16>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for Utf16StringBE {
    type Item = BigEndian<u16>;
    type IntoIter = alloc::vec::IntoIter<BigEndian<u16>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl IntoIterator for Utf16StringLE {
    type Item = LittleEndian<u16>;
    type IntoIter = alloc::vec::IntoIter<LittleEndian<u16>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Host-endian aliases.
#[cfg(target_endian = "little")]
pub type Utf16Str<'a> = Utf16StrLE<'a>;
#[cfg(target_endian = "big")]
pub type Utf16Str<'a> = Utf16StrBE<'a>;

#[cfg(target_endian = "little")]
pub type Utf16String = Utf16StringLE;
#[cfg(target_endian = "big")]
pub type Utf16String = Utf16StringBE;

impl From<&str> for Utf16StringBE {
    fn from(s: &str) -> Self {
        Self(s.encode_utf16().map(BigEndian::from).collect())
    }
}

impl From<&str> for Utf16StringLE {
    fn from(s: &str) -> Self {
        Self(s.encode_utf16().map(LittleEndian::from).collect())
    }
}

impl TryFrom<Utf16StrBE<'_>> for String {
    type Error = Utf16Error;

    fn try_from(v: Utf16StrBE<'_>) -> Result<Self, Self::Error> {
        decode_utf16(v.0.iter().map(|x| x.to_native()))
    }
}

impl TryFrom<Utf16StrLE<'_>> for String {
    type Error = Utf16Error;

    fn try_from(v: Utf16StrLE<'_>) -> Result<Self, Self::Error> {
        decode_utf16(v.0.iter().map(|x| x.to_native()))
    }
}

impl TryFrom<&Utf16StringBE> for String {
    type Error = Utf16Error;

    fn try_from(v: &Utf16StringBE) -> Result<Self, Self::Error> {
        String::try_from(Utf16StrBE::from(v.0.as_slice()))
    }
}

impl TryFrom<&Utf16StringLE> for String {
    type Error = Utf16Error;

    fn try_from(v: &Utf16StringLE) -> Result<Self, Self::Error> {
        String::try_from(Utf16StrLE::from(v.0.as_slice()))
    }
}

impl SpecificEndianOwned for Utf16StringBE {
    type Big = Utf16StringBE;
    type Little = Utf16StringLE;

    fn to_big_endian(&self) -> Self::Big {
        self.clone()
    }

    fn to_little_endian(&self) -> Self::Little {
        Utf16StringLE(self.0.iter().map(|x| LittleEndian::from(x.to_native())).collect())
    }

    fn from_big_endian(&self) -> Self::Big {
        self.clone()
    }

    fn from_little_endian(&self) -> Self::Little {
        self.to_little_endian()
    }
}

impl SpecificEndianOwned for Utf16StringLE {
    type Big = Utf16StringBE;
    type Little = Utf16StringLE;

    fn to_big_endian(&self) -> Self::Big {
        Utf16StringBE(self.0.iter().map(|x| BigEndian::from(x.to_native())).collect())
    }

    fn to_little_endian(&self) -> Self::Little {
        self.clone()
    }

    fn from_big_endian(&self) -> Self::Big {
        self.to_big_endian()
    }

    fn from_little_endian(&self) -> Self::Little {
        self.clone()
    }
}

fn decode_utf16<I: Iterator<Item = u16>>(it: I) -> Result<String, Utf16Error> {
    char::decode_utf16(it)
        .map(|r| r.map_err(|_| Utf16Error::InvalidUtf16))
        .collect()
}

//! UTF-32 helper types.

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::fmt;
use core::ops::Deref;

use crate::{BigEndian, LittleEndian, SpecificEndianOwned};

/// Errors returned when decoding UTF-32.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Utf32Error {
    /// UTF-32 input contained an invalid Unicode scalar value.
    InvalidUtf32,
}

impl fmt::Display for Utf32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Utf32Error::InvalidUtf32 => write!(f, "invalid UTF-32"),
        }
    }
}

#[cfg(any(feature = "io-std", feature = "io"))]
impl std::error::Error for Utf32Error {}

/// Borrowed UTF-32 code units (big-endian encoded `u32`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Utf32StrBE<'a>(pub &'a [BigEndian<u32>]);

impl<'a> From<&'a [BigEndian<u32>]> for Utf32StrBE<'a> {
    fn from(v: &'a [BigEndian<u32>]) -> Self {
        Self(v)
    }
}

impl AsRef<[BigEndian<u32>]> for Utf32StrBE<'_> {
    fn as_ref(&self) -> &[BigEndian<u32>] {
        self.0
    }
}

impl Deref for Utf32StrBE<'_> {
    type Target = [BigEndian<u32>];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Borrowed UTF-32 code units (little-endian encoded `u32`).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Utf32StrLE<'a>(pub &'a [LittleEndian<u32>]);

impl<'a> From<&'a [LittleEndian<u32>]> for Utf32StrLE<'a> {
    fn from(v: &'a [LittleEndian<u32>]) -> Self {
        Self(v)
    }
}

impl AsRef<[LittleEndian<u32>]> for Utf32StrLE<'_> {
    fn as_ref(&self) -> &[LittleEndian<u32>] {
        self.0
    }
}

impl Deref for Utf32StrLE<'_> {
    type Target = [LittleEndian<u32>];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Owned UTF-32 code units (big-endian `u32`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Utf32StringBE(pub Vec<BigEndian<u32>>);

impl AsRef<[BigEndian<u32>]> for Utf32StringBE {
    fn as_ref(&self) -> &[BigEndian<u32>] {
        self.0.as_slice()
    }
}

impl Deref for Utf32StringBE {
    type Target = [BigEndian<u32>];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

/// Owned UTF-32 code units (little-endian `u32`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Utf32StringLE(pub Vec<LittleEndian<u32>>);

impl AsRef<[LittleEndian<u32>]> for Utf32StringLE {
    fn as_ref(&self) -> &[LittleEndian<u32>] {
        self.0.as_slice()
    }
}

impl Deref for Utf32StringLE {
    type Target = [LittleEndian<u32>];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl<'a> IntoIterator for Utf32StrBE<'a> {
    type Item = &'a BigEndian<u32>;
    type IntoIter = core::slice::Iter<'a, BigEndian<u32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for Utf32StrLE<'a> {
    type Item = &'a LittleEndian<u32>;
    type IntoIter = core::slice::Iter<'a, LittleEndian<u32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a Utf32StringBE {
    type Item = &'a BigEndian<u32>;
    type IntoIter = core::slice::Iter<'a, BigEndian<u32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a Utf32StringLE {
    type Item = &'a LittleEndian<u32>;
    type IntoIter = core::slice::Iter<'a, LittleEndian<u32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for Utf32StringBE {
    type Item = BigEndian<u32>;
    type IntoIter = alloc::vec::IntoIter<BigEndian<u32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl IntoIterator for Utf32StringLE {
    type Item = LittleEndian<u32>;
    type IntoIter = alloc::vec::IntoIter<LittleEndian<u32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Host-endian aliases.
#[cfg(target_endian = "little")]
pub type Utf32Str<'a> = Utf32StrLE<'a>;
#[cfg(target_endian = "big")]
pub type Utf32Str<'a> = Utf32StrBE<'a>;

#[cfg(target_endian = "little")]
pub type Utf32String = Utf32StringLE;
#[cfg(target_endian = "big")]
pub type Utf32String = Utf32StringBE;

impl From<&str> for Utf32StringBE {
    fn from(s: &str) -> Self {
        Self(s.chars().map(|c| BigEndian::from(c as u32)).collect())
    }
}

impl From<&str> for Utf32StringLE {
    fn from(s: &str) -> Self {
        Self(s.chars().map(|c| LittleEndian::from(c as u32)).collect())
    }
}

impl TryFrom<Utf32StrBE<'_>> for String {
    type Error = Utf32Error;

    fn try_from(v: Utf32StrBE<'_>) -> Result<Self, Self::Error> {
        decode_utf32(v.0.iter().map(|x| x.to_native()))
    }
}

impl TryFrom<Utf32StrLE<'_>> for String {
    type Error = Utf32Error;

    fn try_from(v: Utf32StrLE<'_>) -> Result<Self, Self::Error> {
        decode_utf32(v.0.iter().map(|x| x.to_native()))
    }
}

impl TryFrom<&Utf32StringBE> for String {
    type Error = Utf32Error;

    fn try_from(v: &Utf32StringBE) -> Result<Self, Self::Error> {
        String::try_from(Utf32StrBE::from(v.0.as_slice()))
    }
}

impl TryFrom<&Utf32StringLE> for String {
    type Error = Utf32Error;

    fn try_from(v: &Utf32StringLE) -> Result<Self, Self::Error> {
        String::try_from(Utf32StrLE::from(v.0.as_slice()))
    }
}

impl SpecificEndianOwned for Utf32StringBE {
    type Big = Utf32StringBE;
    type Little = Utf32StringLE;

    fn to_big_endian(&self) -> Self::Big {
        self.clone()
    }

    fn to_little_endian(&self) -> Self::Little {
        Utf32StringLE(self.0.iter().map(|x| LittleEndian::from(x.to_native())).collect())
    }

    fn from_big_endian(&self) -> Self::Big {
        self.clone()
    }

    fn from_little_endian(&self) -> Self::Little {
        self.to_little_endian()
    }
}

impl SpecificEndianOwned for Utf32StringLE {
    type Big = Utf32StringBE;
    type Little = Utf32StringLE;

    fn to_big_endian(&self) -> Self::Big {
        Utf32StringBE(self.0.iter().map(|x| BigEndian::from(x.to_native())).collect())
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

fn decode_utf32<I: Iterator<Item = u32>>(it: I) -> Result<String, Utf32Error> {
    let mut out = String::new();
    for cu in it {
        match char::from_u32(cu) {
            Some(c) => out.push(c),
            None => return Err(Utf32Error::InvalidUtf32),
        }
    }
    Ok(out)
}

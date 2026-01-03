//! Fixed-size, inline text helpers.

extern crate alloc;

use alloc::string::String;
use core::{fmt, mem::MaybeUninit};

#[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
mod utf16;

#[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
mod utf32;

#[cfg(all(feature = "text_fixed", feature = "text_utf8"))]
pub mod utf8;

#[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
pub use utf16::*;

#[cfg(all(feature = "text_fixed", feature = "text_utf8"))]
pub use utf8::*;

#[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
pub use utf32::*;

/// Error returned when converting into a fixed-codepoint string fails.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixedTextError {
    /// The input contains fewer or more than `N` Unicode scalar values.
    WrongCodepointCount { expected: usize, found: usize },
}

#[cfg(all(feature = "text_fixed", feature = "text_utf8"))]
impl From<FixedUtf8Error> for FixedTextError {
    fn from(e: FixedUtf8Error) -> Self {
        match e {
            FixedUtf8Error::TooManyBytes { max, found } => FixedTextError::WrongCodepointCount {
                expected: max,
                found,
            },
            FixedUtf8Error::InvalidUtf8 => FixedTextError::WrongCodepointCount {
                expected: 0,
                found: 0,
            },
        }
    }
}

#[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
impl From<FixedUtf16Error> for FixedTextError {
    fn from(e: FixedUtf16Error) -> Self {
        match e {
            FixedUtf16Error::WrongCodeUnitCount { expected, found } => {
                FixedTextError::WrongCodepointCount { expected, found }
            }
            FixedUtf16Error::InvalidUtf16 => FixedTextError::WrongCodepointCount {
                expected: 0,
                found: 0,
            },
        }
    }
}

#[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
impl From<FixedUtf32Error> for FixedTextError {
    fn from(e: FixedUtf32Error) -> Self {
        match e {
            FixedUtf32Error::WrongCodeUnitCount { expected, found } => {
                FixedTextError::WrongCodepointCount { expected, found }
            }
            FixedUtf32Error::InvalidUtf32 => FixedTextError::WrongCodepointCount {
                expected: 0,
                found: 0,
            },
        }
    }
}

impl fmt::Display for FixedTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixedTextError::WrongCodepointCount { expected, found } => {
                write!(
                    f,
                    "wrong number of codepoints (expected {expected}, found {found})"
                )
            }
        }
    }
}

#[cfg(any(feature = "io-std", feature = "io"))]
impl std::error::Error for FixedTextError {}

/// A fixed-length string of exactly `N` Unicode scalar values (codepoints).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedCodepointString<const N: usize> {
    chars: [char; N],
}

/// A borrowed view of exactly `N` Unicode scalar values.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedCodepointStr<'a, const N: usize>(pub &'a [char; N]);

impl<const N: usize> FixedCodepointString<N> {
    /// Returns the stored codepoints.
    pub const fn as_chars(&self) -> &[char; N] {
        &self.chars
    }

    /// Returns a borrowed view of this fixed-codepoint string.
    pub const fn as_str_view(&self) -> FixedCodepointStr<'_, N> {
        FixedCodepointStr(&self.chars)
    }

    /// Builds a `String` by collecting the stored codepoints.
    pub fn to_string_lossless(&self) -> String {
        self.chars.iter().collect()
    }
}

impl<const N: usize> fmt::Display for FixedCodepointString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.chars {
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

impl<const N: usize> AsRef<[char]> for FixedCodepointString<N> {
    fn as_ref(&self) -> &[char] {
        self.chars.as_slice()
    }
}

impl<'a, const N: usize> FixedCodepointStr<'a, N> {
    pub const fn as_chars(&self) -> &'a [char; N] {
        self.0
    }
}

impl<'a, const N: usize> From<&'a [char; N]> for FixedCodepointStr<'a, N> {
    fn from(v: &'a [char; N]) -> Self {
        Self(v)
    }
}

impl<'a, const N: usize> From<&'a FixedCodepointString<N>> for FixedCodepointStr<'a, N> {
    fn from(v: &'a FixedCodepointString<N>) -> Self {
        v.as_str_view()
    }
}

impl<'a, const N: usize> fmt::Display for FixedCodepointStr<'a, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in *self.0 {
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

impl<'a, const N: usize> AsRef<[char]> for FixedCodepointStr<'a, N> {
    fn as_ref(&self) -> &[char] {
        self.0.as_slice()
    }
}

impl<const N: usize> From<&FixedCodepointString<N>> for String {
    fn from(v: &FixedCodepointString<N>) -> Self {
        v.chars.iter().collect()
    }
}

#[cfg(all(test, feature = "text_utf16", feature = "text_utf32"))]
mod tests {
    use super::*;
    use crate::{BigEndian, LittleEndian};

    #[test]
    fn fixed_utf16_code_units_alias_is_host_endian() {
        #[cfg(target_endian = "little")]
        {
            let _v: FixedUtf16CodeUnits<2> = FixedUtf16LeCodeUnits::<2>::try_from("hi").unwrap();
        }

        #[cfg(target_endian = "big")]
        {
            let _v: FixedUtf16CodeUnits<2> = FixedUtf16BeCodeUnits::<2>::try_from("hi").unwrap();
        }
    }

    #[test]
    fn fixed_utf32_code_units_alias_is_host_endian() {
        #[cfg(target_endian = "little")]
        {
            let _v: FixedUtf32CodeUnits<2> = FixedUtf32LeCodeUnits::<2>::try_from("hi").unwrap();
        }

        #[cfg(target_endian = "big")]
        {
            let _v: FixedUtf32CodeUnits<2> = FixedUtf32BeCodeUnits::<2>::try_from("hi").unwrap();
        }
    }

    #[test]
    fn fixed_code_units_can_be_wrapped_in_endian_types() {
        // This test mostly exists to ensure the trait bounds are satisfied:
        // `LittleEndian<T>` requires `T: SpecificEndian<T>`.
        const N: usize = 2;

        let le16: LittleEndian<FixedUtf16CodeUnits<N>> = "hi".try_into().unwrap();
        let be16: BigEndian<FixedUtf16CodeUnits<N>> = "hi".try_into().unwrap();
        let _native16: FixedUtf16CodeUnits<N> = le16.to_native();
        let _native16b: FixedUtf16CodeUnits<N> = be16.to_native();

        let le32: LittleEndian<FixedUtf32CodeUnits<N>> = "hi".try_into().unwrap();
        let be32: BigEndian<FixedUtf32CodeUnits<N>> = "hi".try_into().unwrap();
        let _native32: FixedUtf32CodeUnits<N> = le32.to_native();
        let _native32b: FixedUtf32CodeUnits<N> = be32.to_native();
    }
}

impl<const N: usize> From<FixedCodepointStr<'_, N>> for String {
    fn from(v: FixedCodepointStr<'_, N>) -> Self {
        v.0.iter().collect()
    }
}

impl<const N: usize> From<FixedCodepointStr<'_, N>> for FixedCodepointString<N> {
    fn from(v: FixedCodepointStr<'_, N>) -> Self {
        Self { chars: *v.0 }
    }
}

impl<const N: usize> From<FixedCodepointStr<'_, N>> for [char; N] {
    fn from(v: FixedCodepointStr<'_, N>) -> Self {
        *v.0
    }
}

impl<const N: usize> TryFrom<&str> for FixedCodepointString<N> {
    type Error = FixedTextError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut out: [MaybeUninit<char>; N] = [MaybeUninit::uninit(); N];
        let mut count = 0usize;

        for c in s.chars() {
            if count >= N {
                return Err(FixedTextError::WrongCodepointCount {
                    expected: N,
                    found: count + 1,
                });
            }
            out[count].write(c);
            count += 1;
        }

        if count != N {
            return Err(FixedTextError::WrongCodepointCount {
                expected: N,
                found: count,
            });
        }

        // Safety: we wrote exactly N chars.
        let chars = unsafe { out.map(|m| m.assume_init()) };
        Ok(Self { chars })
    }
}

impl<const N: usize> TryFrom<String> for FixedCodepointString<N> {
    type Error = FixedTextError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

/// Errors for fixed UTF-16 code-unit storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixedUtf16Error {
    /// Input had the wrong number of UTF-16 code units.
    WrongCodeUnitCount { expected: usize, found: usize },
    /// Input code units are not valid UTF-16.
    InvalidUtf16,
}

// Convenience: allow `"...".try_into()` directly into endian-tagged fixed buffers.
//
// This keeps the README examples clean, and is consistent with the rest of this crate's
// ergonomics where `BigEndian<T>`/`LittleEndian<T>` are easy to construct.
#[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
impl<const N: usize> TryFrom<&str> for crate::LittleEndian<FixedUtf16CodeUnits<N>> {
    type Error = FixedUtf16Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let native = FixedUtf16CodeUnits::<N>::try_from(s)?;
        Ok(crate::LittleEndian::from(native))
    }
}

#[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
impl<const N: usize> TryFrom<String> for crate::LittleEndian<FixedUtf16CodeUnits<N>> {
    type Error = FixedUtf16Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

#[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
impl<const N: usize> TryFrom<&str> for crate::BigEndian<FixedUtf16CodeUnits<N>> {
    type Error = FixedUtf16Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let native = FixedUtf16CodeUnits::<N>::try_from(s)?;
        Ok(crate::BigEndian::from(native))
    }
}

#[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
impl<const N: usize> TryFrom<String> for crate::BigEndian<FixedUtf16CodeUnits<N>> {
    type Error = FixedUtf16Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

#[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
impl<const N: usize> TryFrom<&str> for crate::LittleEndian<FixedUtf32CodeUnits<N>> {
    type Error = FixedUtf32Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let native = FixedUtf32CodeUnits::<N>::try_from(s)?;
        Ok(crate::LittleEndian::from(native))
    }
}

#[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
impl<const N: usize> TryFrom<String> for crate::LittleEndian<FixedUtf32CodeUnits<N>> {
    type Error = FixedUtf32Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

#[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
impl<const N: usize> TryFrom<&str> for crate::BigEndian<FixedUtf32CodeUnits<N>> {
    type Error = FixedUtf32Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let native = FixedUtf32CodeUnits::<N>::try_from(s)?;
        Ok(crate::BigEndian::from(native))
    }
}

#[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
impl<const N: usize> TryFrom<String> for crate::BigEndian<FixedUtf32CodeUnits<N>> {
    type Error = FixedUtf32Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl fmt::Display for FixedUtf16Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixedUtf16Error::WrongCodeUnitCount { expected, found } => write!(
                f,
                "wrong number of UTF-16 code units (expected {expected}, found {found})"
            ),
            FixedUtf16Error::InvalidUtf16 => write!(f, "invalid UTF-16"),
        }
    }
}

#[cfg(any(feature = "io-std", feature = "io"))]
impl std::error::Error for FixedUtf16Error {}

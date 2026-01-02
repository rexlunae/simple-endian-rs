//! Fixed-size UTF-8 byte storage and common conventions.
//!
//! These types parallel the fixed UTF-16/UTF-32 helpers, but operate on UTF-8 bytes.
//! There is no endianness concept for UTF-8: the wire format is just `[u8; N]`.

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::fmt;

/// Errors for fixed UTF-8 byte storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixedUtf8Error {
    /// Input had more than `N` bytes (after encoding), so it can’t fit.
    TooManyBytes { max: usize, found: usize },
    /// Input bytes are not valid UTF-8.
    InvalidUtf8,
}

impl fmt::Display for FixedUtf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixedUtf8Error::TooManyBytes { max, found } => {
                write!(f, "UTF-8 string too long (max {max} bytes, found {found})")
            }
            FixedUtf8Error::InvalidUtf8 => write!(f, "invalid UTF-8"),
        }
    }
}

/// Inline, fixed-size UTF-8 bytes.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf8Bytes<const N: usize> {
    pub(crate) bytes: [u8; N],
}

/// A borrowed reference to exactly `N` UTF-8 bytes.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf8BytesRef<'a, const N: usize>(pub &'a [u8; N]);

impl<const N: usize> FixedUtf8Bytes<N> {
    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.bytes
    }
}

impl<'a, const N: usize> FixedUtf8BytesRef<'a, N> {
    pub const fn as_bytes(&self) -> &'a [u8; N] {
        self.0
    }
}

impl<const N: usize> From<[u8; N]> for FixedUtf8Bytes<N> {
    fn from(bytes: [u8; N]) -> Self {
        Self { bytes }
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for FixedUtf8BytesRef<'a, N> {
    fn from(v: &'a [u8; N]) -> Self {
        Self(v)
    }
}

/// NUL-padded fixed UTF-8 bytes.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf8NullPadded<const N: usize>(pub FixedUtf8Bytes<N>);

/// Space-padded fixed UTF-8 bytes.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf8SpacePadded<const N: usize>(pub FixedUtf8Bytes<N>);

impl<const N: usize> From<FixedUtf8Bytes<N>> for FixedUtf8NullPadded<N> {
    fn from(v: FixedUtf8Bytes<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf8Bytes<N>> for FixedUtf8SpacePadded<N> {
    fn from(v: FixedUtf8Bytes<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf8NullPadded<N>> for FixedUtf8Bytes<N> {
    fn from(v: FixedUtf8NullPadded<N>) -> Self {
        v.0
    }
}

impl<const N: usize> From<FixedUtf8SpacePadded<N>> for FixedUtf8Bytes<N> {
    fn from(v: FixedUtf8SpacePadded<N>) -> Self {
        v.0
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf8NullPadded<N> {
    type Error = FixedUtf8Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let b = s.as_bytes();
        if b.len() > N {
            return Err(FixedUtf8Error::TooManyBytes {
                max: N,
                found: b.len(),
            });
        }
        let mut out = [0u8; N];
        out[..b.len()].copy_from_slice(b);
        Ok(Self(FixedUtf8Bytes { bytes: out }))
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf8SpacePadded<N> {
    type Error = FixedUtf8Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let b = s.as_bytes();
        if b.len() > N {
            return Err(FixedUtf8Error::TooManyBytes {
                max: N,
                found: b.len(),
            });
        }
        let mut out = [b' '; N];
        out[..b.len()].copy_from_slice(b);
        Ok(Self(FixedUtf8Bytes { bytes: out }))
    }
}

impl<const N: usize> TryFrom<String> for FixedUtf8NullPadded<N> {
    type Error = FixedUtf8Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl<const N: usize> TryFrom<String> for FixedUtf8SpacePadded<N> {
    type Error = FixedUtf8Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

fn trim_null_bytes(bytes: &[u8]) -> &[u8] {
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == 0 {
        end -= 1;
    }
    &bytes[..end]
}

fn trim_space_bytes(bytes: &[u8]) -> &[u8] {
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == b' ' {
        end -= 1;
    }
    &bytes[..end]
}

impl<const N: usize> TryFrom<&FixedUtf8NullPadded<N>> for String {
    type Error = FixedUtf8Error;

    fn try_from(v: &FixedUtf8NullPadded<N>) -> Result<Self, Self::Error> {
        let trimmed = trim_null_bytes(&v.0.bytes);
        core::str::from_utf8(trimmed)
            .map(String::from)
            .map_err(|_| FixedUtf8Error::InvalidUtf8)
    }
}

impl<const N: usize> TryFrom<&FixedUtf8SpacePadded<N>> for String {
    type Error = FixedUtf8Error;

    fn try_from(v: &FixedUtf8SpacePadded<N>) -> Result<Self, Self::Error> {
        let trimmed = trim_space_bytes(&v.0.bytes);
        core::str::from_utf8(trimmed)
            .map(String::from)
            .map_err(|_| FixedUtf8Error::InvalidUtf8)
    }
}

impl<const N: usize> From<&FixedUtf8NullPadded<N>> for Vec<u8> {
    fn from(v: &FixedUtf8NullPadded<N>) -> Self {
        v.0.bytes.to_vec()
    }
}

impl<const N: usize> From<&FixedUtf8SpacePadded<N>> for Vec<u8> {
    fn from(v: &FixedUtf8SpacePadded<N>) -> Self {
        v.0.bytes.to_vec()
    }
}

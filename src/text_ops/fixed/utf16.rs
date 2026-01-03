//! Fixed-size UTF-16 code-unit storage and common conventions.
//!
//! Historically this module only provided UTF-16 **little-endian** fixed buffers.
//! The fixed module now supports both endiannesses; the existing `...Le...` names
//! remain as type aliases for backwards compatibility.

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::fmt;

use crate::{
    BigEndian, LittleEndian, SpecificEndian, SpecificEndianOwned, Utf16StrBE, Utf16StrLE,
    Utf16StringBE, Utf16StringLE,
};

use super::FixedUtf16Error;

/// Inline, fixed-size UTF-16 code units stored with explicit endianness.
///
/// This type stores **exactly `N` UTF-16 code units** inline.
///
/// This is the endian-parameterized core type. For the host-endian convenience
/// alias, see [`FixedUtf16CodeUnits`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf16CodeUnitsEndian<E, const N: usize> {
    pub(crate) units: [E; N],
}

/// A borrowed reference to exactly `N` UTF-16 code units stored with explicit endianness.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf16CodeUnitsRefEndian<'a, E, const N: usize>(pub &'a [E; N]);

/// Host-endian fixed UTF-16 code units.
#[cfg(target_endian = "little")]
pub type FixedUtf16CodeUnits<const N: usize> = FixedUtf16LeCodeUnits<N>;
/// Host-endian fixed UTF-16 code units.
#[cfg(target_endian = "big")]
pub type FixedUtf16CodeUnits<const N: usize> = FixedUtf16BeCodeUnits<N>;

/// A borrowed reference to exactly `N` host-endian UTF-16 code units.
#[cfg(target_endian = "little")]
pub type FixedUtf16CodeUnitsRef<'a, const N: usize> = FixedUtf16LeCodeUnitsRef<'a, N>;
/// A borrowed reference to exactly `N` host-endian UTF-16 code units.
#[cfg(target_endian = "big")]
pub type FixedUtf16CodeUnitsRef<'a, const N: usize> = FixedUtf16BeCodeUnitsRef<'a, N>;

/// Little-endian fixed UTF-16 code units.
pub type FixedUtf16LeCodeUnits<const N: usize> = FixedUtf16CodeUnitsEndian<LittleEndian<u16>, N>;
/// Big-endian fixed UTF-16 code units.
pub type FixedUtf16BeCodeUnits<const N: usize> = FixedUtf16CodeUnitsEndian<BigEndian<u16>, N>;

/// A borrowed reference to exactly `N` UTF-16LE code units.
pub type FixedUtf16LeCodeUnitsRef<'a, const N: usize> =
    FixedUtf16CodeUnitsRefEndian<'a, LittleEndian<u16>, N>;
/// A borrowed reference to exactly `N` UTF-16BE code units.
pub type FixedUtf16BeCodeUnitsRef<'a, const N: usize> =
    FixedUtf16CodeUnitsRefEndian<'a, BigEndian<u16>, N>;

impl<E, const N: usize> FixedUtf16CodeUnitsEndian<E, N> {
    pub const fn as_units(&self) -> &[E; N] {
        &self.units
    }
}

impl<'a, E, const N: usize> FixedUtf16CodeUnitsRefEndian<'a, E, N> {
    pub const fn as_units(&self) -> &'a [E; N] {
        self.0
    }
}

impl<E, const N: usize> From<[E; N]> for FixedUtf16CodeUnitsEndian<E, N> {
    fn from(units: [E; N]) -> Self {
        Self { units }
    }
}

impl<'a, E, const N: usize> From<&'a [E; N]> for FixedUtf16CodeUnitsRefEndian<'a, E, N> {
    fn from(v: &'a [E; N]) -> Self {
        Self(v)
    }
}

impl<const N: usize> TryFrom<&[u16]> for FixedUtf16LeCodeUnits<N> {
    type Error = FixedUtf16Error;

    fn try_from(v: &[u16]) -> Result<Self, Self::Error> {
        if v.len() != N {
            return Err(FixedUtf16Error::WrongCodeUnitCount {
                expected: N,
                found: v.len(),
            });
        }
        let mut units = [LittleEndian::from_bits(0u16); N];
        for (dst, src) in units.iter_mut().zip(v.iter().copied()) {
            *dst = LittleEndian::from_bits(src);
        }
        Ok(Self { units })
    }
}

impl<const N: usize> TryFrom<&[u16]> for FixedUtf16BeCodeUnits<N> {
    type Error = FixedUtf16Error;

    fn try_from(v: &[u16]) -> Result<Self, Self::Error> {
        if v.len() != N {
            return Err(FixedUtf16Error::WrongCodeUnitCount {
                expected: N,
                found: v.len(),
            });
        }
        let mut units = [BigEndian::from_bits(0u16); N];
        for (dst, src) in units.iter_mut().zip(v.iter().copied()) {
            *dst = BigEndian::from_bits(src);
        }
        Ok(Self { units })
    }
}

impl<const N: usize> TryFrom<&[LittleEndian<u16>]> for FixedUtf16LeCodeUnits<N> {
    type Error = FixedUtf16Error;

    fn try_from(v: &[LittleEndian<u16>]) -> Result<Self, Self::Error> {
        if v.len() != N {
            return Err(FixedUtf16Error::WrongCodeUnitCount {
                expected: N,
                found: v.len(),
            });
        }
        let mut units = [LittleEndian::from_bits(0u16); N];
        units.copy_from_slice(v);
        Ok(Self { units })
    }
}

impl<const N: usize> TryFrom<&[BigEndian<u16>]> for FixedUtf16BeCodeUnits<N> {
    type Error = FixedUtf16Error;

    fn try_from(v: &[BigEndian<u16>]) -> Result<Self, Self::Error> {
        if v.len() != N {
            return Err(FixedUtf16Error::WrongCodeUnitCount {
                expected: N,
                found: v.len(),
            });
        }
        let mut units = [BigEndian::from_bits(0u16); N];
        units.copy_from_slice(v);
        Ok(Self { units })
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf16LeCodeUnits<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut it = s.encode_utf16();
        let mut units = [LittleEndian::from_bits(0u16); N];

        for (idx, dst) in units.iter_mut().enumerate() {
            match it.next() {
                Some(cu) => *dst = LittleEndian::from_bits(cu),
                None => {
                    return Err(FixedUtf16Error::WrongCodeUnitCount {
                        expected: N,
                        found: idx,
                    });
                }
            }
        }

        // If there are *more* code units beyond N, that's also an error.
        if let Some(_) = it.next() {
            // We already consumed exactly N; found is at least N+1.
            return Err(FixedUtf16Error::WrongCodeUnitCount {
                expected: N,
                found: N + 1,
            });
        }

        Ok(Self { units })
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf16BeCodeUnits<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut it = s.encode_utf16();
        let mut units = [BigEndian::from_bits(0u16); N];

        for (idx, dst) in units.iter_mut().enumerate() {
            match it.next() {
                Some(cu) => *dst = BigEndian::from_bits(cu),
                None => {
                    return Err(FixedUtf16Error::WrongCodeUnitCount {
                        expected: N,
                        found: idx,
                    });
                }
            }
        }

        if let Some(_) = it.next() {
            return Err(FixedUtf16Error::WrongCodeUnitCount {
                expected: N,
                found: N + 1,
            });
        }

        Ok(Self { units })
    }
}

impl<const N: usize> TryFrom<String> for FixedUtf16LeCodeUnits<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl<const N: usize> TryFrom<String> for FixedUtf16BeCodeUnits<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl<const N: usize> TryFrom<Utf16StringLE> for FixedUtf16LeCodeUnits<N> {
    type Error = FixedUtf16Error;

    fn try_from(v: Utf16StringLE) -> Result<Self, Self::Error> {
        Self::try_from(v.0.as_slice())
    }
}

impl<const N: usize> TryFrom<Utf16StringBE> for FixedUtf16BeCodeUnits<N> {
    type Error = FixedUtf16Error;

    fn try_from(v: Utf16StringBE) -> Result<Self, Self::Error> {
        Self::try_from(v.0.as_slice())
    }
}

impl<const N: usize> TryFrom<&FixedUtf16LeCodeUnits<N>> for String {
    type Error = FixedUtf16Error;

    fn try_from(v: &FixedUtf16LeCodeUnits<N>) -> Result<Self, Self::Error> {
        let native: Vec<u16> = v.units.iter().map(|cu| cu.to_native()).collect();
        String::from_utf16(&native).map_err(|_| FixedUtf16Error::InvalidUtf16)
    }
}

impl<const N: usize> TryFrom<&FixedUtf16BeCodeUnits<N>> for String {
    type Error = FixedUtf16Error;

    fn try_from(v: &FixedUtf16BeCodeUnits<N>) -> Result<Self, Self::Error> {
        let native: Vec<u16> = v.units.iter().map(|cu| cu.to_native()).collect();
        String::from_utf16(&native).map_err(|_| FixedUtf16Error::InvalidUtf16)
    }
}

impl<const N: usize> fmt::Display for FixedUtf16LeCodeUnits<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match String::try_from(self) {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "<invalid UTF-16>"),
        }
    }
}

impl<const N: usize> fmt::Display for FixedUtf16BeCodeUnits<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match String::try_from(self) {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "<invalid UTF-16>"),
        }
    }
}

/// Fixed UTF-16LE code units interpreted as a *packed* string.
///
/// No terminator or padding semantics are applied: decoding uses all `N` code units.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf16LePacked<const N: usize>(pub FixedUtf16LeCodeUnits<N>);

/// Fixed UTF-16BE code units interpreted as a *packed* string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf16BePacked<const N: usize>(pub FixedUtf16BeCodeUnits<N>);

/// Fixed UTF-16LE code units interpreted as a *NUL-padded* string.
///
/// When decoding, the first `0x0000` code unit terminates the string; remaining units
/// are ignored.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf16LeNullPadded<const N: usize>(pub FixedUtf16LeCodeUnits<N>);

/// Fixed UTF-16BE code units interpreted as a *NUL-padded* string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf16BeNullPadded<const N: usize>(pub FixedUtf16BeCodeUnits<N>);

/// Backwards-compatible name for [`FixedUtf16LeNullPadded`].
#[deprecated(
    since = "0.3.3",
    note = "renamed to FixedUtf16LeNullPadded (these fixed-length buffers are NUL-padded; a final terminator is not required)"
)]
pub type FixedUtf16LeNullTerminated<const N: usize> = FixedUtf16LeNullPadded<N>;

/// Fixed UTF-16LE code units interpreted as a *space-padded* string.
///
/// When decoding, trailing ASCII spaces (`0x0020`) are trimmed. (No NUL terminator is
/// required.)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf16LeSpacePadded<const N: usize>(pub FixedUtf16LeCodeUnits<N>);

/// Fixed UTF-16BE code units interpreted as a *space-padded* string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf16BeSpacePadded<const N: usize>(pub FixedUtf16BeCodeUnits<N>);

impl<const N: usize> From<FixedUtf16LeCodeUnits<N>> for FixedUtf16LePacked<N> {
    fn from(v: FixedUtf16LeCodeUnits<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf16BeCodeUnits<N>> for FixedUtf16BePacked<N> {
    fn from(v: FixedUtf16BeCodeUnits<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf16LeCodeUnits<N>> for FixedUtf16LeNullPadded<N> {
    fn from(v: FixedUtf16LeCodeUnits<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf16BeCodeUnits<N>> for FixedUtf16BeNullPadded<N> {
    fn from(v: FixedUtf16BeCodeUnits<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf16LeCodeUnits<N>> for FixedUtf16LeSpacePadded<N> {
    fn from(v: FixedUtf16LeCodeUnits<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf16BeCodeUnits<N>> for FixedUtf16BeSpacePadded<N> {
    fn from(v: FixedUtf16BeCodeUnits<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf16LePacked<N>> for FixedUtf16LeCodeUnits<N> {
    fn from(v: FixedUtf16LePacked<N>) -> Self {
        v.0
    }
}

impl<const N: usize> From<FixedUtf16BePacked<N>> for FixedUtf16BeCodeUnits<N> {
    fn from(v: FixedUtf16BePacked<N>) -> Self {
        v.0
    }
}

impl<const N: usize> From<FixedUtf16LeNullPadded<N>> for FixedUtf16LeCodeUnits<N> {
    fn from(v: FixedUtf16LeNullPadded<N>) -> Self {
        v.0
    }
}

impl<const N: usize> From<FixedUtf16BeNullPadded<N>> for FixedUtf16BeCodeUnits<N> {
    fn from(v: FixedUtf16BeNullPadded<N>) -> Self {
        v.0
    }
}

impl<const N: usize> From<FixedUtf16LeSpacePadded<N>> for FixedUtf16LeCodeUnits<N> {
    fn from(v: FixedUtf16LeSpacePadded<N>) -> Self {
        v.0
    }
}

impl<const N: usize> From<FixedUtf16BeSpacePadded<N>> for FixedUtf16BeCodeUnits<N> {
    fn from(v: FixedUtf16BeSpacePadded<N>) -> Self {
        v.0
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf16LePacked<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        FixedUtf16LeCodeUnits::try_from(s).map(Self)
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf16BePacked<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        FixedUtf16BeCodeUnits::try_from(s).map(Self)
    }
}

impl<const N: usize> TryFrom<String> for FixedUtf16LePacked<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        FixedUtf16LeCodeUnits::try_from(s).map(Self)
    }
}

impl<const N: usize> TryFrom<String> for FixedUtf16BePacked<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        FixedUtf16BeCodeUnits::try_from(s).map(Self)
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf16LeNullPadded<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut it = s.encode_utf16();
        let mut units = [LittleEndian::from_bits(0u16); N];
        let mut i = 0usize;

        while i < N {
            match it.next() {
                Some(cu) => {
                    units[i] = LittleEndian::from_bits(cu);
                    i += 1;
                }
                None => break,
            }
        }

        // If there are any remaining code units, the string doesn't fit.
        if let Some(_) = it.next() {
            return Err(FixedUtf16Error::WrongCodeUnitCount {
                expected: N,
                found: N + 1,
            });
        }

        Ok(Self(FixedUtf16LeCodeUnits { units }))
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf16BeNullPadded<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut it = s.encode_utf16();
        let mut units = [BigEndian::from_bits(0u16); N];
        let mut i = 0usize;

        while i < N {
            match it.next() {
                Some(cu) => {
                    units[i] = BigEndian::from_bits(cu);
                    i += 1;
                }
                None => break,
            }
        }

        if let Some(_) = it.next() {
            return Err(FixedUtf16Error::WrongCodeUnitCount {
                expected: N,
                found: N + 1,
            });
        }

        Ok(Self(FixedUtf16CodeUnitsEndian { units }))
    }
}

impl<const N: usize> TryFrom<String> for FixedUtf16LeNullPadded<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf16LeSpacePadded<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut it = s.encode_utf16();
        // Like the UTF-32 fixed text helpers, we store raw bits tagged with the
        // specified endianness. That means on big-endian hosts we must pre-swap
        // for LE storage so that `.to_native()` returns the intended scalar.
        let space_bits = if cfg!(target_endian = "big") {
            0x0020u16.swap_bytes()
        } else {
            0x0020u16
        };
        let mut units = [LittleEndian::from_bits(space_bits); N];
        let mut i = 0usize;

        while i < N {
            match it.next() {
                Some(cu) => {
                    let bits = if cfg!(target_endian = "big") {
                        cu.swap_bytes()
                    } else {
                        cu
                    };
                    units[i] = LittleEndian::from_bits(bits);
                    i += 1;
                }
                None => break,
            }
        }

        if let Some(_) = it.next() {
            return Err(FixedUtf16Error::WrongCodeUnitCount {
                expected: N,
                found: N + 1,
            });
        }

        Ok(Self(FixedUtf16LeCodeUnits { units }))
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf16BeSpacePadded<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut it = s.encode_utf16();
        // Pre-swap on little-endian hosts for BE storage.
        let space_bits = if cfg!(target_endian = "little") {
            0x0020u16.swap_bytes()
        } else {
            0x0020u16
        };
        let mut units = [BigEndian::from_bits(space_bits); N];
        let mut i = 0usize;

        while i < N {
            match it.next() {
                Some(cu) => {
                    let bits = if cfg!(target_endian = "little") {
                        cu.swap_bytes()
                    } else {
                        cu
                    };
                    units[i] = BigEndian::from_bits(bits);
                    i += 1;
                }
                None => break,
            }
        }

        if let Some(_) = it.next() {
            return Err(FixedUtf16Error::WrongCodeUnitCount {
                expected: N,
                found: N + 1,
            });
        }

        Ok(Self(FixedUtf16CodeUnitsEndian { units }))
    }
}

impl<const N: usize> TryFrom<String> for FixedUtf16LeSpacePadded<N> {
    type Error = FixedUtf16Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl<const N: usize> TryFrom<&FixedUtf16LePacked<N>> for String {
    type Error = FixedUtf16Error;

    fn try_from(v: &FixedUtf16LePacked<N>) -> Result<Self, Self::Error> {
        String::try_from(&v.0)
    }
}

impl<const N: usize> TryFrom<&FixedUtf16BePacked<N>> for String {
    type Error = FixedUtf16Error;

    fn try_from(v: &FixedUtf16BePacked<N>) -> Result<Self, Self::Error> {
        String::try_from(&v.0)
    }
}

impl<const N: usize> TryFrom<&FixedUtf16LeNullPadded<N>> for String {
    type Error = FixedUtf16Error;

    fn try_from(v: &FixedUtf16LeNullPadded<N>) -> Result<Self, Self::Error> {
        // Find first NUL and decode only that prefix.
        let mut end = N;
        for (i, cu) in v.0.as_units().iter().enumerate() {
            if cu.to_native() == 0 {
                end = i;
                break;
            }
        }
        String::try_from(Utf16StrLE::from(&v.0.as_units()[..end]))
            .map_err(|_| FixedUtf16Error::InvalidUtf16)
    }
}

impl<const N: usize> TryFrom<&FixedUtf16BeNullPadded<N>> for String {
    type Error = FixedUtf16Error;

    fn try_from(v: &FixedUtf16BeNullPadded<N>) -> Result<Self, Self::Error> {
        // Find first NUL and decode only that prefix.
        let mut end = N;
        for (i, cu) in v.0.as_units().iter().enumerate() {
            if cu.to_native() == 0 {
                end = i;
                break;
            }
        }
        String::try_from(Utf16StrBE::from(&v.0.as_units()[..end]))
            .map_err(|_| FixedUtf16Error::InvalidUtf16)
    }
}

impl<const N: usize> TryFrom<&FixedUtf16LeSpacePadded<N>> for String {
    type Error = FixedUtf16Error;

    fn try_from(v: &FixedUtf16LeSpacePadded<N>) -> Result<Self, Self::Error> {
        // Trim trailing ASCII spaces.
        let mut end = N;
        while end > 0 {
            let cu = v.0.as_units()[end - 1].to_native();
            if cu == 0x0020 {
                end -= 1;
            } else {
                break;
            }
        }
        String::try_from(Utf16StrLE::from(&v.0.as_units()[..end]))
            .map_err(|_| FixedUtf16Error::InvalidUtf16)
    }
}

impl<const N: usize> TryFrom<&FixedUtf16BeSpacePadded<N>> for String {
    type Error = FixedUtf16Error;

    fn try_from(v: &FixedUtf16BeSpacePadded<N>) -> Result<Self, Self::Error> {
        // Trim trailing ASCII spaces.
        let mut end = N;
        while end > 0 {
            let cu = v.0.as_units()[end - 1].to_native();
            if cu == 0x0020 {
                end -= 1;
            } else {
                break;
            }
        }
        String::try_from(Utf16StrBE::from(&v.0.as_units()[..end]))
            .map_err(|_| FixedUtf16Error::InvalidUtf16)
    }
}

impl<const N: usize> SpecificEndianOwned for FixedUtf16LeCodeUnits<N> {
    type Big = FixedUtf16BeCodeUnits<N>;
    type Little = FixedUtf16LeCodeUnits<N>;

    fn to_big_endian(&self) -> Self::Big {
        let mut units = [BigEndian::from_bits(0u16); N];
        for (dst, src) in units.iter_mut().zip(self.units.iter()) {
            *dst = BigEndian::from_bits(src.to_native());
        }
        FixedUtf16CodeUnitsEndian { units }
    }

    fn to_little_endian(&self) -> Self::Little {
        *self
    }

    fn from_big_endian(&self) -> Self::Big {
        SpecificEndianOwned::to_big_endian(self)
    }

    fn from_little_endian(&self) -> Self::Little {
        *self
    }
}

impl<const N: usize> SpecificEndianOwned for FixedUtf16BeCodeUnits<N> {
    type Big = FixedUtf16BeCodeUnits<N>;
    type Little = FixedUtf16LeCodeUnits<N>;

    fn to_big_endian(&self) -> Self::Big {
        *self
    }

    fn to_little_endian(&self) -> Self::Little {
        let mut units = [LittleEndian::from_bits(0u16); N];
        for (dst, src) in units.iter_mut().zip(self.units.iter()) {
            *dst = LittleEndian::from_bits(src.to_native());
        }
        FixedUtf16CodeUnitsEndian { units }
    }

    fn from_big_endian(&self) -> Self::Big {
        *self
    }

    fn from_little_endian(&self) -> Self::Little {
        SpecificEndianOwned::to_little_endian(self)
    }
}

// Implement `SpecificEndian<T>` so the fixed buffers can be wrapped in `BigEndian<T>` / `LittleEndian<T>`.
impl<const N: usize> SpecificEndian<FixedUtf16LeCodeUnits<N>> for FixedUtf16LeCodeUnits<N> {
    fn to_big_endian(&self) -> FixedUtf16LeCodeUnits<N> {
        // Represent *these bits* as big-endian code units.
        // We must swap each contained code unit.
        let mut units = [LittleEndian::from_bits(0u16); N];
        for (dst, src) in units.iter_mut().zip(self.units.iter()) {
            let v = src.to_native();
            *dst = LittleEndian::from_bits(v.to_be());
        }
        FixedUtf16CodeUnitsEndian { units }
    }

    fn to_little_endian(&self) -> FixedUtf16LeCodeUnits<N> {
        *self
    }

    fn from_big_endian(&self) -> FixedUtf16LeCodeUnits<N> {
        // Stored bits are big-endian; reinterpret into little-endian code units.
        let mut units = [LittleEndian::from_bits(0u16); N];
        for (dst, src) in units.iter_mut().zip(self.units.iter()) {
            let v = src.to_native();
            *dst = LittleEndian::from_bits(u16::from_be(v));
        }
        FixedUtf16CodeUnitsEndian { units }
    }

    fn from_little_endian(&self) -> FixedUtf16LeCodeUnits<N> {
        *self
    }
}

impl<const N: usize> SpecificEndian<FixedUtf16BeCodeUnits<N>> for FixedUtf16BeCodeUnits<N> {
    fn to_big_endian(&self) -> FixedUtf16BeCodeUnits<N> {
        *self
    }

    fn to_little_endian(&self) -> FixedUtf16BeCodeUnits<N> {
        let mut units = [BigEndian::from_bits(0u16); N];
        for (dst, src) in units.iter_mut().zip(self.units.iter()) {
            let v = src.to_native();
            *dst = BigEndian::from_bits(v.to_le());
        }
        FixedUtf16CodeUnitsEndian { units }
    }

    fn from_big_endian(&self) -> FixedUtf16BeCodeUnits<N> {
        *self
    }

    fn from_little_endian(&self) -> FixedUtf16BeCodeUnits<N> {
        let mut units = [BigEndian::from_bits(0u16); N];
        for (dst, src) in units.iter_mut().zip(self.units.iter()) {
            let v = src.to_native();
            *dst = BigEndian::from_bits(u16::from_le(v));
        }
        FixedUtf16CodeUnitsEndian { units }
    }
}

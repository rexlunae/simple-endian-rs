//! Fixed-size UTF-32 code-unit storage and common conventions.

extern crate alloc;

use alloc::string::String;
use core::fmt;

use crate::{BigEndian, LittleEndian, SpecificEndian, SpecificEndianOwned, Utf32StrBE, Utf32StrLE, Utf32StringBE, Utf32StringLE};

/// Errors for fixed UTF-32 code-unit storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixedUtf32Error {
    /// Input had the wrong number of UTF-32 code units.
    WrongCodeUnitCount { expected: usize, found: usize },
    /// Input code units are not valid Unicode scalar values.
    InvalidUtf32,
}

impl fmt::Display for FixedUtf32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixedUtf32Error::WrongCodeUnitCount { expected, found } => write!(
                f,
                "wrong number of UTF-32 code units (expected {expected}, found {found})"
            ),
            FixedUtf32Error::InvalidUtf32 => write!(f, "invalid UTF-32"),
        }
    }
}

#[cfg(any(feature = "io-std", feature = "io"))]
impl std::error::Error for FixedUtf32Error {}

/// Inline, fixed-size UTF-32 code units stored with explicit endianness.
///
/// This is the endian-parameterized core type. For the host-endian convenience
/// alias, see [`FixedUtf32CodeUnits`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf32CodeUnitsEndian<E, const N: usize> {
    pub(crate) units: [E; N],
}

/// A borrowed reference to exactly `N` UTF-32 code units stored with explicit endianness.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf32CodeUnitsRefEndian<'a, E, const N: usize>(pub &'a [E; N]);

/// Host-endian fixed UTF-32 code units.
#[cfg(target_endian = "little")]
pub type FixedUtf32CodeUnits<const N: usize> = FixedUtf32LeCodeUnits<N>;
/// Host-endian fixed UTF-32 code units.
#[cfg(target_endian = "big")]
pub type FixedUtf32CodeUnits<const N: usize> = FixedUtf32BeCodeUnits<N>;

/// A borrowed reference to exactly `N` host-endian UTF-32 code units.
#[cfg(target_endian = "little")]
pub type FixedUtf32CodeUnitsRef<'a, const N: usize> = FixedUtf32LeCodeUnitsRef<'a, N>;
/// A borrowed reference to exactly `N` host-endian UTF-32 code units.
#[cfg(target_endian = "big")]
pub type FixedUtf32CodeUnitsRef<'a, const N: usize> = FixedUtf32BeCodeUnitsRef<'a, N>;

pub type FixedUtf32LeCodeUnits<const N: usize> = FixedUtf32CodeUnitsEndian<LittleEndian<u32>, N>;
pub type FixedUtf32BeCodeUnits<const N: usize> = FixedUtf32CodeUnitsEndian<BigEndian<u32>, N>;

pub type FixedUtf32LeCodeUnitsRef<'a, const N: usize> =
    FixedUtf32CodeUnitsRefEndian<'a, LittleEndian<u32>, N>;
pub type FixedUtf32BeCodeUnitsRef<'a, const N: usize> =
    FixedUtf32CodeUnitsRefEndian<'a, BigEndian<u32>, N>;

impl<E, const N: usize> FixedUtf32CodeUnitsEndian<E, N> {
    pub const fn as_units(&self) -> &[E; N] {
        &self.units
    }
}

impl<'a, E, const N: usize> FixedUtf32CodeUnitsRefEndian<'a, E, N> {
    pub const fn as_units(&self) -> &'a [E; N] {
        self.0
    }
}

impl<E, const N: usize> From<[E; N]> for FixedUtf32CodeUnitsEndian<E, N> {
    fn from(units: [E; N]) -> Self {
        Self { units }
    }
}

impl<'a, E, const N: usize> From<&'a [E; N]> for FixedUtf32CodeUnitsRefEndian<'a, E, N> {
    fn from(v: &'a [E; N]) -> Self {
        Self(v)
    }
}

impl<const N: usize> TryFrom<&[u32]> for FixedUtf32LeCodeUnits<N> {
    type Error = FixedUtf32Error;

    fn try_from(v: &[u32]) -> Result<Self, Self::Error> {
        if v.len() != N {
            return Err(FixedUtf32Error::WrongCodeUnitCount {
                expected: N,
                found: v.len(),
            });
        }
        let mut units = [LittleEndian::from_bits(0u32); N];
        for (dst, src) in units.iter_mut().zip(v.iter().copied()) {
            *dst = LittleEndian::from_bits(src);
        }
        Ok(Self { units })
    }
}

impl<const N: usize> TryFrom<&[u32]> for FixedUtf32BeCodeUnits<N> {
    type Error = FixedUtf32Error;

    fn try_from(v: &[u32]) -> Result<Self, Self::Error> {
        if v.len() != N {
            return Err(FixedUtf32Error::WrongCodeUnitCount {
                expected: N,
                found: v.len(),
            });
        }
        let mut units = [BigEndian::from_bits(0u32); N];
        for (dst, src) in units.iter_mut().zip(v.iter().copied()) {
            *dst = BigEndian::from_bits(src);
        }
        Ok(Self { units })
    }
}

impl<const N: usize> TryFrom<&[LittleEndian<u32>]> for FixedUtf32LeCodeUnits<N> {
    type Error = FixedUtf32Error;

    fn try_from(v: &[LittleEndian<u32>]) -> Result<Self, Self::Error> {
        if v.len() != N {
            return Err(FixedUtf32Error::WrongCodeUnitCount {
                expected: N,
                found: v.len(),
            });
        }
        let mut units = [LittleEndian::from_bits(0u32); N];
        units.copy_from_slice(v);
        Ok(Self { units })
    }
}

impl<const N: usize> TryFrom<&[BigEndian<u32>]> for FixedUtf32BeCodeUnits<N> {
    type Error = FixedUtf32Error;

    fn try_from(v: &[BigEndian<u32>]) -> Result<Self, Self::Error> {
        if v.len() != N {
            return Err(FixedUtf32Error::WrongCodeUnitCount {
                expected: N,
                found: v.len(),
            });
        }
        let mut units = [BigEndian::from_bits(0u32); N];
        units.copy_from_slice(v);
        Ok(Self { units })
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf32LeCodeUnits<N> {
    type Error = FixedUtf32Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut it = s.chars();
        let mut units = [LittleEndian::from_bits(0u32); N];

        for (idx, dst) in units.iter_mut().enumerate() {
            match it.next() {
                Some(ch) => *dst = LittleEndian::from_bits(ch as u32),
                None => {
                    return Err(FixedUtf32Error::WrongCodeUnitCount {
                        expected: N,
                        found: idx,
                    })
                }
            }
        }

        if let Some(_) = it.next() {
            return Err(FixedUtf32Error::WrongCodeUnitCount {
                expected: N,
                found: N + 1,
            });
        }

        Ok(Self { units })
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf32BeCodeUnits<N> {
    type Error = FixedUtf32Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut it = s.chars();
        let mut units = [BigEndian::from_bits(0u32); N];

        for (idx, dst) in units.iter_mut().enumerate() {
            match it.next() {
                Some(ch) => *dst = BigEndian::from_bits(ch as u32),
                None => {
                    return Err(FixedUtf32Error::WrongCodeUnitCount {
                        expected: N,
                        found: idx,
                    })
                }
            }
        }

        if let Some(_) = it.next() {
            return Err(FixedUtf32Error::WrongCodeUnitCount {
                expected: N,
                found: N + 1,
            });
        }

        Ok(Self { units })
    }
}

impl<const N: usize> TryFrom<String> for FixedUtf32LeCodeUnits<N> {
    type Error = FixedUtf32Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl<const N: usize> TryFrom<String> for FixedUtf32BeCodeUnits<N> {
    type Error = FixedUtf32Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl<const N: usize> TryFrom<Utf32StringLE> for FixedUtf32LeCodeUnits<N> {
    type Error = FixedUtf32Error;

    fn try_from(v: Utf32StringLE) -> Result<Self, Self::Error> {
        Self::try_from(v.0.as_slice())
    }
}

impl<const N: usize> TryFrom<Utf32StringBE> for FixedUtf32BeCodeUnits<N> {
    type Error = FixedUtf32Error;

    fn try_from(v: Utf32StringBE) -> Result<Self, Self::Error> {
        Self::try_from(v.0.as_slice())
    }
}

impl<const N: usize> TryFrom<&FixedUtf32LeCodeUnits<N>> for String {
    type Error = FixedUtf32Error;

    fn try_from(v: &FixedUtf32LeCodeUnits<N>) -> Result<Self, Self::Error> {
        let mut out = String::new();
        for cu in v.units.iter().map(|x| x.to_native()) {
            match char::from_u32(cu) {
                Some(c) => out.push(c),
                None => return Err(FixedUtf32Error::InvalidUtf32),
            }
        }
        Ok(out)
    }
}

impl<const N: usize> TryFrom<&FixedUtf32BeCodeUnits<N>> for String {
    type Error = FixedUtf32Error;

    fn try_from(v: &FixedUtf32BeCodeUnits<N>) -> Result<Self, Self::Error> {
        let mut out = String::new();
        for cu in v.units.iter().map(|x| x.to_native()) {
            match char::from_u32(cu) {
                Some(c) => out.push(c),
                None => return Err(FixedUtf32Error::InvalidUtf32),
            }
        }
        Ok(out)
    }
}

impl<const N: usize> fmt::Display for FixedUtf32LeCodeUnits<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match String::try_from(self) {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "<invalid UTF-32>"),
        }
    }
}

impl<const N: usize> fmt::Display for FixedUtf32BeCodeUnits<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match String::try_from(self) {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "<invalid UTF-32>"),
        }
    }
}

/// Fixed UTF-32LE code units interpreted as a *packed* string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf32LePacked<const N: usize>(pub FixedUtf32LeCodeUnits<N>);

/// Fixed UTF-32BE code units interpreted as a *packed* string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf32BePacked<const N: usize>(pub FixedUtf32BeCodeUnits<N>);

/// Fixed UTF-32LE code units interpreted as a *NUL-padded* string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf32LeNullPadded<const N: usize>(pub FixedUtf32LeCodeUnits<N>);

/// Fixed UTF-32BE code units interpreted as a *NUL-padded* string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf32BeNullPadded<const N: usize>(pub FixedUtf32BeCodeUnits<N>);

/// Fixed UTF-32LE code units interpreted as a *space-padded* string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf32LeSpacePadded<const N: usize>(pub FixedUtf32LeCodeUnits<N>);

/// Fixed UTF-32BE code units interpreted as a *space-padded* string.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FixedUtf32BeSpacePadded<const N: usize>(pub FixedUtf32BeCodeUnits<N>);

impl<const N: usize> From<FixedUtf32LeCodeUnits<N>> for FixedUtf32LePacked<N> {
    fn from(v: FixedUtf32LeCodeUnits<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf32BeCodeUnits<N>> for FixedUtf32BePacked<N> {
    fn from(v: FixedUtf32BeCodeUnits<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf32LeCodeUnits<N>> for FixedUtf32LeNullPadded<N> {
    fn from(v: FixedUtf32LeCodeUnits<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf32BeCodeUnits<N>> for FixedUtf32BeNullPadded<N> {
    fn from(v: FixedUtf32BeCodeUnits<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf32LeCodeUnits<N>> for FixedUtf32LeSpacePadded<N> {
    fn from(v: FixedUtf32LeCodeUnits<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf32BeCodeUnits<N>> for FixedUtf32BeSpacePadded<N> {
    fn from(v: FixedUtf32BeCodeUnits<N>) -> Self {
        Self(v)
    }
}

impl<const N: usize> From<FixedUtf32LePacked<N>> for FixedUtf32LeCodeUnits<N> {
    fn from(v: FixedUtf32LePacked<N>) -> Self {
        v.0
    }
}

impl<const N: usize> From<FixedUtf32BePacked<N>> for FixedUtf32BeCodeUnits<N> {
    fn from(v: FixedUtf32BePacked<N>) -> Self {
        v.0
    }
}

impl<const N: usize> From<FixedUtf32LeNullPadded<N>> for FixedUtf32LeCodeUnits<N> {
    fn from(v: FixedUtf32LeNullPadded<N>) -> Self {
        v.0
    }
}

impl<const N: usize> From<FixedUtf32BeNullPadded<N>> for FixedUtf32BeCodeUnits<N> {
    fn from(v: FixedUtf32BeNullPadded<N>) -> Self {
        v.0
    }
}

impl<const N: usize> From<FixedUtf32LeSpacePadded<N>> for FixedUtf32LeCodeUnits<N> {
    fn from(v: FixedUtf32LeSpacePadded<N>) -> Self {
        v.0
    }
}

impl<const N: usize> From<FixedUtf32BeSpacePadded<N>> for FixedUtf32BeCodeUnits<N> {
    fn from(v: FixedUtf32BeSpacePadded<N>) -> Self {
        v.0
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf32LePacked<N> {
    type Error = FixedUtf32Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        FixedUtf32LeCodeUnits::try_from(s).map(Self)
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf32BePacked<N> {
    type Error = FixedUtf32Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        FixedUtf32BeCodeUnits::try_from(s).map(Self)
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf32LeNullPadded<N> {
    type Error = FixedUtf32Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut it = s.chars();
        let mut units = [LittleEndian::from_bits(0u32); N];
        let mut i = 0usize;

        while i < N {
            match it.next() {
                Some(ch) => {
                    units[i] = LittleEndian::from_bits(ch as u32);
                    i += 1;
                }
                None => break,
            }
        }

        if let Some(_) = it.next() {
            return Err(FixedUtf32Error::WrongCodeUnitCount {
                expected: N,
                found: N + 1,
            });
        }

        Ok(Self(FixedUtf32CodeUnitsEndian { units }))
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf32BeNullPadded<N> {
    type Error = FixedUtf32Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut it = s.chars();
        let mut units = [BigEndian::from_bits(0u32); N];
        let mut i = 0usize;

        while i < N {
            match it.next() {
                Some(ch) => {
                    units[i] = BigEndian::from_bits(ch as u32);
                    i += 1;
                }
                None => break,
            }
        }

        if let Some(_) = it.next() {
            return Err(FixedUtf32Error::WrongCodeUnitCount {
                expected: N,
                found: N + 1,
            });
        }

        Ok(Self(FixedUtf32CodeUnitsEndian { units }))
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf32LeSpacePadded<N> {
    type Error = FixedUtf32Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut it = s.chars();
        let mut units = [LittleEndian::from_bits(0x0020u32); N];
        let mut i = 0usize;

        while i < N {
            match it.next() {
                Some(ch) => {
                    units[i] = LittleEndian::from_bits(ch as u32);
                    i += 1;
                }
                None => break,
            }
        }

        if let Some(_) = it.next() {
            return Err(FixedUtf32Error::WrongCodeUnitCount {
                expected: N,
                found: N + 1,
            });
        }

        Ok(Self(FixedUtf32CodeUnitsEndian { units }))
    }
}

impl<const N: usize> TryFrom<&str> for FixedUtf32BeSpacePadded<N> {
    type Error = FixedUtf32Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut it = s.chars();
        let mut units = [BigEndian::from_bits(0x0020u32); N];
        let mut i = 0usize;

        while i < N {
            match it.next() {
                Some(ch) => {
                    units[i] = BigEndian::from_bits(ch as u32);
                    i += 1;
                }
                None => break,
            }
        }

        if let Some(_) = it.next() {
            return Err(FixedUtf32Error::WrongCodeUnitCount {
                expected: N,
                found: N + 1,
            });
        }

        Ok(Self(FixedUtf32CodeUnitsEndian { units }))
    }
}

impl<const N: usize> TryFrom<&FixedUtf32LePacked<N>> for String {
    type Error = FixedUtf32Error;

    fn try_from(v: &FixedUtf32LePacked<N>) -> Result<Self, Self::Error> {
        String::try_from(&v.0)
    }
}

impl<const N: usize> TryFrom<&FixedUtf32BePacked<N>> for String {
    type Error = FixedUtf32Error;

    fn try_from(v: &FixedUtf32BePacked<N>) -> Result<Self, Self::Error> {
        String::try_from(&v.0)
    }
}

impl<const N: usize> TryFrom<&FixedUtf32LeNullPadded<N>> for String {
    type Error = FixedUtf32Error;

    fn try_from(v: &FixedUtf32LeNullPadded<N>) -> Result<Self, Self::Error> {
        let mut end = N;
        for (i, cu) in v.0.as_units().iter().enumerate() {
            if cu.to_native() == 0 {
                end = i;
                break;
            }
        }
        String::try_from(Utf32StrLE::from(&v.0.as_units()[..end])).map_err(|_| FixedUtf32Error::InvalidUtf32)
    }
}

impl<const N: usize> TryFrom<&FixedUtf32BeNullPadded<N>> for String {
    type Error = FixedUtf32Error;

    fn try_from(v: &FixedUtf32BeNullPadded<N>) -> Result<Self, Self::Error> {
        let mut end = N;
        for (i, cu) in v.0.as_units().iter().enumerate() {
            if cu.to_native() == 0 {
                end = i;
                break;
            }
        }
        String::try_from(Utf32StrBE::from(&v.0.as_units()[..end])).map_err(|_| FixedUtf32Error::InvalidUtf32)
    }
}

impl<const N: usize> TryFrom<&FixedUtf32LeSpacePadded<N>> for String {
    type Error = FixedUtf32Error;

    fn try_from(v: &FixedUtf32LeSpacePadded<N>) -> Result<Self, Self::Error> {
        let mut end = N;
        while end > 0 {
            let cu = v.0.as_units()[end - 1].to_native();
            if cu == 0x0020 {
                end -= 1;
            } else {
                break;
            }
        }
        String::try_from(Utf32StrLE::from(&v.0.as_units()[..end])).map_err(|_| FixedUtf32Error::InvalidUtf32)
    }
}

impl<const N: usize> TryFrom<&FixedUtf32BeSpacePadded<N>> for String {
    type Error = FixedUtf32Error;

    fn try_from(v: &FixedUtf32BeSpacePadded<N>) -> Result<Self, Self::Error> {
        let mut end = N;
        while end > 0 {
            let cu = v.0.as_units()[end - 1].to_native();
            if cu == 0x0020 {
                end -= 1;
            } else {
                break;
            }
        }
        String::try_from(Utf32StrBE::from(&v.0.as_units()[..end])).map_err(|_| FixedUtf32Error::InvalidUtf32)
    }
}

impl<const N: usize> SpecificEndianOwned for FixedUtf32LeCodeUnits<N> {
    type Big = FixedUtf32BeCodeUnits<N>;
    type Little = FixedUtf32LeCodeUnits<N>;

    fn to_big_endian(&self) -> Self::Big {
        let mut units = [BigEndian::from_bits(0u32); N];
        for (dst, src) in units.iter_mut().zip(self.units.iter()) {
            *dst = BigEndian::from_bits(src.to_native());
        }
    FixedUtf32CodeUnitsEndian { units }
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

impl<const N: usize> SpecificEndianOwned for FixedUtf32BeCodeUnits<N> {
    type Big = FixedUtf32BeCodeUnits<N>;
    type Little = FixedUtf32LeCodeUnits<N>;

    fn to_big_endian(&self) -> Self::Big {
        *self
    }

    fn to_little_endian(&self) -> Self::Little {
        let mut units = [LittleEndian::from_bits(0u32); N];
        for (dst, src) in units.iter_mut().zip(self.units.iter()) {
            *dst = LittleEndian::from_bits(src.to_native());
        }
    FixedUtf32CodeUnitsEndian { units }
    }

    fn from_big_endian(&self) -> Self::Big {
        *self
    }

    fn from_little_endian(&self) -> Self::Little {
        SpecificEndianOwned::to_little_endian(self)
    }
}

// Implement `SpecificEndian<T>` so the fixed buffers can be wrapped in `BigEndian<T>` / `LittleEndian<T>`.
impl<const N: usize> SpecificEndian<FixedUtf32LeCodeUnits<N>> for FixedUtf32LeCodeUnits<N> {
    fn to_big_endian(&self) -> FixedUtf32LeCodeUnits<N> {
        let mut units = [LittleEndian::from_bits(0u32); N];
        for (dst, src) in units.iter_mut().zip(self.units.iter()) {
            let v = src.to_native();
            *dst = LittleEndian::from_bits(v.to_be());
        }
        FixedUtf32CodeUnitsEndian { units }
    }

    fn to_little_endian(&self) -> FixedUtf32LeCodeUnits<N> {
        *self
    }

    fn from_big_endian(&self) -> FixedUtf32LeCodeUnits<N> {
        let mut units = [LittleEndian::from_bits(0u32); N];
        for (dst, src) in units.iter_mut().zip(self.units.iter()) {
            let v = src.to_native();
            *dst = LittleEndian::from_bits(u32::from_be(v));
        }
        FixedUtf32CodeUnitsEndian { units }
    }

    fn from_little_endian(&self) -> FixedUtf32LeCodeUnits<N> {
        *self
    }
}

impl<const N: usize> SpecificEndian<FixedUtf32BeCodeUnits<N>> for FixedUtf32BeCodeUnits<N> {
    fn to_big_endian(&self) -> FixedUtf32BeCodeUnits<N> {
        *self
    }

    fn to_little_endian(&self) -> FixedUtf32BeCodeUnits<N> {
        let mut units = [BigEndian::from_bits(0u32); N];
        for (dst, src) in units.iter_mut().zip(self.units.iter()) {
            let v = src.to_native();
            *dst = BigEndian::from_bits(v.to_le());
        }
        FixedUtf32CodeUnitsEndian { units }
    }

    fn from_big_endian(&self) -> FixedUtf32BeCodeUnits<N> {
        *self
    }

    fn from_little_endian(&self) -> FixedUtf32BeCodeUnits<N> {
        let mut units = [BigEndian::from_bits(0u32); N];
        for (dst, src) in units.iter_mut().zip(self.units.iter()) {
            let v = src.to_native();
            *dst = BigEndian::from_bits(u32::from_le(v));
        }
        FixedUtf32CodeUnitsEndian { units }
    }
}

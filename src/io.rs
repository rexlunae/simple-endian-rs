// top-level helper removed; modules import size_of where needed

// Core-only helpers: operate on slices/Vec<u8> and don't require std IO traits.
// Enabled under the `io-core` feature.

#[cfg(feature = "io-core")]
pub mod core_io {
    use crate::{BigEndian, LittleEndian, SpecificEndian};
    use core::mem::size_of;

    // Canonical representation: use u128 as a universal integer representation
    // for all supported types. This simplifies generic conversion without
    // associated type gymnastics.
    pub trait EndianRepr: Copy {
        fn from_u128(v: u128) -> Self;
        fn to_u128(self) -> u128;
    }

    impl EndianRepr for u8 {
        fn from_u128(v: u128) -> Self { v as u8 }
        fn to_u128(self) -> u128 { self as u128 }
    }
    impl EndianRepr for u16 {
        fn from_u128(v: u128) -> Self { v as u16 }
        fn to_u128(self) -> u128 { self as u128 }
    }
    impl EndianRepr for u32 {
        fn from_u128(v: u128) -> Self { v as u32 }
        fn to_u128(self) -> u128 { self as u128 }
    }
    impl EndianRepr for u64 {
        fn from_u128(v: u128) -> Self { v as u64 }
        fn to_u128(self) -> u128 { self as u128 }
    }
    impl EndianRepr for u128 {
        fn from_u128(v: u128) -> Self { v }
        fn to_u128(self) -> u128 { self }
    }
    impl EndianRepr for f32 {
        fn from_u128(v: u128) -> Self { f32::from_bits(v as u32) }
        fn to_u128(self) -> u128 { self.to_bits() as u128 }
    }
    impl EndianRepr for f64 {
        fn from_u128(v: u128) -> Self { f64::from_bits(v as u64) }
        fn to_u128(self) -> u128 { self.to_bits() as u128 }
    }

    // read_be_from_slice/read_le_from_slice removed: use `FromSlice` impls
    // and the convenience `read_from_slice` function below instead.


    // (private write helpers removed; use `write_to_extend` on the FromSlice impl)

    /// Read raw bytes into a `BigEndian<T>`.
    ///
    /// Short helper; avoids `_be`/`_le` in the public API name.
    ///
    /// Example:
    ///
    /// ```ignore
    /// use simple_endian::io::core_io;
    /// let data: [u8; 2] = [0x12, 0x34];
    /// let be: simple_endian::BigEndian<u16> = core_io::from_bytes(&data).unwrap();
    /// assert_eq!(be.to_native(), 0x1234u16);
    /// ```
    /// Zero-allocation enum that holds either a `BigEndian<T>` or
    /// `LittleEndian<T>` value. This replaces an earlier boxed trait-based
    /// approach to avoid heap allocation.
    pub enum EndianValue<T>
    where
        T: crate::SpecificEndian<T>,
    {
        Big(BigEndian<T>),
        Little(LittleEndian<T>),
    }

    impl<T> EndianValue<T>
    where
        T: crate::SpecificEndian<T> + Copy,
    {
        /// Convert the stored endian wrapper into the host-native `T`.
        pub fn to_native(&self) -> T {
            match self {
                EndianValue::Big(b) => b.to_native(),
                EndianValue::Little(l) => l.to_native(),
            }
        }
    }

    /// Read raw bytes and return an `EndianValue<T>` chosen by
    /// `T::default().endian()`.
    pub fn from_bytes<T>(data: &[u8]) -> Result<EndianValue<T>, &'static str>
    where
        T: crate::SpecificEndian<T> + Default + Copy + EndianRepr,
    {
        let default = T::default();
        match default.endian() {
            crate::Endian::Big => read_from_slice::<BigEndian<T>>(data).map(EndianValue::Big),
            crate::Endian::Little => read_from_slice::<LittleEndian<T>>(data).map(EndianValue::Little),
        }
    }

    /// Convenience: read bytes and return the host-native `T` directly.
    pub fn from_bytes_to_native<T>(data: &[u8]) -> Result<T, &'static str>
    where
        T: crate::SpecificEndian<T> + Default + Copy + EndianRepr,
    {
        Ok(from_bytes::<T>(data)?.to_native())
    }
    

    /// Trait describing types that can be read from / written to a byte slice
    /// representation. Implemented for `BigEndian<T>` and `LittleEndian<T>`.
    pub trait FromSlice: Sized {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str>;
        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str>;
    }

    /// Convenience generic helpers.
    pub fn read_from_slice<E: FromSlice>(data: &[u8]) -> Result<E, &'static str> {
        E::read_from_slice(data)
    }

    pub fn write_to_extend<E: FromSlice>(v: &E, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
        v.write_to_extend(out)
    }

    impl<T> FromSlice for BigEndian<T>
    where
        T: SpecificEndian<T> + Copy + EndianRepr,
    {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            if data.len() < size_of::<T>() {
                return Err("insufficient data");
            }
            let buf = &data[..size_of::<T>()];
            let v = match size_of::<T>() {
                1 => T::from_u128(buf[0] as u128),
                2 => {
                    let mut a = [0u8; 2];
                    a.copy_from_slice(buf);
                    let x = u16::from_be_bytes(a);
                    T::from_u128(x as u128)
                }
                4 => {
                    let mut a = [0u8; 4];
                    a.copy_from_slice(buf);
                    let x = u32::from_be_bytes(a);
                    T::from_u128(x as u128)
                }
                8 => {
                    let mut a = [0u8; 8];
                    a.copy_from_slice(buf);
                    let x = u64::from_be_bytes(a);
                    T::from_u128(x as u128)
                }
                16 => {
                    let mut a = [0u8; 16];
                    a.copy_from_slice(buf);
                    let x = u128::from_be_bytes(a);
                    T::from_u128(x)
                }
                _ => return Err("unsupported size"),
            };
            Ok(BigEndian::from(v))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            // Write bytes in big-endian order for the logical value.
            let repr = self.to_native().to_u128();
            match size_of::<T>() {
                1 => {
                    let b = repr as u8;
                    out.extend(core::iter::IntoIterator::into_iter([b]));
                    Ok(())
                }
                2 => {
                    let x = repr as u16;
                    out.extend(core::iter::IntoIterator::into_iter(x.to_be_bytes()));
                    Ok(())
                }
                4 => {
                    let x = repr as u32;
                    out.extend(core::iter::IntoIterator::into_iter(x.to_be_bytes()));
                    Ok(())
                }
                8 => {
                    let x = repr as u64;
                    out.extend(core::iter::IntoIterator::into_iter(x.to_be_bytes()));
                    Ok(())
                }
                16 => {
                    let x = repr as u128;
                    out.extend(core::iter::IntoIterator::into_iter(x.to_be_bytes()));
                    Ok(())
                }
                _ => Err("unsupported size"),
            }
        }
    }

    impl<T> FromSlice for LittleEndian<T>
    where
        T: SpecificEndian<T> + Copy + EndianRepr,
    {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            if data.len() < size_of::<T>() {
                return Err("insufficient data");
            }
            let buf = &data[..size_of::<T>()];
            let v = match size_of::<T>() {
                1 => T::from_u128(buf[0] as u128),
                2 => {
                    let mut a = [0u8; 2];
                    a.copy_from_slice(buf);
                    let x = u16::from_le_bytes(a);
                    T::from_u128(x as u128)
                }
                4 => {
                    let mut a = [0u8; 4];
                    a.copy_from_slice(buf);
                    let x = u32::from_le_bytes(a);
                    T::from_u128(x as u128)
                }
                8 => {
                    let mut a = [0u8; 8];
                    a.copy_from_slice(buf);
                    let x = u64::from_le_bytes(a);
                    T::from_u128(x as u128)
                }
                16 => {
                    let mut a = [0u8; 16];
                    a.copy_from_slice(buf);
                    let x = u128::from_le_bytes(a);
                    T::from_u128(x)
                }
                _ => return Err("unsupported size"),
            };
            Ok(LittleEndian::from(v))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            let repr = self.to_native().to_u128();
            match size_of::<T>() {
                1 => {
                    let b = repr as u8;
                    out.extend(core::iter::IntoIterator::into_iter([b]));
                    Ok(())
                }
                2 => {
                    let x = repr as u16;
                    out.extend(core::iter::IntoIterator::into_iter(x.to_le_bytes()));
                    Ok(())
                }
                4 => {
                    let x = repr as u32;
                    out.extend(core::iter::IntoIterator::into_iter(x.to_le_bytes()));
                    Ok(())
                }
                8 => {
                    let x = repr as u64;
                    out.extend(core::iter::IntoIterator::into_iter(x.to_le_bytes()));
                    Ok(())
                }
                16 => {
                    let x = repr as u128;
                    out.extend(core::iter::IntoIterator::into_iter(x.to_le_bytes()));
                    Ok(())
                }
                _ => Err("unsupported size"),
            }
        }
    }

    // --- Fixed UTF helpers (feature-gated) ---

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> FromSlice for crate::FixedUtf16BeCodeUnits<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            if data.len() < 2 * N {
                return Err("insufficient data");
            }

            let mut out = [crate::BigEndian::<u16>::from_bits(0); N];
            let mut i = 0;
            while i < N {
                let base = 2 * i;
                let v = u16::from_be_bytes([data[base], data[base + 1]]);
                out[i] = crate::BigEndian::<u16>::from_bits(v);
                i += 1;
            }

            Ok(crate::FixedUtf16BeCodeUnits::from(out))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            for cu in self.as_units() {
                out.extend(core::iter::IntoIterator::into_iter(cu.to_bits().to_be_bytes()));
            }
            Ok(())
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> FromSlice for crate::FixedUtf16LeCodeUnits<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            if data.len() < 2 * N {
                return Err("insufficient data");
            }

            let mut out = [crate::LittleEndian::<u16>::from_bits(0); N];
            let mut i = 0;
            while i < N {
                let base = 2 * i;
                let v = u16::from_le_bytes([data[base], data[base + 1]]);
                out[i] = crate::LittleEndian::<u16>::from_bits(v);
                i += 1;
            }

            Ok(crate::FixedUtf16LeCodeUnits::from(out))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            for cu in self.as_units() {
                out.extend(core::iter::IntoIterator::into_iter(cu.to_bits().to_le_bytes()));
            }
            Ok(())
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> FromSlice for crate::FixedUtf32BeCodeUnits<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            if data.len() < 4 * N {
                return Err("insufficient data");
            }

            let mut out = [crate::BigEndian::<u32>::from_bits(0); N];
            let mut i = 0;
            while i < N {
                let base = 4 * i;
                let v = u32::from_be_bytes([data[base], data[base + 1], data[base + 2], data[base + 3]]);
                out[i] = crate::BigEndian::<u32>::from_bits(v);
                i += 1;
            }

            Ok(crate::FixedUtf32BeCodeUnits::from(out))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            for cu in self.as_units() {
                out.extend(core::iter::IntoIterator::into_iter(cu.to_bits().to_be_bytes()));
            }
            Ok(())
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> FromSlice for crate::FixedUtf32LeCodeUnits<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            if data.len() < 4 * N {
                return Err("insufficient data");
            }

            let mut out = [crate::LittleEndian::<u32>::from_bits(0); N];
            let mut i = 0;
            while i < N {
                let base = 4 * i;
                let v = u32::from_le_bytes([data[base], data[base + 1], data[base + 2], data[base + 3]]);
                out[i] = crate::LittleEndian::<u32>::from_bits(v);
                i += 1;
            }

            Ok(crate::FixedUtf32LeCodeUnits::from(out))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            for cu in self.as_units() {
                out.extend(core::iter::IntoIterator::into_iter(cu.to_bits().to_le_bytes()));
            }
            Ok(())
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> FromSlice for crate::FixedUtf16BeNullPadded<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            Ok(crate::FixedUtf16BeNullPadded::from(
                <crate::FixedUtf16BeCodeUnits<N> as FromSlice>::read_from_slice(data)?,
            ))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            <crate::FixedUtf16BeCodeUnits<N> as FromSlice>::write_to_extend(&self.0, out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> FromSlice for crate::FixedUtf16BeSpacePadded<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            Ok(crate::FixedUtf16BeSpacePadded::from(
                <crate::FixedUtf16BeCodeUnits<N> as FromSlice>::read_from_slice(data)?,
            ))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            <crate::FixedUtf16BeCodeUnits<N> as FromSlice>::write_to_extend(&self.0, out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> FromSlice for crate::FixedUtf16LeNullPadded<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            Ok(crate::FixedUtf16LeNullPadded::from(
                <crate::FixedUtf16LeCodeUnits<N> as FromSlice>::read_from_slice(data)?,
            ))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            <crate::FixedUtf16LeCodeUnits<N> as FromSlice>::write_to_extend(&self.0, out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> FromSlice for crate::FixedUtf16LeSpacePadded<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            Ok(crate::FixedUtf16LeSpacePadded::from(
                <crate::FixedUtf16LeCodeUnits<N> as FromSlice>::read_from_slice(data)?,
            ))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            <crate::FixedUtf16LeCodeUnits<N> as FromSlice>::write_to_extend(&self.0, out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> FromSlice for crate::FixedUtf32BeNullPadded<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            Ok(crate::FixedUtf32BeNullPadded::from(
                <crate::FixedUtf32BeCodeUnits<N> as FromSlice>::read_from_slice(data)?,
            ))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            <crate::FixedUtf32BeCodeUnits<N> as FromSlice>::write_to_extend(&self.0, out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> FromSlice for crate::FixedUtf32BeSpacePadded<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            Ok(crate::FixedUtf32BeSpacePadded::from(
                <crate::FixedUtf32BeCodeUnits<N> as FromSlice>::read_from_slice(data)?,
            ))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            <crate::FixedUtf32BeCodeUnits<N> as FromSlice>::write_to_extend(&self.0, out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> FromSlice for crate::FixedUtf32LeNullPadded<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            Ok(crate::FixedUtf32LeNullPadded::from(
                <crate::FixedUtf32LeCodeUnits<N> as FromSlice>::read_from_slice(data)?,
            ))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            <crate::FixedUtf32LeCodeUnits<N> as FromSlice>::write_to_extend(&self.0, out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> FromSlice for crate::FixedUtf32LeSpacePadded<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            Ok(crate::FixedUtf32LeSpacePadded::from(
                <crate::FixedUtf32LeCodeUnits<N> as FromSlice>::read_from_slice(data)?,
            ))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            <crate::FixedUtf32LeCodeUnits<N> as FromSlice>::write_to_extend(&self.0, out)
        }
    }
}

// Std-backed Read/Write wrappers: enabled under `io-std` which depends on `io-core`.
#[cfg(feature = "io-std")]
pub mod std_io {
    use super::core_io;
    use crate::{BigEndian, LittleEndian};
    use std::io::{self, Read, Write};
    use core::mem::size_of;

    fn read_be<R, T>(reader: &mut R) -> io::Result<BigEndian<T>>
    where
        R: Read,
        T: crate::SpecificEndian<T> + Default + Copy + core_io::EndianRepr,
    {
        let mut buf = vec![0u8; size_of::<T>()];
        reader.read_exact(&mut buf)?;
    core_io::read_from_slice::<BigEndian<T>>(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
    }

    fn read_le<R, T>(reader: &mut R) -> io::Result<LittleEndian<T>>
    where
        R: Read,
        T: crate::SpecificEndian<T> + Default + Copy + core_io::EndianRepr,
    {
        let mut buf = vec![0u8; size_of::<T>()];
        reader.read_exact(&mut buf)?;
    core_io::read_from_slice::<LittleEndian<T>>(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
    }

    fn write_be<W, T>(writer: &mut W, v: &BigEndian<T>) -> io::Result<()>
    where
        W: Write,
        T: crate::SpecificEndian<T> + Copy + core_io::EndianRepr,
    {
        let mut out = Vec::new();
        core_io::write_to_extend(v, &mut out).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        writer.write_all(&out)
    }

    fn write_le<W, T>(writer: &mut W, v: &LittleEndian<T>) -> io::Result<()>
    where
        W: Write,
        T: crate::SpecificEndian<T> + Copy + core_io::EndianRepr,
    {
        let mut out = Vec::new();
        core_io::write_to_extend(v, &mut out).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        writer.write_all(&out)
    }

    pub trait EndianRead: Sized {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self>;
    }

    pub trait EndianWrite {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()>;
    }

    impl<T> EndianRead for BigEndian<T>
    where
        T: crate::SpecificEndian<T> + Default + Copy + core_io::EndianRepr,
    {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            read_be::<R, T>(reader)
        }
    }

    impl<T> EndianRead for LittleEndian<T>
    where
        T: crate::SpecificEndian<T> + Default + Copy + core_io::EndianRepr,
    {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            read_le::<R, T>(reader)
        }
    }

    impl<T> EndianWrite for BigEndian<T>
    where
        T: crate::SpecificEndian<T> + Copy + core_io::EndianRepr,
    {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            write_be::<W, T>(writer, self)
        }
    }

    impl<T> EndianWrite for LittleEndian<T>
    where
        T: crate::SpecificEndian<T> + Copy + core_io::EndianRepr,
    {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            write_le::<W, T>(writer, self)
        }
    }

    pub fn read_specific<R, E>(reader: &mut R) -> io::Result<E>
    where
        R: Read,
        E: EndianRead,
    {
        E::read_from(reader)
    }

    pub fn write_specific<W, E>(writer: &mut W, v: &E) -> io::Result<()>
    where
        W: Write,
        E: EndianWrite,
    {
        v.write_to(writer)
    }

    // --- Fixed UTF helpers (feature-gated) ---

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianRead for crate::FixedUtf16BeCodeUnits<N> {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 2 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianRead for crate::FixedUtf16BeNullPadded<N> {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 2 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf16BeNullPadded<N> {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianRead for crate::FixedUtf16BeSpacePadded<N> {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 2 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf16BeSpacePadded<N> {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf16BeCodeUnits<N> {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianRead for crate::FixedUtf16LeCodeUnits<N> {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 2 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianRead for crate::FixedUtf16LeNullPadded<N> {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 2 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf16LeNullPadded<N> {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianRead for crate::FixedUtf16LeSpacePadded<N> {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 2 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf16LeSpacePadded<N> {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf16LeCodeUnits<N> {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianRead for crate::FixedUtf32BeCodeUnits<N> {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 4 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianRead for crate::FixedUtf32BeNullPadded<N> {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 4 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf32BeNullPadded<N> {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianRead for crate::FixedUtf32BeSpacePadded<N> {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 4 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf32BeSpacePadded<N> {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf32BeCodeUnits<N> {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianRead for crate::FixedUtf32LeCodeUnits<N> {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 4 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianRead for crate::FixedUtf32LeNullPadded<N> {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 4 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf32LeNullPadded<N> {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianRead for crate::FixedUtf32LeSpacePadded<N> {
        fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 4 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf32LeSpacePadded<N> {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf32LeCodeUnits<N> {
        fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }
}

#[cfg(all(test, feature = "io-std"))]
mod tests {
    use super::std_io::*;
    use std::io::Cursor;
    use crate::{BigEndian, LittleEndian, SpecificEndian};

    fn round_trip_be<T>(val: T)
    where
        T: SpecificEndian<T> + Copy + PartialEq + core::fmt::Debug,
        BigEndian<T>: EndianWrite + EndianRead + From<T> + Into<T>,
    {
        let be: BigEndian<T> = BigEndian::from(val);
        let mut buf = Vec::new();
        write_specific(&mut buf, &be).unwrap();

        let mut cur = Cursor::new(buf);
        let out: BigEndian<T> = read_specific(&mut cur).unwrap();
        assert_eq!(out.to_native(), be.to_native());
    }

    fn round_trip_le<T>(val: T)
    where
        T: SpecificEndian<T> + Copy + PartialEq + core::fmt::Debug,
        LittleEndian<T>: EndianWrite + EndianRead + From<T> + Into<T>,
    {
        let le: LittleEndian<T> = LittleEndian::from(val);
        let mut buf = Vec::new();
        write_specific(&mut buf, &le).unwrap();

        let mut cur = Cursor::new(buf);
        let out: LittleEndian<T> = read_specific(&mut cur).unwrap();
        assert_eq!(out.to_native(), le.to_native());
    }

    #[test]
    fn be_u16_round_trip() {
        round_trip_be::<u16>(0x1234);
    }

    #[test]
    fn le_u16_round_trip() {
        round_trip_le::<u16>(0x1234);
    }

    #[test]
    fn be_u32_round_trip() {
        round_trip_be::<u32>(0x12345678);
    }

    #[test]
    fn le_u32_round_trip() {
        round_trip_le::<u32>(0x12345678);
    }

    #[test]
    fn be_u64_round_trip() {
        round_trip_be::<u64>(0x1234567890abcdef);
    }

    #[test]
    fn le_u64_round_trip() {
        round_trip_le::<u64>(0x1234567890abcdef);
    }

    #[test]
    fn be_f32_round_trip() {
        round_trip_be::<f32>(1234.5);
    }

    #[test]
    fn le_f32_round_trip() {
        round_trip_le::<f32>(1234.5);
    }

    #[test]
    fn be_f64_round_trip() {
        round_trip_be::<f64>(1234567.89);
    }

    #[test]
    fn le_f64_round_trip() {
        round_trip_le::<f64>(1234567.89);
    }

    #[test]
    fn multiple_sequence_read() {
        // Write sequence: u16be, u32le, u8be
        let a: BigEndian<u16> = 0xfaceu16.into();
        let b: LittleEndian<u32> = 0xdeadbeefu32.into();
        let c: BigEndian<u8> = 0x7fu8.into();

        let mut buf = Vec::new();
        write_specific(&mut buf, &a).unwrap();
        write_specific(&mut buf, &b).unwrap();
        write_specific(&mut buf, &c).unwrap();

        let mut cur = Cursor::new(buf);
        let ra: BigEndian<u16> = read_specific(&mut cur).unwrap();
        let rb: LittleEndian<u32> = read_specific(&mut cur).unwrap();
        let rc: BigEndian<u8> = read_specific(&mut cur).unwrap();

        assert_eq!(ra.to_native(), a.to_native());
        assert_eq!(rb.to_native(), b.to_native());
        assert_eq!(rc.to_native(), c.to_native());
    }

    #[test]
    fn insufficient_bytes_error() {
        // Create a buffer too small for u64
        let mut cur = Cursor::new(vec![0u8; 3]);
        let res: std::io::Result<BigEndian<u64>> = read_specific(&mut cur);
        assert!(res.is_err());
    }
}

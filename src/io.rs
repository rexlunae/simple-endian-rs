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
        fn from_u128(v: u128) -> Self {
            v as u8
        }
        fn to_u128(self) -> u128 {
            self as u128
        }
    }
    impl EndianRepr for u16 {
        fn from_u128(v: u128) -> Self {
            v as u16
        }
        fn to_u128(self) -> u128 {
            self as u128
        }
    }
    impl EndianRepr for u32 {
        fn from_u128(v: u128) -> Self {
            v as u32
        }
        fn to_u128(self) -> u128 {
            self as u128
        }
    }
    impl EndianRepr for u64 {
        fn from_u128(v: u128) -> Self {
            v as u64
        }
        fn to_u128(self) -> u128 {
            self as u128
        }
    }
    impl EndianRepr for u128 {
        fn from_u128(v: u128) -> Self {
            v
        }
        fn to_u128(self) -> u128 {
            self
        }
    }
    impl EndianRepr for f32 {
        fn from_u128(v: u128) -> Self {
            f32::from_bits(v as u32)
        }
        fn to_u128(self) -> u128 {
            self.to_bits() as u128
        }
    }
    impl EndianRepr for f64 {
        fn from_u128(v: u128) -> Self {
            f64::from_bits(v as u64)
        }
        fn to_u128(self) -> u128 {
            self.to_bits() as u128
        }
    }

    // --- Tuple support ------------------------------------------------------
    //
    // Tuples are encoded as a *concatenation* of each element's bytes.
    //
    // This crate's core IO machinery for `BigEndian<T>` / `LittleEndian<T>` uses a
    // u128-based intermediate representation, so for tuples we interpret that u128 as
    // the raw bytes (big-endian or little-endian depending on the wrapper) packed
    // into a u128.
    //
    // Encoding contract for a tuple `(A, B, ...)`:
    // - `to_u128()` returns a u128 whose *big-endian byte representation* is the
    //   concatenation of each element's big-endian bytes.
    // - `from_u128()` reverses that process.
    //
    // With this convention, the existing `FromSlice for BigEndian<T>/LittleEndian<T>`
    // implementations work for tuples too.

    macro_rules! impl_endianrepr_for_tuple {
        ( $( ($idx:tt, $T:ident) ),+ $(,)? ) => {
            impl<$( $T ),+> EndianRepr for ( $( $T ),+ )
            where
                $( $T: EndianRepr + Copy ),+
            {
                #[allow(unused_assignments)]
                fn from_u128(v: u128) -> Self {
                    let bytes = v.to_be_bytes();
                    let total = 0usize $( + size_of::<$T>() )+;
                    if total > 16 {
                        panic!("tuple EndianRepr total size exceeds 16 bytes");
                    }
                    // The compact value is right-aligned within the u128.
                    let mut offset = 16usize - total;
                    (
                        $(
                            {
                                let n = size_of::<$T>();
                                let chunk = &bytes[offset..offset + n];
                                let part = match n {
                                    1 => chunk[0] as u128,
                                    2 => u16::from_be_bytes([chunk[0], chunk[1]]) as u128,
                                    4 => u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as u128,
                                    8 => u64::from_be_bytes([
                                        chunk[0], chunk[1], chunk[2], chunk[3],
                                        chunk[4], chunk[5], chunk[6], chunk[7],
                                    ]) as u128,
                                    16 => u128::from_be_bytes(chunk.try_into().unwrap()),
                                    _ => panic!("unsupported size in tuple EndianRepr"),
                                };
                                offset += n;
                                <$T as EndianRepr>::from_u128(part)
                            }
                        ),+
                    )
                }

                #[allow(unused_assignments)]
                fn to_u128(self) -> u128 {
                    let mut bytes = [0u8; 16];
                    let total = 0usize $( + size_of::<$T>() )+;
                    if total > 16 {
                        panic!("tuple EndianRepr total size exceeds 16 bytes");
                    }
                    // Right-align data within the u128 so `BigEndian<T>` reads/writes
                    // can slice the last `size_of::<T>()` bytes.
                    let mut offset = 16usize - total;
                    $(
                        {
                            let n = size_of::<$T>();
                            let part = self.$idx.to_u128();
                            match n {
                                1 => bytes[offset] = part as u8,
                                2 => bytes[offset..offset + 2].copy_from_slice(&(part as u16).to_be_bytes()),
                                4 => bytes[offset..offset + 4].copy_from_slice(&(part as u32).to_be_bytes()),
                                8 => bytes[offset..offset + 8].copy_from_slice(&(part as u64).to_be_bytes()),
                                16 => bytes[offset..offset + 16].copy_from_slice(&(part as u128).to_be_bytes()),
                                _ => panic!("unsupported size in tuple EndianRepr"),
                            }
                            offset += n;
                        }
                    )+
                    u128::from_be_bytes(bytes)
                }
            }
        };
    }

    impl_endianrepr_for_tuple!((0, A), (1, B));
    impl_endianrepr_for_tuple!((0, A), (1, B), (2, C));
    impl_endianrepr_for_tuple!((0, A), (1, B), (2, C), (3, D));
    impl_endianrepr_for_tuple!((0, A), (1, B), (2, C), (3, D), (4, E));
    impl_endianrepr_for_tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F));
    impl_endianrepr_for_tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G));
    impl_endianrepr_for_tuple!(
        (0, A),
        (1, B),
        (2, C),
        (3, D),
        (4, E),
        (5, F),
        (6, G),
        (7, H)
    );
    impl_endianrepr_for_tuple!(
        (0, A),
        (1, B),
        (2, C),
        (3, D),
        (4, E),
        (5, F),
        (6, G),
        (7, H),
        (8, I)
    );
    impl_endianrepr_for_tuple!(
        (0, A),
        (1, B),
        (2, C),
        (3, D),
        (4, E),
        (5, F),
        (6, G),
        (7, H),
        (8, I),
        (9, J)
    );
    impl_endianrepr_for_tuple!(
        (0, A),
        (1, B),
        (2, C),
        (3, D),
        (4, E),
        (5, F),
        (6, G),
        (7, H),
        (8, I),
        (9, J),
        (10, K)
    );
    impl_endianrepr_for_tuple!(
        (0, A),
        (1, B),
        (2, C),
        (3, D),
        (4, E),
        (5, F),
        (6, G),
        (7, H),
        (8, I),
        (9, J),
        (10, K),
        (11, L)
    );

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
            crate::Endian::Little => {
                read_from_slice::<LittleEndian<T>>(data).map(EndianValue::Little)
            }
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

    pub fn write_to_extend<E: FromSlice>(
        v: &E,
        out: &mut impl Extend<u8>,
    ) -> Result<(), &'static str> {
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
            // For composite types (like tuples), `EndianRepr` treats the u128 as a
            // compact byte container and expects the bytes to be right-aligned.
            let mut a = [0u8; 16];
            a[16 - buf.len()..].copy_from_slice(buf);
            let v = T::from_u128(u128::from_be_bytes(a));
            Ok(BigEndian::from(v))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            // Write bytes in big-endian order for the logical value.
            //
            // For tuples, `to_u128()` acts as a byte container (right-aligned).
            let repr = self.to_native().to_u128();
            let bytes = repr.to_be_bytes();
            let n = size_of::<T>();
            if !(n == 1 || n == 2 || n == 4 || n == 8 || n == 16) {
                return Err("unsupported size");
            }
            // Emit the last `n` bytes (compact encoding).
            out.extend(core::iter::IntoIterator::into_iter(
                bytes[16 - n..].iter().copied(),
            ));
            Ok(())
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
            // Composite types treat the u128 as a compact byte container.
            let mut a = [0u8; 16];
            a[..buf.len()].copy_from_slice(buf);
            let v = T::from_u128(u128::from_le_bytes(a));
            Ok(LittleEndian::from(v))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            let repr = self.to_native().to_u128();
            let bytes = repr.to_le_bytes();
            let n = size_of::<T>();
            if !(n == 1 || n == 2 || n == 4 || n == 8 || n == 16) {
                return Err("unsupported size");
            }
            out.extend(core::iter::IntoIterator::into_iter(
                bytes[..n].iter().copied(),
            ));
            Ok(())
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
                // Parse standard UTF-16BE wire bytes into a native scalar.
                let native = u16::from_be_bytes([data[base], data[base + 1]]);
                // Store as correctly endian-tagged bits.
                #[cfg(target_endian = "big")]
                {
                    out[i] = crate::BigEndian::<u16>::from_bits(native);
                }
                #[cfg(target_endian = "little")]
                {
                    out[i] = crate::BigEndian::<u16>::from_bits(native.swap_bytes());
                }
                i += 1;
            }

            Ok(crate::FixedUtf16BeCodeUnits::from(out))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            for cu in self.as_units() {
                // Serialize the *native scalar* as UTF-16BE on the wire.
                out.extend(core::iter::IntoIterator::into_iter(
                    cu.to_native().to_be_bytes(),
                ));
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
                // Parse standard UTF-16LE wire bytes into a native scalar.
                let native = u16::from_le_bytes([data[base], data[base + 1]]);
                // Store as correctly endian-tagged bits.
                #[cfg(target_endian = "little")]
                {
                    out[i] = crate::LittleEndian::<u16>::from_bits(native);
                }
                #[cfg(target_endian = "big")]
                {
                    out[i] = crate::LittleEndian::<u16>::from_bits(native.swap_bytes());
                }
                i += 1;
            }

            Ok(crate::FixedUtf16LeCodeUnits::from(out))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            for cu in self.as_units() {
                // Serialize the *native scalar* as UTF-16LE on the wire.
                out.extend(core::iter::IntoIterator::into_iter(
                    cu.to_native().to_le_bytes(),
                ));
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
                // Parse standard UTF-32BE wire bytes into a native scalar.
                let native = u32::from_be_bytes([
                    data[base],
                    data[base + 1],
                    data[base + 2],
                    data[base + 3],
                ]);
                // Store as correctly endian-tagged bits.
                #[cfg(target_endian = "big")]
                {
                    out[i] = crate::BigEndian::<u32>::from_bits(native);
                }
                #[cfg(target_endian = "little")]
                {
                    out[i] = crate::BigEndian::<u32>::from_bits(native.swap_bytes());
                }
                i += 1;
            }

            Ok(crate::FixedUtf32BeCodeUnits::from(out))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            for cu in self.as_units() {
                // Serialize the *native scalar* as UTF-32BE on the wire.
                out.extend(core::iter::IntoIterator::into_iter(
                    cu.to_native().to_be_bytes(),
                ));
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
                // Parse standard UTF-32LE wire bytes into a native scalar.
                let native = u32::from_le_bytes([
                    data[base],
                    data[base + 1],
                    data[base + 2],
                    data[base + 3],
                ]);
                // Store as correctly endian-tagged bits.
                #[cfg(target_endian = "little")]
                {
                    out[i] = crate::LittleEndian::<u32>::from_bits(native);
                }
                #[cfg(target_endian = "big")]
                {
                    out[i] = crate::LittleEndian::<u32>::from_bits(native.swap_bytes());
                }
                i += 1;
            }

            Ok(crate::FixedUtf32LeCodeUnits::from(out))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            for cu in self.as_units() {
                // Serialize the *native scalar* as UTF-32LE on the wire.
                out.extend(core::iter::IntoIterator::into_iter(
                    cu.to_native().to_le_bytes(),
                ));
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

    #[cfg(all(feature = "text_fixed", feature = "text_utf8"))]
    impl<const N: usize> FromSlice for crate::FixedUtf8NullPadded<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            if data.len() < N {
                return Err("insufficient data");
            }
            let mut out = [0u8; N];
            out.copy_from_slice(&data[..N]);
            Ok(crate::FixedUtf8NullPadded::from(
                crate::FixedUtf8Bytes::from(out),
            ))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            out.extend(self.0.as_bytes().iter().copied());
            Ok(())
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf8"))]
    impl<const N: usize> FromSlice for crate::FixedUtf8SpacePadded<N> {
        fn read_from_slice(data: &[u8]) -> Result<Self, &'static str> {
            if data.len() < N {
                return Err("insufficient data");
            }
            let mut out = [0u8; N];
            out.copy_from_slice(&data[..N]);
            Ok(crate::FixedUtf8SpacePadded::from(
                crate::FixedUtf8Bytes::from(out),
            ))
        }

        fn write_to_extend(&self, out: &mut impl Extend<u8>) -> Result<(), &'static str> {
            out.extend(self.0.as_bytes().iter().copied());
            Ok(())
        }
    }
}

// Std-backed Read/Write wrappers: enabled under `io-std` which depends on `io-core`.
#[cfg(feature = "io-std")]
pub mod std_io {
    use super::core_io;
    use crate::{BigEndian, LittleEndian};
    use core::any::TypeId;
    use core::mem::size_of;
    use std::io::{self, Read, Write};

    fn read_be<R, T>(reader: &mut R) -> io::Result<BigEndian<T>>
    where
        R: Read + ?Sized,
        T: crate::SpecificEndian<T> + Default + Copy + core_io::EndianRepr + 'static,
    {
        // Fast paths for common primitives to avoid heap allocation.
        if TypeId::of::<T>() == TypeId::of::<u16>() {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;
            let v = u16::from_be_bytes(buf);
            // SAFETY: We just proved T == u16.
            let v: T = unsafe { core::mem::transmute_copy(&v) };
            return Ok(BigEndian::from(v));
        }
        if TypeId::of::<T>() == TypeId::of::<u32>() {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            let v = u32::from_be_bytes(buf);
            let v: T = unsafe { core::mem::transmute_copy(&v) };
            return Ok(BigEndian::from(v));
        }
        if TypeId::of::<T>() == TypeId::of::<u64>() {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf)?;
            let v = u64::from_be_bytes(buf);
            let v: T = unsafe { core::mem::transmute_copy(&v) };
            return Ok(BigEndian::from(v));
        }

        let mut buf = vec![0u8; size_of::<T>()];
        reader.read_exact(&mut buf)?;
        core_io::read_from_slice::<BigEndian<T>>(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
    }

    fn read_le<R, T>(reader: &mut R) -> io::Result<LittleEndian<T>>
    where
        R: Read + ?Sized,
        T: crate::SpecificEndian<T> + Default + Copy + core_io::EndianRepr + 'static,
    {
        if TypeId::of::<T>() == TypeId::of::<u16>() {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;
            let v = u16::from_le_bytes(buf);
            let v: T = unsafe { core::mem::transmute_copy(&v) };
            return Ok(LittleEndian::from(v));
        }
        if TypeId::of::<T>() == TypeId::of::<u32>() {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            let v = u32::from_le_bytes(buf);
            let v: T = unsafe { core::mem::transmute_copy(&v) };
            return Ok(LittleEndian::from(v));
        }
        if TypeId::of::<T>() == TypeId::of::<u64>() {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf)?;
            let v = u64::from_le_bytes(buf);
            let v: T = unsafe { core::mem::transmute_copy(&v) };
            return Ok(LittleEndian::from(v));
        }

        let mut buf = vec![0u8; size_of::<T>()];
        reader.read_exact(&mut buf)?;
        core_io::read_from_slice::<LittleEndian<T>>(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
    }

    fn write_be<W, T>(writer: &mut W, v: &BigEndian<T>) -> io::Result<()>
    where
        W: Write + ?Sized,
        T: crate::SpecificEndian<T> + Copy + core_io::EndianRepr + 'static,
    {
        if TypeId::of::<T>() == TypeId::of::<u16>() {
            let n: u16 = unsafe { core::mem::transmute_copy(&v.to_native()) };
            return writer.write_all(&n.to_be_bytes());
        }
        if TypeId::of::<T>() == TypeId::of::<u32>() {
            let n: u32 = unsafe { core::mem::transmute_copy(&v.to_native()) };
            return writer.write_all(&n.to_be_bytes());
        }
        if TypeId::of::<T>() == TypeId::of::<u64>() {
            let n: u64 = unsafe { core::mem::transmute_copy(&v.to_native()) };
            return writer.write_all(&n.to_be_bytes());
        }

        let mut out = Vec::new();
        core_io::write_to_extend(v, &mut out)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        writer.write_all(&out)
    }

    fn write_le<W, T>(writer: &mut W, v: &LittleEndian<T>) -> io::Result<()>
    where
        W: Write + ?Sized,
        T: crate::SpecificEndian<T> + Copy + core_io::EndianRepr + 'static,
    {
        if TypeId::of::<T>() == TypeId::of::<u16>() {
            let n: u16 = unsafe { core::mem::transmute_copy(&v.to_native()) };
            return writer.write_all(&n.to_le_bytes());
        }
        if TypeId::of::<T>() == TypeId::of::<u32>() {
            let n: u32 = unsafe { core::mem::transmute_copy(&v.to_native()) };
            return writer.write_all(&n.to_le_bytes());
        }
        if TypeId::of::<T>() == TypeId::of::<u64>() {
            let n: u64 = unsafe { core::mem::transmute_copy(&v.to_native()) };
            return writer.write_all(&n.to_le_bytes());
        }

        let mut out = Vec::new();
        core_io::write_to_extend(v, &mut out)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        writer.write_all(&out)
    }

    pub trait EndianRead: Sized {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self>;
    }

    pub trait EndianWrite {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()>;
    }

    impl<T> EndianRead for BigEndian<T>
    where
        T: crate::SpecificEndian<T> + Default + Copy + core_io::EndianRepr + 'static,
    {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            read_be::<R, T>(reader)
        }
    }

    impl<T> EndianRead for LittleEndian<T>
    where
        T: crate::SpecificEndian<T> + Default + Copy + core_io::EndianRepr + 'static,
    {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            read_le::<R, T>(reader)
        }
    }

    impl<T> EndianWrite for BigEndian<T>
    where
        T: crate::SpecificEndian<T> + Copy + core_io::EndianRepr + 'static,
    {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            write_be::<W, T>(writer, self)
        }
    }

    impl<T> EndianWrite for LittleEndian<T>
    where
        T: crate::SpecificEndian<T> + Copy + core_io::EndianRepr + 'static,
    {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            write_le::<W, T>(writer, self)
        }
    }

    // Tuple support lives at the `SpecificEndian` layer.
    //
    // Note: We intentionally do *not* provide specialized std-IO impls for
    // `BigEndian<(..)>` / `LittleEndian<(..)>` here because the blanket impls
    // above (`impl<T> EndianRead/EndianWrite for BigEndian<T>`) already cover
    // tuples once they implement `core_io::EndianRepr`. Adding explicit tuple
    // impls causes trait coherence conflicts (E0119).

    impl<const N: usize> EndianRead for [u8; N] {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = [0u8; N];
            reader.read_exact(&mut buf)?;
            Ok(buf)
        }
    }

    impl<const N: usize> EndianWrite for [u8; N] {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            writer.write_all(self)
        }
    }

    impl<E, const N: usize> EndianRead for [E; N]
    where
        E: EndianRead + Copy,
    {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut out = [E::read_from(reader)?; N];
            for i in 1..N {
                out[i] = E::read_from(reader)?;
            }
            Ok(out)
        }
    }

    impl<E, const N: usize> EndianWrite for [E; N]
    where
        E: EndianWrite,
    {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            for v in self {
                v.write_to(writer)?;
            }
            Ok(())
        }
    }

    /// Read an endian-aware value of type `E` from a reader.
    ///
    /// This helper works with both sized readers (e.g. `std::io::Cursor<Vec<u8>>`) and
    /// unsized trait objects like `&mut dyn std::io::Read`.
    ///
    /// In particular, this is designed to support the common “extension trait” pattern:
    ///
    /// ```rust
    /// use std::io::{self, Read};
    ///
    /// pub trait ReadBytesExt: Read {
    ///     fn read_u32_be(&mut self) -> io::Result<u32>;
    /// }
    ///
    /// impl<R: Read + ?Sized> ReadBytesExt for R {
    ///     fn read_u32_be(&mut self) -> io::Result<u32> {
    ///         let be: simple_endian::BigEndian<u32> = simple_endian::read_specific(self)?;
    ///         Ok(be.to_native())
    ///     }
    /// }
    ///
    /// fn read_from_dyn(r: &mut dyn Read) -> io::Result<u32> {
    ///     r.read_u32_be()
    /// }
    /// ```
    pub fn read_specific<R, E>(reader: &mut R) -> io::Result<E>
    where
        R: Read + ?Sized,
        E: EndianRead,
    {
        E::read_from(reader)
    }

    /// Write an endian-aware value of type `E` to a writer.
    ///
    /// Like [`read_specific`], this supports both sized writers and `&mut dyn std::io::Write`.
    pub fn write_specific<W, E>(writer: &mut W, v: &E) -> io::Result<()>
    where
        W: Write + ?Sized,
        E: EndianWrite,
    {
        v.write_to(writer)
    }

    /// Dyn-friendly adapter for `read_specific`.
    ///
    /// This is purely ergonomic: it lets consumers call the helper from
    /// `&mut dyn Read` contexts without having to name (or be generic over) the
    /// reader type.
    pub fn read_specific_dyn<E>(reader: &mut dyn Read) -> io::Result<E>
    where
        E: EndianRead,
    {
        read_specific::<dyn Read, E>(reader)
    }

    /// Dyn-friendly adapter for `write_specific`.
    ///
    /// This is purely ergonomic: it lets consumers call the helper from
    /// `&mut dyn Write` contexts without having to name (or be generic over) the
    /// writer type.
    pub fn write_specific_dyn<E>(writer: &mut dyn Write, v: &E) -> io::Result<()>
    where
        E: EndianWrite,
    {
        write_specific::<dyn Write, E>(writer, v)
    }

    /// Read a value in its *wire* representation and convert it into a native type.
    ///
    /// This is the recommended ergonomic pattern for `#[derive(Endianize)]` types:
    ///
    /// * the generated `*Wire` type is the IO/layout type (it implements [`EndianRead`])
    /// * your “real” type is the native type used throughout your program
    ///
    /// Conceptually:
    ///
    /// 1. read `W` from the stream (endian-correct)
    /// 2. convert into `T` using `From<W>`
    ///
    /// Under the hood, this is equivalent to:
    ///
    /// ```ignore
    /// let wire: W = read_specific(reader)?;
    /// let native: T = wire.into();
    /// ```
    ///
    /// ### Example
    ///
    /// ```ignore
    /// use simple_endian::Endianize;
    /// use simple_endian::read_native;
    ///
    /// #[derive(Endianize)]
    /// #[endian(le)]
    /// #[repr(C)]
    /// struct Header {
    ///     magic: u32,
    ///     version: u16,
    /// }
    ///
    /// // Reads `HeaderWire` and converts to `Header`.
    /// let header: Header = read_native::<_, HeaderWire, Header>(&mut reader)?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    ///
    /// ### Notes
    ///
    /// * This composes naturally for nested `Endianize` types (a `PacketWire` contains a `HeaderWire`).
    /// * For enums, the wire representation is `tag + union payload`, so you typically want to keep
    ///   trait derives on the native enum and only convert at the boundary.
    pub fn read_native<R, W, T>(reader: &mut R) -> io::Result<T>
    where
        R: Read + ?Sized,
        W: EndianRead,
        T: From<W>,
    {
        let wire: W = read_specific(reader)?;
        Ok(wire.into())
    }

    /// Like [`read_native`], but uses `TryFrom<W>` for fallible conversion.
    ///
    /// Conversion errors are mapped to `io::ErrorKind::InvalidData`.
    pub fn try_read_native<R, W, T>(reader: &mut R) -> io::Result<T>
    where
        R: Read + ?Sized,
        W: EndianRead,
        T: TryFrom<W>,
        <T as TryFrom<W>>::Error: core::fmt::Display,
    {
        let wire: W = read_specific(reader)?;
        T::try_from(wire).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    /// Convert a native value into its *wire* representation and write it.
    ///
    /// This is the “mirror” of [`read_native`]: convert first, then write.
    ///
    /// Under the hood this is equivalent to:
    ///
    /// ```ignore
    /// let wire: W = value.into();
    /// write_specific(writer, &wire)
    /// ```
    pub fn write_native<Wrt, W, T>(writer: &mut Wrt, v: T) -> io::Result<()>
    where
        Wrt: Write + ?Sized,
        W: EndianWrite,
        W: From<T>,
    {
        let wire: W = v.into();
        write_specific(writer, &wire)
    }

    /// Convenience wrapper over [`write_native`] when you only have a reference.
    ///
    /// This clones the value and forwards to [`write_native`].
    pub fn write_native_ref<Wrt, W, T>(writer: &mut Wrt, v: &T) -> io::Result<()>
    where
        Wrt: Write + ?Sized,
        W: EndianWrite,
        T: Clone,
        W: From<T>,
    {
        write_native::<Wrt, W, T>(writer, v.clone())
    }

    /// Like [`write_native`], but uses `TryFrom<T>` for fallible conversion.
    ///
    /// Conversion errors are mapped to `io::ErrorKind::InvalidInput`.
    pub fn try_write_native<Wrt, W, T>(writer: &mut Wrt, v: T) -> io::Result<()>
    where
        Wrt: Write + ?Sized,
        W: EndianWrite,
        W: TryFrom<T>,
        <W as TryFrom<T>>::Error: core::fmt::Display,
    {
        let wire: W = W::try_from(v)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;
        write_specific(writer, &wire)
    }

    /// Convenience wrapper over [`try_write_native`] when you only have a reference.
    ///
    /// This clones the value and forwards to [`try_write_native`].
    pub fn try_write_native_ref<Wrt, W, T>(writer: &mut Wrt, v: &T) -> io::Result<()>
    where
        Wrt: Write + ?Sized,
        W: EndianWrite,
        T: Clone,
        W: TryFrom<T>,
        <W as TryFrom<T>>::Error: core::fmt::Display,
    {
        try_write_native::<Wrt, W, T>(writer, v.clone())
    }

    // --- Fixed UTF helpers (feature-gated) ---

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianRead for crate::FixedUtf16BeCodeUnits<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 2 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianRead for crate::FixedUtf16BeNullPadded<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 2 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf16BeNullPadded<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianRead for crate::FixedUtf16BeSpacePadded<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 2 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf16BeSpacePadded<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf16BeCodeUnits<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianRead for crate::FixedUtf16LeCodeUnits<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 2 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianRead for crate::FixedUtf16LeNullPadded<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 2 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf16LeNullPadded<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianRead for crate::FixedUtf16LeSpacePadded<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 2 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf16LeSpacePadded<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf16"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf16LeCodeUnits<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianRead for crate::FixedUtf32BeCodeUnits<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 4 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianRead for crate::FixedUtf32BeNullPadded<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 4 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf32BeNullPadded<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianRead for crate::FixedUtf32BeSpacePadded<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 4 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf32BeSpacePadded<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf32BeCodeUnits<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianRead for crate::FixedUtf32LeCodeUnits<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 4 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianRead for crate::FixedUtf32LeNullPadded<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 4 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf32LeNullPadded<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianRead for crate::FixedUtf32LeSpacePadded<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; 4 * N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf32LeSpacePadded<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf32"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf32LeCodeUnits<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    // --- Fixed UTF-8 helpers (feature-gated) ---

    #[cfg(all(feature = "text_fixed", feature = "text_utf8"))]
    impl<const N: usize> EndianRead for crate::FixedUtf8NullPadded<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf8"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf8NullPadded<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
            let mut out = Vec::new();
            core_io::write_to_extend(self, &mut out)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            writer.write_all(&out)
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf8"))]
    impl<const N: usize> EndianRead for crate::FixedUtf8SpacePadded<N> {
        fn read_from<R: Read + ?Sized>(reader: &mut R) -> io::Result<Self> {
            let mut buf = vec![0u8; N];
            reader.read_exact(&mut buf)?;
            core_io::read_from_slice::<Self>(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
        }
    }

    #[cfg(all(feature = "text_fixed", feature = "text_utf8"))]
    impl<const N: usize> EndianWrite for crate::FixedUtf8SpacePadded<N> {
        fn write_to<W: Write + ?Sized>(&self, writer: &mut W) -> io::Result<()> {
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
    use crate::{BigEndian, LittleEndian, SpecificEndian};
    use std::io::Cursor;

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

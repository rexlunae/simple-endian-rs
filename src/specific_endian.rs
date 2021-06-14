/// Any object implementing `SpecificEndian<T>` can be converted between big and little endian.  Implement this trait to allow for endian conversion by this crate.
pub trait SpecificEndian<T>
where
    Self: Into<T> + Clone + Copy,
{
    fn to_big_endian(&self) -> T;
    fn to_little_endian(&self) -> T;
    fn from_big_endian(&self) -> T;
    fn from_little_endian(&self) -> T;
}

#[cfg(feature = "byte_impls")]
mod byte_impls {
    use super::*;
    /// A macro implementing `SpecificEndian<T>` for simple data types where big and little endian forms are the same.
    macro_rules! make_specific_endian_single_byte {
        ($wrap_ty:ty) => {
            impl SpecificEndian<$wrap_ty> for $wrap_ty {
                fn to_big_endian(&self) -> Self {
                    *self
                }
                fn to_little_endian(&self) -> Self {
                    *self
                }
                fn from_big_endian(&self) -> Self {
                    *self
                }
                fn from_little_endian(&self) -> Self {
                    *self
                }
            }
        };
    }

    make_specific_endian_single_byte!(u8);
    make_specific_endian_single_byte!(i8);
    // If bool ends up being represented by something other than a byte, this might not work right.
    make_specific_endian_single_byte!(bool);
}

#[cfg(feature = "integer_impls")]
mod integer_impls {
    use super::*;
    /// A macro for implementing `SpecificEndian<T>` on types that have endian conversions built into Rust.  Currently, this is the primitive integer types.
    macro_rules! make_specific_endian_integer {
        ($wrap_ty:ty) => {
            impl SpecificEndian<$wrap_ty> for $wrap_ty {
                fn to_big_endian(&self) -> Self {
                    self.to_be()
                }
                fn to_little_endian(&self) -> Self {
                    self.to_le()
                }
                fn from_big_endian(&self) -> Self {
                    Self::from_be(*self)
                }
                fn from_little_endian(&self) -> Self {
                    Self::from_le(*self)
                }
            }
        };
    }

    make_specific_endian_integer!(u16);
    make_specific_endian_integer!(i16);
    make_specific_endian_integer!(u32);
    make_specific_endian_integer!(i32);
    make_specific_endian_integer!(u64);
    make_specific_endian_integer!(i64);
    make_specific_endian_integer!(u128);
    make_specific_endian_integer!(i128);
    make_specific_endian_integer!(usize);
    make_specific_endian_integer!(isize);
}

#[cfg(feature = "float_impls")]
mod float_impls {
    use super::*;
    /// Uses .from_bits() and .to_bits() to implement SpecificEndian<T> with Integer types.  Can be used with any type having these methods, but mainly for use with the floats.
    macro_rules! make_specific_endian_float {
        ($wrap_ty:ty) => {
            impl SpecificEndian<$wrap_ty> for $wrap_ty {
                fn to_big_endian(&self) -> Self {
                    Self::from_bits(self.to_bits().to_be())
                }
                fn to_little_endian(&self) -> Self {
                    Self::from_bits(self.to_bits().to_le())
                }
                fn from_big_endian(&self) -> Self {
                    Self::from_bits(self.to_bits().from_big_endian())
                }
                fn from_little_endian(&self) -> Self {
                    Self::from_bits(self.to_bits().from_little_endian())
                }
            }
        };
    }

    make_specific_endian_float!(f32);
    make_specific_endian_float!(f64);
}

/// A big-endian representation of type `T` that implements `SpecificEndian<T>`.  Data stored in the struct must be converted to big-endian using `::from()` or `.into()`.
#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct BigEndian<T: SpecificEndian<T>> {
    pub(crate) _v: T,
}
unsafe impl<T: Send + SpecificEndian<T>> Send for BigEndian<T> {}
unsafe impl<T: Sync + SpecificEndian<T>> Sync for BigEndian<T> {}

impl<T> BigEndian<T>
where
    T: SpecificEndian<T>,
{
    /// Returns the raw data stored in the struct.
    pub fn to_bits(&self) -> T {
        self._v
    }
    /// Imports the data raw into a BigEndian<T> struct.
    pub fn from_bits(v: T) -> Self {
        Self { _v: v }
    }
    /// Converts the data to the same type T in host-native endian.
    pub fn to_native(&self) -> T {
        T::from_big_endian(&self._v)
    }
}

impl<T: SpecificEndian<T>> From<T> for BigEndian<T> {
    fn from(v: T) -> BigEndian<T> {
        BigEndian::<T> {
            _v: v.to_big_endian(),
        }
    }
}

/// A little-endian representation of type `T` that implements `SpecificEndian<T>`.  Data stored in the struct must be converted to little-endian using `::from()` or `.into()`.
#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct LittleEndian<T: SpecificEndian<T>> {
    pub(crate) _v: T,
}
unsafe impl<T: Send + SpecificEndian<T>> Send for LittleEndian<T> {}
unsafe impl<T: Sync + SpecificEndian<T>> Sync for LittleEndian<T> {}

impl<T> LittleEndian<T>
where
    T: SpecificEndian<T>,
{
    /// Returns the raw data stored in the struct.
    pub fn to_bits(&self) -> T {
        self._v
    }
    /// Imports the data raw into a LittleEndian<T> struct.
    pub fn from_bits(v: T) -> Self {
        Self { _v: v }
    }
    /// Converts the data to the same type T in host-native endian.
    pub fn to_native(&self) -> T {
        T::from_little_endian(&self._v)
    }
}

impl<T: SpecificEndian<T>> From<T> for LittleEndian<T> {
    fn from(v: T) -> LittleEndian<T> {
        LittleEndian::<T> {
            _v: v.to_little_endian(),
        }
    }
}

#[cfg(feature = "big_endian")]
mod big_endian_primatives {
    #[allow(unused_imports)]
    use super::*;
    // Rust's orphan trait rule prevents us from using a generic implementation on the primitive types, so we do this:
    #[allow(unused_macros)]
    macro_rules! make_primitive_type_from_be {
        ($wrap_ty:ty) => {
            impl From<BigEndian<$wrap_ty>> for $wrap_ty {
                fn from(v: BigEndian<$wrap_ty>) -> $wrap_ty {
                    v._v.from_big_endian()
                }
            }
        };
    }

    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(bool);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(u8);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(i8);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(u16);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(i16);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(u32);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(i32);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(u64);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(i64);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(u128);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(i128);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(usize);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_be!(isize);
    #[cfg(feature = "float_impls")]
    make_primitive_type_from_be!(f32);
    #[cfg(feature = "float_impls")]
    make_primitive_type_from_be!(f64);
}

#[cfg(feature = "little_endian")]
mod little_endian_primatives {
    #[allow(unused_imports)]
    use super::*;
    // Rust's orphan trait rule prevents us from using a generic implementation on the primitive types, so we do this:
    #[allow(unused_macros)]
    macro_rules! make_primitive_type_from_le {
        ($wrap_ty:ty) => {
            impl From<LittleEndian<$wrap_ty>> for $wrap_ty {
                fn from(v: LittleEndian<$wrap_ty>) -> $wrap_ty {
                    v._v.from_little_endian()
                }
            }
        };
    }

    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(bool);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(u8);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(i8);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(u16);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(i16);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(u32);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(i32);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(u64);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(i64);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(u128);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(i128);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(usize);
    #[cfg(feature = "integer_impls")]
    make_primitive_type_from_le!(isize);
    #[cfg(feature = "float_impls")]
    make_primitive_type_from_le!(f32);
    #[cfg(feature = "float_impls")]
    make_primitive_type_from_le!(f64);
}

#[cfg(feature = "both_endian")]
mod both_endian_primatives {
    use super::*;
    /// Allow conversion directly from `LittleEndian<T>` to `BigEndian<T>` without manually going through native endian.
    impl<T: SpecificEndian<T>> From<LittleEndian<T>> for BigEndian<T> {
        fn from(v: LittleEndian<T>) -> BigEndian<T> {
            BigEndian::<T>::from(v.to_native())
        }
    }

    /// Allow conversion directly from `BigEndian<T>` to `LittleEndian<T>` without manually going through native endian.
    impl<T: SpecificEndian<T>> From<BigEndian<T>> for LittleEndian<T> {
        fn from(v: BigEndian<T>) -> LittleEndian<T> {
            LittleEndian::<T>::from(v.to_native())
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use crate::*;
    use core::mem::size_of;

    #[test]
    fn declare_all() {
        let _a: BigEndian<i16> = 0xfe.into();
        let _a: LittleEndian<i16> = 0xfe.into();
        let _a: BigEndian<u16> = 0xfe.into();
        let _a: LittleEndian<u16> = 0xfe.into();

        let _a: BigEndian<i32> = 0xfe.into();
        let _a: LittleEndian<i32> = 0xfe.into();
        let _a: BigEndian<u32> = 0xfe.into();
        let _a: LittleEndian<u32> = 0xfe.into();

        let _a: BigEndian<i64> = 0xfe.into();
        let _a: LittleEndian<i64> = 0xfe.into();
        let _a: BigEndian<u64> = 0xfe.into();
        let _a: LittleEndian<u64> = 0xfe.into();

        let _a: BigEndian<i128> = 0xfe.into();
        let _a: LittleEndian<i128> = 0xfe.into();
        let _a: BigEndian<u128> = 0xfe.into();
        let _a: LittleEndian<u128> = 0xfe.into();
    }

    #[test]
    fn make_struct() {
        #[repr(C)]
        struct Foo(
            BigEndian<i16>,
            LittleEndian<i16>,
            BigEndian<u16>,
            LittleEndian<u16>,
            BigEndian<i32>,
            LittleEndian<i32>,
            BigEndian<u32>,
            LittleEndian<u32>,
            BigEndian<i64>,
            LittleEndian<i64>,
            BigEndian<u64>,
            LittleEndian<u64>,
            BigEndian<i128>,
            LittleEndian<i128>,
            BigEndian<u128>,
            LittleEndian<u128>,
            BigEndian<f32>,
            LittleEndian<f32>,
            BigEndian<f64>,
            LittleEndian<f64>,
        );

        let _foo = Foo(
            0.into(),
            1.into(),
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            7.into(),
            8.into(),
            9.into(),
            10.into(),
            11.into(),
            12.into(),
            13.into(),
            14.into(),
            15.into(),
            (0.1).into(),
            (123.5).into(),
            (7.8).into(),
            (12345.4567).into(),
        );
    }

    #[test]
    fn store_be() {
        let be: BigEndian<u64> = 0xfe.into();
        if cfg!(byte_order = "big endian") {
            assert_eq!(be.to_bits(), 0xfe);
        } else {
            assert_eq!(be.to_bits(), 0xfe00000000000000);
        }
    }

    #[test]
    fn same_size() {
        assert_eq!(size_of::<u64be>(), size_of::<u64>());
    }

    #[test]
    fn store_le() {
        let le: LittleEndian<u64> = 0xfe.into();
        if cfg!(byte_order = "big endian") {
            assert_eq!(le.to_bits(), 0xfe00000000000000);
        } else {
            assert_eq!(le.to_bits(), 0xfe);
        }
    }

    #[test]
    fn cast() {
        let be = BigEndian::from(12345);
        let ne: u64 = be.into();
        assert_eq!(ne, 12345);
    }

    #[test]
    fn convert_back() {
        let be = BigEndian::from(12345);
        assert_eq!(12345, u64::from(be));
    }

    #[test]
    fn convert_to_native() {
        let be = BigEndian::from(0xfe);
        assert_eq!(0xfe, be.to_native());
    }

    #[test]
    fn store_fp_be() {
        let be1 = BigEndian::<f64>::from(1234.5678);
        if cfg!(byte_order = "little endian") {
            assert_ne!(1234.5678, be1.to_bits());
        }
        assert_eq!(1234.5678, f64::from(be1));
    }

    #[test]
    fn store_fp_le() {
        let le1 = LittleEndian::<f64>::from(1234.5678);
        if cfg!(byte_order = "big endian") {
            assert_ne!(1234.5678, le1.to_bits());
        }
        assert_eq!(1234.5678, f64::from(le1));
    }

    #[test]
    fn inferred_type() {
        let mut be1 = BigEndian::from(1234);
        be1 &= BigEndian::from(5678);
        assert_eq!(be1, 1026.into());
    }

    #[test]
    fn inferred_type_fp() {
        let mut be1 = BigEndian::from(1234.5);
        be1 += BigEndian::from(5678.1);
        assert_eq!(be1, 6912.6.into());
    }

    #[test]
    fn inferred_type_bigger() {
        let mut be1 = BigEndian::from(0x0feeddcc);
        be1 &= BigEndian::from(0xff00);
        assert_eq!(be1, 0xdd00.into());
    }

    #[test]
    fn mixed_endian_big() {
        let be = BigEndian::from(100);
        let le = LittleEndian::from(200);
        let me = be + le.into();
        assert_eq!(me, 300.into());
    }

    #[test]
    fn mixed_endian_little() {
        let be = BigEndian::from(100);
        let le = LittleEndian::from(200);
        let me = le + be.into();
        assert_eq!(me, 300.into());
    }

    #[test]
    fn custom_type() {
        #[derive(Copy, Clone, Debug)]
        enum EndianAwareExample {
            BigEndianFunction(u64),
            LittleEndianFunction(u64),
        }
        impl SpecificEndian<EndianAwareExample> for EndianAwareExample {
            fn to_big_endian(&self) -> Self {
                match self {
                    EndianAwareExample::BigEndianFunction(_v) => *self,
                    EndianAwareExample::LittleEndianFunction(v) => {
                        EndianAwareExample::BigEndianFunction(v.to_big_endian())
                    }
                }
            }
            fn to_little_endian(&self) -> Self {
                match self {
                    EndianAwareExample::LittleEndianFunction(_v) => *self,
                    EndianAwareExample::BigEndianFunction(v) => {
                        EndianAwareExample::BigEndianFunction(v.to_little_endian())
                    }
                }
            }
            fn from_big_endian(&self) -> Self {
                match self {
                    EndianAwareExample::BigEndianFunction(_v) => *self,
                    EndianAwareExample::LittleEndianFunction(v) => {
                        EndianAwareExample::BigEndianFunction(v.to_big_endian())
                    }
                }
            }
            fn from_little_endian(&self) -> Self {
                match self {
                    EndianAwareExample::LittleEndianFunction(_v) => *self,
                    EndianAwareExample::BigEndianFunction(v) => {
                        EndianAwareExample::BigEndianFunction(v.to_little_endian())
                    }
                }
            }
        }
        let foo: BigEndian<EndianAwareExample> =
            EndianAwareExample::LittleEndianFunction(0xf0).into();
        #[allow(unused_assignments)]
        let value = match foo.to_native() {
            EndianAwareExample::BigEndianFunction(v) => v,
            // TODO this doesn't seem right? It'll cause the assert to always fail.
            EndianAwareExample::LittleEndianFunction(_v) => 0,
        };
        assert_eq!(value, 0x0f000000000000000);
    }
}

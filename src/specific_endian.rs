/// Any object implementing `SpecificEndian<T>` can be converted between big and little endian.  Implement this trait to allow for endian conversion by this crate.
#[allow(clippy::wrong_self_convention)]
pub enum Endian {
    Big,
    Little,
}

pub trait SpecificEndian<T>
where
    Self: Into<T> + Clone + Copy,
{
    fn to_big_endian(&self) -> T;
    fn to_little_endian(&self) -> T;
    fn from_big_endian(&self) -> T;
    fn from_little_endian(&self) -> T;

    fn endian(&self) -> Endian {
        // Default to the host target endianness. This is simpler and avoids
        // requiring additional trait bounds in the default implementation.
        if cfg!(target_endian = "big") {
            Endian::Big
        } else {
            Endian::Little
        }
    }
}

/// Endian conversion trait for **owned / non-Copy** types.
///
/// The existing [`SpecificEndian`] trait requires `Copy` because many of this crate's
/// wrapper APIs (like `BigEndian<T>::to_bits()` and `to_native()`) return values by copy.
///
/// Text and buffer types (e.g. `Vec<_>`, `String`) are not `Copy`, but we still want to
/// provide endian-aware conversions for them. This trait mirrors the API of
/// [`SpecificEndian`] without requiring `Copy`.
///
/// This is intentionally a separate trait to avoid a breaking change to the existing
/// `SpecificEndian` ecosystem.
#[allow(clippy::wrong_self_convention)]
pub trait SpecificEndianOwned
where
    Self: Clone,
{
    /// The big-endian form of this type.
    type Big;
    /// The little-endian form of this type.
    type Little;

    fn to_big_endian(&self) -> Self::Big;
    fn to_little_endian(&self) -> Self::Little;
    fn from_big_endian(&self) -> Self::Big;
    fn from_little_endian(&self) -> Self::Little;

    fn endian(&self) -> Endian {
        if cfg!(target_endian = "big") {
            Endian::Big
        } else {
            Endian::Little
        }
    }
}

// When `simple_specific_endian_bridge` is enabled we provide no-op SpecificEndian impls for
// the SimpleEndian + Copy primitives (bool/u8/i8). Those would overlap with the `byte_impls`
// impls below, so we disable this module in that configuration.
#[cfg(all(feature = "byte_impls", not(feature = "simple_specific_endian_bridge")))]
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

/// Bridge `SimpleEndian` -> `SpecificEndian` for primitive, endianness-invariant types.
///
/// We intentionally implement this only for the `SimpleEndian + Copy` primitives we expose
/// today (bool/u8/i8), because a true blanket impl like
/// `impl<T: SimpleEndian + Copy> SpecificEndian<T> for T` would overlap with other `SpecificEndian`
/// impls (e.g. integers/floats) and is rejected by Rust.
#[cfg(feature = "simple_specific_endian_bridge")]
mod simple_specific_endian_bridge {
    use super::*;
    use crate::SimpleEndian;

    // bool uses the same gating as the SimpleEndian impl.
    #[cfg(feature = "simple_bool")]
    impl SpecificEndian<bool> for bool {
        fn to_big_endian(&self) -> bool {
            (*self).to_big_endian()
        }
        fn to_little_endian(&self) -> bool {
            (*self).to_little_endian()
        }
        fn from_big_endian(&self) -> bool {
            (*self).from_big_endian()
        }
        fn from_little_endian(&self) -> bool {
            (*self).from_little_endian()
        }
    }

    // u8/i8 use the same gating as the SimpleEndian impl.
    #[cfg(feature = "simple_byte_impls")]
    impl SpecificEndian<u8> for u8 {
        fn to_big_endian(&self) -> u8 {
            (*self).to_big_endian()
        }
        fn to_little_endian(&self) -> u8 {
            (*self).to_little_endian()
        }
        fn from_big_endian(&self) -> u8 {
            (*self).from_big_endian()
        }
        fn from_little_endian(&self) -> u8 {
            (*self).from_little_endian()
        }
    }

    #[cfg(feature = "simple_byte_impls")]
    impl SpecificEndian<i8> for i8 {
        fn to_big_endian(&self) -> i8 {
            (*self).to_big_endian()
        }
        fn to_little_endian(&self) -> i8 {
            (*self).to_little_endian()
        }
        fn from_big_endian(&self) -> i8 {
            (*self).from_big_endian()
        }
        fn from_little_endian(&self) -> i8 {
            (*self).from_little_endian()
        }
    }

    #[cfg(feature = "simple_char_impls")]
    impl SpecificEndian<char> for char {
        fn to_big_endian(&self) -> char {
            (*self).to_big_endian()
        }
        fn to_little_endian(&self) -> char {
            (*self).to_little_endian()
        }
        fn from_big_endian(&self) -> char {
            (*self).from_big_endian()
        }
        fn from_little_endian(&self) -> char {
            (*self).from_little_endian()
        }
    }
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

#[cfg(all(feature = "integer_impls", feature = "nonzero"))]
mod nonzero_impls {
    use super::*;
    use core::num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize,
        NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    };

    /// Implement `SpecificEndian` for `core::num::NonZero*` integers by swapping the underlying integer.
    ///
    /// This is always infallible: byte swapping preserves non-zeroness.
    macro_rules! make_specific_endian_nonzero {
        ($nz:ty, $int:ty) => {
            impl SpecificEndian<$nz> for $nz {
                fn to_big_endian(&self) -> Self {
                    // SAFETY: `get()` is non-zero and byte swapping preserves non-zero.
                    unsafe { <$nz>::new_unchecked(self.get().to_be()) }
                }
                fn to_little_endian(&self) -> Self {
                    unsafe { <$nz>::new_unchecked(self.get().to_le()) }
                }
                fn from_big_endian(&self) -> Self {
                    unsafe { <$nz>::new_unchecked(<$int>::from_be(self.get())) }
                }
                fn from_little_endian(&self) -> Self {
                    unsafe { <$nz>::new_unchecked(<$int>::from_le(self.get())) }
                }
            }
        };
    }

    make_specific_endian_nonzero!(NonZeroU8, u8);
    make_specific_endian_nonzero!(NonZeroI8, i8);
    make_specific_endian_nonzero!(NonZeroU16, u16);
    make_specific_endian_nonzero!(NonZeroI16, i16);
    make_specific_endian_nonzero!(NonZeroU32, u32);
    make_specific_endian_nonzero!(NonZeroI32, i32);
    make_specific_endian_nonzero!(NonZeroU64, u64);
    make_specific_endian_nonzero!(NonZeroI64, i64);
    make_specific_endian_nonzero!(NonZeroU128, u128);
    make_specific_endian_nonzero!(NonZeroI128, i128);
    make_specific_endian_nonzero!(NonZeroUsize, usize);
    make_specific_endian_nonzero!(NonZeroIsize, isize);
}

#[cfg(all(feature = "integer_impls", feature = "wrapping"))]
mod wrapping_impls {
    use super::*;
    use core::num::Wrapping;

    impl<T> SpecificEndian<Wrapping<T>> for Wrapping<T>
    where
        T: SpecificEndian<T>,
    {
        fn to_big_endian(&self) -> Self {
            Wrapping(self.0.to_big_endian())
        }
        fn to_little_endian(&self) -> Self {
            Wrapping(self.0.to_little_endian())
        }
        fn from_big_endian(&self) -> Self {
            Wrapping(self.0.from_big_endian())
        }
        fn from_little_endian(&self) -> Self {
            Wrapping(self.0.from_little_endian())
        }
    }
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
pub struct BigEndian<T: SpecificEndian<T>>(pub(crate) T);

impl<T> BigEndian<T>
where
    T: SpecificEndian<T>,
{
    /// Returns the raw data stored in the struct.
    pub fn to_bits(&self) -> T {
        self.0
    }
    /// Imports the data raw into a BigEndian<T> struct.
    pub fn from_bits(v: T) -> Self {
        Self(v)
    }
    /// Converts the data to the same type T in host-native endian.
    pub fn to_native(&self) -> T {
        T::from_big_endian(&self.0)
    }
}

impl<T: SpecificEndian<T>> From<T> for BigEndian<T> {
    fn from(v: T) -> BigEndian<T> {
        BigEndian::<T>(v.to_big_endian())
    }
}

/// A little-endian representation of type `T` that implements `SpecificEndian<T>`.  Data stored in the struct must be converted to little-endian using `::from()` or `.into()`.
#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct LittleEndian<T: SpecificEndian<T>>(pub(crate) T);

impl<T> LittleEndian<T>
where
    T: SpecificEndian<T>,
{
    /// Returns the raw data stored in the struct.
    pub fn to_bits(&self) -> T {
        self.0
    }
    /// Imports the data raw into a LittleEndian<T> struct.
    pub fn from_bits(v: T) -> Self {
        Self(v)
    }
    /// Converts the data to the same type T in host-native endian.
    pub fn to_native(&self) -> T {
        T::from_little_endian(&self.0)
    }
}

impl<T: SpecificEndian<T>> From<T> for LittleEndian<T> {
    fn from(v: T) -> LittleEndian<T> {
        LittleEndian::<T>(v.to_little_endian())
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
                    v.0.from_big_endian()
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
                    v.0.from_little_endian()
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
    use crate::*;
    use core::mem::size_of;

    // These tests are specifically for the `SimpleEndian` -> `SpecificEndian` bridge.
    // They ensure we get the `SpecificEndian` methods on the expected primitives and that
    // the conversions are no-ops.
    #[cfg(all(feature = "simple_specific_endian_bridge", feature = "simple_bool"))]
    #[test]
    fn bridge_bool_is_noop() {
        fn assert_specific_endian<T: SpecificEndian<T>>() {}
        assert_specific_endian::<bool>();

        let v = true;
        assert_eq!(SpecificEndian::to_big_endian(&v), v);
        assert_eq!(SpecificEndian::to_little_endian(&v), v);
        assert_eq!(SpecificEndian::from_big_endian(&v), v);
        assert_eq!(SpecificEndian::from_little_endian(&v), v);
    }

    #[cfg(all(feature = "simple_specific_endian_bridge", feature = "simple_byte_impls"))]
    #[test]
    fn bridge_u8_is_noop() {
        fn assert_specific_endian<T: SpecificEndian<T>>() {}
        assert_specific_endian::<u8>();

        let v: u8 = 0xfe;
        assert_eq!(SpecificEndian::to_big_endian(&v), v);
        assert_eq!(SpecificEndian::to_little_endian(&v), v);
        assert_eq!(SpecificEndian::from_big_endian(&v), v);
        assert_eq!(SpecificEndian::from_little_endian(&v), v);
    }

    #[cfg(all(feature = "simple_specific_endian_bridge", feature = "simple_byte_impls"))]
    #[test]
    fn bridge_i8_is_noop() {
        fn assert_specific_endian<T: SpecificEndian<T>>() {}
        assert_specific_endian::<i8>();

        let v: i8 = -42;
        assert_eq!(SpecificEndian::to_big_endian(&v), v);
        assert_eq!(SpecificEndian::to_little_endian(&v), v);
        assert_eq!(SpecificEndian::from_big_endian(&v), v);
        assert_eq!(SpecificEndian::from_little_endian(&v), v);
    }

    #[cfg(all(feature = "simple_specific_endian_bridge", feature = "simple_char_impls"))]
    #[test]
    fn bridge_char_is_noop() {
        fn assert_specific_endian<T: SpecificEndian<T>>() {}
        assert_specific_endian::<char>();

        let v: char = '🦀';
        assert_eq!(SpecificEndian::to_big_endian(&v), v);
        assert_eq!(SpecificEndian::to_little_endian(&v), v);
        assert_eq!(SpecificEndian::from_big_endian(&v), v);
        assert_eq!(SpecificEndian::from_little_endian(&v), v);
    }

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
        if cfg!(target_endian = "big") {
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
        if cfg!(target_endian = "big") {
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
        println!("{}", u64::from(be));
    }

    #[test]
    fn convert_to_native() {
        let be = BigEndian::from(0xfe);
        println!("{:x}, {:x}", be.0, be.to_native());
        assert_eq!(0xfe, be.to_native());
    }

    #[test]
    fn store_fp_be() {
        let be1 = BigEndian::<f64>::from(1234.5678);
        if cfg!(target_endian = "little") {
            assert_ne!(1234.5678, be1.to_bits());
        }
        assert_eq!(1234.5678, f64::from(be1));
    }

    #[test]
    fn store_fp_le() {
        let le1 = LittleEndian::<f64>::from(1234.5678);
        if cfg!(target_endian = "big") {
            assert_ne!(1234.5678, le1.to_bits());
        }
        assert_eq!(1234.5678, f64::from(le1));
    }

    #[test]
    fn inferred_type() {
        let mut be1 = BigEndian::from(1234);
        be1 &= BigEndian::from(5678);
        println!("{} {} {}", be1, be1.to_bits(), be1.to_native());
        assert_eq!(be1, 1026.into());
    }

    #[test]
    fn inferred_type_fp() {
        let mut be1 = BigEndian::from(1234.5);
        be1 += BigEndian::from(5678.1);
        println!("{} {} {}", be1, be1.to_bits(), be1.to_native());
        assert_eq!(be1, 6912.6.into());
    }

    #[test]
    fn inferred_type_bigger() {
        let mut be1 = BigEndian::from(0x0feeddcc);
        be1 &= BigEndian::from(0xff00);
        println!("{} {} {}", be1, be1.to_bits(), be1.to_native());
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
                    EndianAwareExample::BigEndianFunction(_) => *self,
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
                    EndianAwareExample::LittleEndianFunction(_) => *self,
                    EndianAwareExample::BigEndianFunction(v) => {
                        EndianAwareExample::BigEndianFunction(v.to_little_endian())
                    }
                }
            }
        }
        let foo: BigEndian<EndianAwareExample> =
            EndianAwareExample::LittleEndianFunction(0xf0).into();
        #[allow(unused_assignments)]
        let mut value = 0;
        match foo.to_native() {
            EndianAwareExample::BigEndianFunction(v) => {
                println!("be: {:x}", v);
                value = v
            }
            EndianAwareExample::LittleEndianFunction(v) => {
                println!("le: {:x}", v);
                value = 0
            }
        }
        assert_eq!(value, 0x0f000000000000000);
    }
}

/// Many byte-order-handling libraries focus on providing code to convert to and from big- or little-endian.  However,
/// this requires users of those libraries to use a lot of explicit logic.  This library uses the Rust type system to
/// enforce conversions invisibly, and also ensure that they are done consistently.  A struct member can be read and written
/// simply using the standard From and Into trait methods (from() and into()).  No explicit endian checks are required.
///  
/// # Example 1:
/// 
///```rust
/// use simple_endian::*;
///
/// fn init() {
///     struct BinPacket {
///         a: u64be,
///         b: u32be,
///     }
///     let mut bp = BinPacket{a: 0xfe.into(), b: 10.into()};
///     let new_a = bp.a.to_native() * 1234; 
 
///     bp.a = new_a.into();
///     bp.b = 1234.into();
/// }
/// ```
/// 
/// Trying to write `bp.a = new_a;` causes an error because the type u64 can't be directly stored.
/// 

use std::{
    cmp::Ordering,
    ops::{BitAnd, Not, Add, AddAssign, Div, DivAssign, Mul, MulAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign, BitAndAssign, BitXor, BitXorAssign, BitOr, BitOrAssign},
    fmt::{Formatter, Result, UpperHex, LowerHex, Octal, Binary, Display},
};

/// A trait that allows endian conversions.  Any type that wants to use this crate to do endian conversions must implement this trait.
pub trait SpecificEndian<T> where Self: Into<T> + Clone + Copy {
    fn to_big_endian(&self) -> T;
    fn to_little_endian(&self) -> T;
    fn from_big_endian(&self) -> T;
    fn from_little_endian(&self) -> T;

}

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
    }
}

make_specific_endian_single_byte!(u8);
make_specific_endian_single_byte!(i8);
// If bool ends up being represented by something other than a byte, this might not work right.
make_specific_endian_single_byte!(bool);

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
    }
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

// Rust doesn't have built-in byte-swapping for floating-point types,
// so we use integer logic.
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
    }
}

make_specific_endian_float!(f32);
make_specific_endian_float!(f64);

/// A big-endian struct that basically just uses the type system to tag the endianness.
#[derive(Copy, Clone, Hash, Debug)]
pub struct BigEndian<T: SpecificEndian<T>> (T);
unsafe impl<T: Send + SpecificEndian<T>> Send for BigEndian<T> {}
unsafe impl<T: Sync + SpecificEndian<T>> Sync for BigEndian<T> {}


impl<T> BigEndian<T> where T: SpecificEndian<T> {
    pub fn raw(&self) -> T {
        self.0
    }
    pub fn to_native(&self) -> T {
        T::from_big_endian(&self.0)
    }
}

impl<T: SpecificEndian<T>> From<T> for BigEndian<T> {
    fn from(v: T) -> BigEndian<T> {
        BigEndian::<T>(v.to_big_endian())
    }
}

macro_rules! add_equality_ops {
    ($wrap_ty:ty) => {
        impl PartialEq for $wrap_ty {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }
        impl Eq for $wrap_ty {}
        impl PartialOrd for $wrap_ty {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.to_native().partial_cmp(&other.to_native())
            }
        }
    
    }
}

add_equality_ops!(BigEndian<bool>);
add_equality_ops!(BigEndian<u8>);
add_equality_ops!(BigEndian<i8>);
add_equality_ops!(BigEndian<u16>);
add_equality_ops!(BigEndian<i16>);
add_equality_ops!(BigEndian<u32>);
add_equality_ops!(BigEndian<i32>);
add_equality_ops!(BigEndian<u64>);
add_equality_ops!(BigEndian<i64>);
add_equality_ops!(BigEndian<u128>);
add_equality_ops!(BigEndian<i128>);
add_equality_ops!(BigEndian<usize>);
add_equality_ops!(BigEndian<isize>);
add_equality_ops!(BigEndian<f32>);
add_equality_ops!(BigEndian<f64>);

add_equality_ops!(LittleEndian<bool>);
add_equality_ops!(LittleEndian<u8>);
add_equality_ops!(LittleEndian<i8>);
add_equality_ops!(LittleEndian<u16>);
add_equality_ops!(LittleEndian<i16>);
add_equality_ops!(LittleEndian<u32>);
add_equality_ops!(LittleEndian<i32>);
add_equality_ops!(LittleEndian<u64>);
add_equality_ops!(LittleEndian<i64>);
add_equality_ops!(LittleEndian<u128>);
add_equality_ops!(LittleEndian<i128>);
add_equality_ops!(LittleEndian<usize>);
add_equality_ops!(LittleEndian<isize>);
add_equality_ops!(LittleEndian<f32>);
add_equality_ops!(LittleEndian<f64>);


/// Implement the bitwise operations on the types.  These should be as fast in either endian, because they are endian-agnostic.
macro_rules! add_bitwise_ops {
    ($wrap_ty:ty) => {
        impl Ord for $wrap_ty {
            fn cmp(&self, other: &Self) -> Ordering {
                self.to_native().cmp(&other.to_native())
            }
        }
        impl BitAnd for $wrap_ty {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }
        impl BitAndAssign for $wrap_ty {
           fn bitand_assign(&mut self, rhs: Self) {
                *self = *self & rhs
            }
        }
        impl BitXor for $wrap_ty {
            // We don't need to convert endian for this op.
            type Output = Self;

            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.0 ^ rhs.0)
            }
        }
        impl BitXorAssign for $wrap_ty {
            fn bitxor_assign(&mut self, rhs: Self) {
                *self = *self ^ rhs
            }
        }
        impl BitOr for $wrap_ty {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }
        impl BitOrAssign for $wrap_ty {
            fn bitor_assign(&mut self, rhs: Self) {
                *self = *self | rhs;
            }
        }
        impl Not for $wrap_ty {
            type Output = Self;

            fn not(self) -> Self::Output {
                Self(!self.0)
            }
        }        
    }
}

add_bitwise_ops!(BigEndian<bool>);
add_bitwise_ops!(BigEndian<u8>);
add_bitwise_ops!(BigEndian<i8>);
add_bitwise_ops!(BigEndian<u16>);
add_bitwise_ops!(BigEndian<i16>);
add_bitwise_ops!(BigEndian<u32>);
add_bitwise_ops!(BigEndian<i32>);
add_bitwise_ops!(BigEndian<u64>);
add_bitwise_ops!(BigEndian<i64>);
add_bitwise_ops!(BigEndian<u128>);
add_bitwise_ops!(BigEndian<i128>);
add_bitwise_ops!(BigEndian<usize>);
add_bitwise_ops!(BigEndian<isize>);

add_bitwise_ops!(LittleEndian<bool>);
add_bitwise_ops!(LittleEndian<u8>);
add_bitwise_ops!(LittleEndian<i8>);
add_bitwise_ops!(LittleEndian<u16>);
add_bitwise_ops!(LittleEndian<i16>);
add_bitwise_ops!(LittleEndian<u32>);
add_bitwise_ops!(LittleEndian<i32>);
add_bitwise_ops!(LittleEndian<u64>);
add_bitwise_ops!(LittleEndian<i64>);
add_bitwise_ops!(LittleEndian<u128>);
add_bitwise_ops!(LittleEndian<i128>);
add_bitwise_ops!(LittleEndian<usize>);
add_bitwise_ops!(LittleEndian<isize>);

macro_rules! add_shift_ops {
    ($wrap_ty:ty) => {
        impl Shl for $wrap_ty {
            type Output = Self;
        
            fn shl(self, other: Self) -> Self {
                Self::from(self.to_native() << other.to_native())
            }
        }
        impl ShlAssign for $wrap_ty {
            fn shl_assign(&mut self, rhs: Self) {
                *self = Self::from((*self).to_native() << rhs.to_native());
            }
        }
        impl Shr for $wrap_ty {
            type Output = Self;
        
            fn shr(self, other: Self) -> Self {
                Self::from(self.to_native() >> other.to_native())
            }
        }
        impl ShrAssign for $wrap_ty {
            fn shr_assign(&mut self, rhs: Self) {
                *self = Self::from((*self).to_native() >> rhs.to_native());
            }
        }
    }
}

add_shift_ops!(BigEndian<u8>);
add_shift_ops!(BigEndian<i8>);
add_shift_ops!(BigEndian<u16>);
add_shift_ops!(BigEndian<i16>);
add_shift_ops!(BigEndian<u32>);
add_shift_ops!(BigEndian<i32>);
add_shift_ops!(BigEndian<u64>);
add_shift_ops!(BigEndian<i64>);
add_shift_ops!(BigEndian<u128>);
add_shift_ops!(BigEndian<i128>);
add_shift_ops!(BigEndian<usize>);
add_shift_ops!(BigEndian<isize>);

add_shift_ops!(LittleEndian<u8>);
add_shift_ops!(LittleEndian<i8>);
add_shift_ops!(LittleEndian<u16>);
add_shift_ops!(LittleEndian<i16>);
add_shift_ops!(LittleEndian<u32>);
add_shift_ops!(LittleEndian<i32>);
add_shift_ops!(LittleEndian<u64>);
add_shift_ops!(LittleEndian<i64>);
add_shift_ops!(LittleEndian<u128>);
add_shift_ops!(LittleEndian<i128>);
add_shift_ops!(LittleEndian<usize>);
add_shift_ops!(LittleEndian<isize>);


macro_rules! add_math_ops {
    ($wrap_ty:ty) => {
        impl Add for $wrap_ty {
            type Output = Self;

            fn add(self, other: Self) -> Self {
                Self::from(self.to_native() + other.to_native())
            }
        }

        impl AddAssign for $wrap_ty {
            fn add_assign(&mut self, other: Self) {
                *self = *self + other;
            }
        }

        impl Mul for $wrap_ty {
            type Output = Self;

            fn mul(self, other: Self) -> Self {
                Self::from(self.to_native() * other.to_native())
            }
        }

        impl MulAssign for $wrap_ty {
            fn mul_assign(&mut self, other: Self) {
                *self = *self * other;
            }
        }

        impl Div for $wrap_ty {
            type Output = Self;

            fn div(self, other: Self) -> Self {
                Self::from(self.to_native() / other.to_native())
            }
        }

        impl DivAssign for $wrap_ty {
            fn div_assign(&mut self, other: Self) {
                *self = *self / other;
            }
        }

        impl Sub for $wrap_ty {
            type Output = Self;

            fn sub(self, other: Self) -> Self {
                Self::from(self.to_native() - other.to_native())
            }
        }
        impl SubAssign for $wrap_ty {
            fn sub_assign(&mut self, other: Self) {
                *self = *self - other;
            }
        }

    }
}

add_math_ops!(BigEndian<u8>);
add_math_ops!(BigEndian<i8>);
add_math_ops!(BigEndian<u16>);
add_math_ops!(BigEndian<i16>);
add_math_ops!(BigEndian<u32>);
add_math_ops!(BigEndian<i32>);
add_math_ops!(BigEndian<u64>);
add_math_ops!(BigEndian<i64>);
add_math_ops!(BigEndian<u128>);
add_math_ops!(BigEndian<i128>);
add_math_ops!(BigEndian<usize>);
add_math_ops!(BigEndian<isize>);
add_math_ops!(BigEndian<f32>);
add_math_ops!(BigEndian<f64>);

add_math_ops!(LittleEndian<u8>);
add_math_ops!(LittleEndian<i8>);
add_math_ops!(LittleEndian<u16>);
add_math_ops!(LittleEndian<i16>);
add_math_ops!(LittleEndian<u32>);
add_math_ops!(LittleEndian<i32>);
add_math_ops!(LittleEndian<u64>);
add_math_ops!(LittleEndian<i64>);
add_math_ops!(LittleEndian<u128>);
add_math_ops!(LittleEndian<i128>);
add_math_ops!(LittleEndian<usize>);
add_math_ops!(LittleEndian<isize>);
add_math_ops!(LittleEndian<f32>);
add_math_ops!(LittleEndian<f64>);


// Rust's orphan trait rule prevents us from using a generic implementation on the primitive types, so we do this:
macro_rules! make_primitive_type_from_be {
    ($wrap_ty:ty) => {

        impl From<BigEndian<$wrap_ty>> for $wrap_ty {
            fn from(v: BigEndian<$wrap_ty>) -> $wrap_ty {
                v.0.from_big_endian()
            }
        }

    }
}

make_primitive_type_from_be!(bool);
make_primitive_type_from_be!(u8);
make_primitive_type_from_be!(i8);
make_primitive_type_from_be!(u16);
make_primitive_type_from_be!(i16);
make_primitive_type_from_be!(u32);
make_primitive_type_from_be!(i32);
make_primitive_type_from_be!(u64);
make_primitive_type_from_be!(i64);
make_primitive_type_from_be!(u128);
make_primitive_type_from_be!(i128);
make_primitive_type_from_be!(usize);
make_primitive_type_from_be!(isize);
make_primitive_type_from_be!(f32);
make_primitive_type_from_be!(f64);

/// A little-endian struct that basically just uses the type system to tag the endianness.
#[derive(Copy, Clone, Hash, Debug)]
pub struct LittleEndian<T: SpecificEndian<T>> (T);
unsafe impl<T: Send + SpecificEndian<T>> Send for LittleEndian<T> {}
unsafe impl<T: Sync + SpecificEndian<T>> Sync for LittleEndian<T> {}

impl<T> LittleEndian<T> where T: SpecificEndian<T> {
    pub fn raw(&self) -> T {
        self.0
    }
    pub fn to_native(&self) -> T {
        T::from_little_endian(&self.0)
    }

}

impl<T: SpecificEndian<T>> From<T> for LittleEndian<T> {
    fn from(v: T) -> LittleEndian<T> {
        LittleEndian::<T>(v.to_little_endian())
    }
}

// Rust's orphan trait rule prevents us from using a generic implementation on the primitive types, so we do this:
macro_rules! make_primitive_type_from_le {
    ($wrap_ty:ty) => {

        impl From<LittleEndian<$wrap_ty>> for $wrap_ty {
            fn from(v: LittleEndian<$wrap_ty>) -> $wrap_ty {
                v.0.from_little_endian()
            }
        }

    }
}

make_primitive_type_from_le!(bool);
make_primitive_type_from_le!(u8);
make_primitive_type_from_le!(i8);
make_primitive_type_from_le!(u16);
make_primitive_type_from_le!(i16);
make_primitive_type_from_le!(u32);
make_primitive_type_from_le!(i32);
make_primitive_type_from_le!(u64);
make_primitive_type_from_le!(i64);
make_primitive_type_from_le!(u128);
make_primitive_type_from_le!(i128);
make_primitive_type_from_le!(usize);
make_primitive_type_from_le!(isize);
make_primitive_type_from_le!(f32);
make_primitive_type_from_le!(f64);



impl<T: UpperHex + SpecificEndian<T>> UpperHex for BigEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:X}", self.to_native()) // delegate to i32's implementation
    }
}
impl<T: UpperHex + SpecificEndian<T>> UpperHex for LittleEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:X}", self.to_native()) // delegate to i32's implementation
    }
}


impl<T: LowerHex + SpecificEndian<T>> LowerHex for BigEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:x}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: LowerHex + SpecificEndian<T>> LowerHex for LittleEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:x}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: Octal + SpecificEndian<T>> Octal for BigEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:o}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: Octal + SpecificEndian<T>> Octal for LittleEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:o}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: Binary + SpecificEndian<T>> Binary for BigEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:b}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: Binary + SpecificEndian<T>> Binary for LittleEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:b}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: Display + SpecificEndian<T>> Display for BigEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.to_native()) // delegate to i32's implementation
    }
}

impl<T: Display + SpecificEndian<T>> Display for LittleEndian<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.to_native()) // delegate to i32's implementation
    }
}

// Shortcut types:
#[allow(non_camel_case_types)]
pub type u16le = LittleEndian<u16>;
#[allow(non_camel_case_types)]
pub type u16be = BigEndian<u16>;
#[allow(non_camel_case_types)]
pub type u32le = LittleEndian<u32>;
#[allow(non_camel_case_types)]
pub type u32be = BigEndian<u32>;
#[allow(non_camel_case_types)]
pub type u64le = LittleEndian<u64>;
#[allow(non_camel_case_types)]
pub type u64be = BigEndian<u64>;
#[allow(non_camel_case_types)]
pub type u128le = LittleEndian<u128>;
#[allow(non_camel_case_types)]
pub type u128be = BigEndian<u128>;
#[allow(non_camel_case_types)]
pub type usizebe = BigEndian<usize>;

#[allow(non_camel_case_types)]
pub type i16le = LittleEndian<i16>;
#[allow(non_camel_case_types)]
pub type i16be = BigEndian<i16>;
#[allow(non_camel_case_types)]
pub type i32le = LittleEndian<i32>;
#[allow(non_camel_case_types)]
pub type i32be = BigEndian<i32>;
#[allow(non_camel_case_types)]
pub type i64le = LittleEndian<i64>;
#[allow(non_camel_case_types)]
pub type i64be = BigEndian<i64>;
#[allow(non_camel_case_types)]
pub type i128le = LittleEndian<i128>;
#[allow(non_camel_case_types)]
pub type i128be = BigEndian<i128>;
#[allow(non_camel_case_types)]
pub type isizebe = BigEndian<isize>;

#[allow(non_camel_case_types)]
pub type f32le = LittleEndian<f32>;
#[allow(non_camel_case_types)]
pub type f32be = BigEndian<f32>;

#[allow(non_camel_case_types)]
pub type f64le = LittleEndian<f64>;
#[allow(non_camel_case_types)]
pub type f64be = BigEndian<f64>;


#[cfg(test)]
mod tests {
    use super::*;

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
        struct Foo (
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

        let _foo = Foo(0.into(), 1.into(), 2.into(), 3.into(), 4.into(), 5.into(), 6.into(), 7.into(), 8.into(), 9.into(), 10.into(), 11.into(), 12.into(), 13.into(), 14.into(), 15.into(), (0.1).into(), (123.5).into(), (7.8).into(), (12345.4567).into());
    }

    #[test]
    fn store_be() {
        let be: BigEndian<u64> = 0xfe.into();
        if cfg!(byte_order = "big endian") {
            assert_eq!(be.raw(), 0xfe);
        }
        else {
            assert_eq!(be.raw(), 0xfe00000000000000);
        }
    }

    #[test]
    fn store_le() {
        let le: LittleEndian<u64> = 0xfe.into();
        if cfg!(byte_order = "big endian") {
            assert_eq!(le.raw(), 0xfe00000000000000);
        }
        else {
            assert_eq!(le.raw(), 0xfe);
        }
    }


    #[test]
    fn cast() {
        let be = BigEndian::<u64>::from(12345);
        let ne: u64 = be.into();
        assert_eq!(ne, 12345);
    }

    #[test]
    fn convert_back() {
        let be = BigEndian::<u64>::from(12345);
        println!("{}", u64::from(be));
    }

    #[test]
    fn convert_to_native() {
        let be = BigEndian::<u64>::from(0xfe);
        println!("{:x}, {:x}", be.0, be.to_native());
        assert_eq!(0xfe, be.to_native());
    }

    #[test]
    fn equality_test() {
        let be1 = BigEndian::<u64>::from(12345);
        let be2 = BigEndian::<u64>::from(12345);
        assert_eq!(true, be1 == be2);
    }

    #[test]
    fn not_equality_test() {
        let be1 = BigEndian::<u64>::from(12345);
        let be2 = BigEndian::<u64>::from(34565);
        assert_eq!(true, be1 != be2);
    }

    #[test]
    fn lt_test() {
        let be1 = BigEndian::<u64>::from(12345);
        let be2 = BigEndian::<u64>::from(34565);
        assert_eq!(true, be1 < be2);
    }

    #[test]
    fn bit_and_test() {
        let be1 = LittleEndian::<u64>::from(0x0f0);
        let be2 = LittleEndian::<u64>::from(0xff0);
        assert_eq!(0x0f0, u64::from(be1 & be2));
    }

    #[test]
    fn unary_not_test() {
        let be1 = BigEndian::<u16>::from(0x0f0);
        assert_eq!(0xff0f, u16::from(!be1));
    }

    #[test]
    fn store_fp_be() {
        let be1 = BigEndian::<f64>::from(1234.5678);
        if cfg!(byte_order = "little endian") {
            assert_ne!(1234.5678, be1.0);
        }
        assert_eq!(1234.5678, f64::from(be1));
    }

    #[test]
    fn store_fp_le() {
        let le1 = LittleEndian::<f64>::from(1234.5678);
        if cfg!(byte_order = "big endian") {
            assert_ne!(1234.5678, le1.0);
        }
        assert_eq!(1234.5678, f64::from(le1));
    }

    #[test]
    fn operate_fp_be() {
        let be1 = BigEndian::<f64>::from(1234.5678);
        let be2 = BigEndian::<f64>::from(6234.5678);
        assert_eq!(true, be1 < be2);
    }

    #[test]
    fn add_fp_be() {
        let mut be1 = f64be::from(1234.5678);
        be1 = be1 + 1.0.into();
        be1 += 1.0.into();
        assert_eq!(be1, 1236.5678.into());
    }

    #[test]
    fn subtract_fp_be() {
        let mut be1 = f64be::from(1234.5678);
        be1 = be1 - 1.0.into();
        be1 -= 1.0.into();
        assert_eq!(be1, 1232.5678.into());
    }

    #[test]
    fn mul_fp_be() {
        let mut be1 = f64be::from(1234.5678);
        be1 = be1 * 10.0.into();
        be1 *= 10.0.into();
        assert_eq!(be1, 123456.78.into());
    }

    #[test]
    fn div_fp_be() {
        let mut ne1: f64 = 1234.5678;
        let mut be1 = f64be::from(ne1);
        be1 = be1 / 10.0.into();
        ne1 = ne1 / 10.0;
        be1 /= 10.0.into();
        ne1 /= 10.0;
        assert_eq!(ne1, be1.into());
    }

    #[test]
    fn shl_be() {
        let mut ne1 = 0xfee1;
        let mut be1 = u64be::from(ne1);
        be1 = be1 << 5.into();
        ne1 = ne1 << 5;
        be1 <<= 5.into();
        ne1 <<= 5;
        assert_eq!(ne1, be1.into());
    }

    #[test]
    fn shr_be() {
        let mut ne1 = 0xfee1;
        let mut be1 = u64be::from(ne1);
        be1 = be1 >> 5.into();
        ne1 = ne1 >> 5;
        be1 >>= 5.into();
        ne1 >>= 5;
        assert_eq!(ne1, be1.into());
    }

    #[test]
    fn inferred_type() {
        let mut be1 = BigEndian::from(1234);
        be1 &= BigEndian::from(5678);
        println!("{} {} {}", be1, be1.raw(), be1.to_native());
        assert_eq!(be1, 1026.into());
    }

    #[test]
    fn inferred_type_fp() {
        let mut be1 = BigEndian::from(1234.5);
        be1 += BigEndian::from(5678.1);
        println!("{} {} {}", be1, be1.raw(), be1.to_native());
        assert_eq!(be1, 6912.6.into());
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
                    EndianAwareExample::LittleEndianFunction(v) => EndianAwareExample::BigEndianFunction(v.to_big_endian()),
                }
            }
            fn to_little_endian(&self) -> Self {
                match self {
                    EndianAwareExample::LittleEndianFunction(_v) => *self,
                    EndianAwareExample::BigEndianFunction(v) => EndianAwareExample::BigEndianFunction(v.to_little_endian()),
                }
            }
            fn from_big_endian(&self) -> Self {
                match self {
                    EndianAwareExample::BigEndianFunction(_v) => *self,
                    EndianAwareExample::LittleEndianFunction(v) => EndianAwareExample::BigEndianFunction(v.to_big_endian()),
                }
            }
            fn from_little_endian(&self) -> Self {
                match self {
                    EndianAwareExample::LittleEndianFunction(_v) => *self,
                    EndianAwareExample::BigEndianFunction(v) => EndianAwareExample::BigEndianFunction(v.to_little_endian()),
                }
            }

        }
        let foo: BigEndian<EndianAwareExample> = EndianAwareExample::LittleEndianFunction(0xf0).into();
        #[allow(unused_assignments)]
        let mut value = 0;
        match foo.to_native() {
            EndianAwareExample::BigEndianFunction(v) => { println!("be: {:x}", v); value = v }
            EndianAwareExample::LittleEndianFunction(v) => { println!("le: {:x}", v); value = 0 }
        }
        assert_eq!(value, 0x0f000000000000000);
    }

}

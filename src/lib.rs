#![feature(test)]
//! Many byte-order-handling libraries focus on providing code to convert to and from big- or little-endian.  However,
//! this requires users of those libraries to use a lot of explicit logic.  This library uses the Rust type system to
//! enforce conversions invisibly, and also ensure that they are done consistently.  A struct member can be read and written
//! simply using the standard From and Into trait methods (from() and into()).  No explicit endian checks are required.
//!  
//! # Example 1:
//! 
//!```rust
//! use simple_endian::*;
//!
//! fn init() {
//!     #[repr(C)]
//!     struct BinPacket {
//!         a: u64be,
//!         b: u32be,
//!     }
//!     let mut bp = BinPacket{a: 0xfe.into(), b: 10.into()};
//!     let new_a = bp.a.to_native() * 1234; 
 
//!     bp.a = new_a.into();
//!     bp.b = 1234.into();
//! }
//! ```
//! 
//! Trying to write `bp.a = new_a;` causes an error because the type u64 can't be directly stored.
//! 
//! # Example 2:
//! 
//! Of course, just storing things in memory isn't that useful unless you write somewhere.
//! 
//! ```rust
//! use simple_endian::*;
//! use std::fs::File;
//! use std::io::prelude::*;
//! use std::mem::{transmute, size_of};
//! 
//! // We have to specify a representation in order to define the layout.
//! #[repr(C)]
//! struct BinBEStruct {
//!     pub a: u64be,
//!     b: u64be,
//!     c: f64be,
//! }
//! 
//! fn main() -> std::io::Result<()> {
//!    let bin_struct = BinBEStruct{a: 345.into(), b: 0xfee.into(), c: 9.345.into()};
//!
//!    let mut pos = 0;
//!    let mut data_file = File::create(".test.bin")?;
//!    let buffer = unsafe { transmute::<&BinBEStruct, &[u8; size_of::<BinBEStruct>()]>(&bin_struct) };
//!
//!    while pos < buffer.len() {
//!        let bytes_written = data_file.write(&buffer[pos..])?;
//!        pos += bytes_written;
//!    }
//!    Ok(())
//! }
//! ```
//! 

use std::{
    cmp::Ordering,
    ops::{BitAnd, Not, Add, AddAssign, Div, DivAssign, Mul, MulAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign, BitAndAssign, BitXor, BitXorAssign, BitOr, BitOrAssign},
    fmt::{Formatter, Result, UpperHex, LowerHex, Octal, Binary, Display},
};

/// Any object implementing SpecificEndian<T> can be converted between big and little endian.  Implement this trait to allow for endian conversion by this crate.
pub trait SpecificEndian<T> where Self: Into<T> + Clone + Copy {
    fn to_big_endian(&self) -> T;
    fn to_little_endian(&self) -> T;
    fn from_big_endian(&self) -> T;
    fn from_little_endian(&self) -> T;

}

/// A macro implementing SpecificEndian<T> for simple data types where big and little endian forms are the same.
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

/// A macro for implementing SpecificEndian<T> on types that have endian conversions built into Rust.  Currently, this is the primitive integer types.
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
    }
}

make_specific_endian_float!(f32);
make_specific_endian_float!(f64);

/// A big-endian representation of type T that implements SpecificEndian<T>.  Data stored in the struct must be converted to big-endian using from() or into().
#[derive(Copy, Clone, Hash, Debug)]
#[repr(transparent)]
pub struct BigEndian<T: SpecificEndian<T>> {_v: T}
unsafe impl<T: Send + SpecificEndian<T>> Send for BigEndian<T> {}
unsafe impl<T: Sync + SpecificEndian<T>> Sync for BigEndian<T> {}


impl<T> BigEndian<T> where T: SpecificEndian<T> {
    /// Returns the raw data stored in the struct.
    pub fn to_bits(&self) -> T {
        self._v
    }
    /// Imports the data raw into a BigEndian<T> struct.
    pub fn from_bits(v: T) -> Self {
        Self{_v: v}
    }
    /// Converts the data to the same type T in host-native endian.
    pub fn to_native(&self) -> T {
        T::from_big_endian(&self._v)
    }
}

impl<T: SpecificEndian<T>> From<T> for BigEndian<T> {
    fn from(v: T) -> BigEndian<T> {
        BigEndian::<T>{_v: v.to_big_endian()}
    }
}


/// A little-endian representation of type T that implements SpecificEndian<T>.  Data stored in the struct must be converted to little-endian using from() or into().
#[derive(Copy, Clone, Hash, Debug)]
#[repr(transparent)]
pub struct LittleEndian<T: SpecificEndian<T>> {_v: T}
unsafe impl<T: Send + SpecificEndian<T>> Send for LittleEndian<T> {}
unsafe impl<T: Sync + SpecificEndian<T>> Sync for LittleEndian<T> {}

impl<T> LittleEndian<T> where T: SpecificEndian<T> {
    /// Returns the raw data stored in the struct.
    pub fn to_bits(&self) -> T {
        self._v
    }
    /// Imports the data raw into a LittleEndian<T> struct.
    pub fn from_bits(v: T) -> Self {
        Self{_v: v}
    }
    /// Converts the data to the same type T in host-native endian.
    pub fn to_native(&self) -> T {
        T::from_little_endian(&self._v)
    }
}

impl<T: SpecificEndian<T>> From<T> for LittleEndian<T> {
    fn from(v: T) -> LittleEndian<T> {
        LittleEndian::<T>{_v: v.to_little_endian()}
    }
}
macro_rules! add_equality_ops {
    ($wrap_ty:ty) => {
        impl PartialEq for $wrap_ty {
            fn eq(&self, other: &Self) -> bool {
                self._v == other._v
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
                Self{_v: self._v & rhs._v}
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
                Self{_v: self._v ^ rhs._v}
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
                Self{_v: self._v | rhs._v}
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
                Self{_v: !self._v}
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
                v._v.from_big_endian()
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

// Rust's orphan trait rule prevents us from using a generic implementation on the primitive types, so we do this:
macro_rules! make_primitive_type_from_le {
    ($wrap_ty:ty) => {

        impl From<LittleEndian<$wrap_ty>> for $wrap_ty {
            fn from(v: LittleEndian<$wrap_ty>) -> $wrap_ty {
                v._v.from_little_endian()
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

/// Shorthand for LittleEndian<u16> 
#[allow(non_camel_case_types)]
pub type u16le = LittleEndian<u16>;
/// Shorthand for BigEndian<u16> 
#[allow(non_camel_case_types)]
pub type u16be = BigEndian<u16>;
/// Shorthand for LittleEndian<u32> 
#[allow(non_camel_case_types)]
pub type u32le = LittleEndian<u32>;
/// Shorthand for BigEndian<u32> 
#[allow(non_camel_case_types)]
pub type u32be = BigEndian<u32>;
/// Shorthand for LittleEndian<u64> 
#[allow(non_camel_case_types)]
pub type u64le = LittleEndian<u64>;
/// Shorthand for BigEndian<u64> 
#[allow(non_camel_case_types)]
pub type u64be = BigEndian<u64>;
/// Shorthand for LittleEndian<u128>
#[allow(non_camel_case_types)]
pub type u128le = LittleEndian<u128>;
/// Shorthand for BigEndian<u128> 
#[allow(non_camel_case_types)]
pub type u128be = BigEndian<u128>;
/// Shorthand for LittleEndian<usize>
#[allow(non_camel_case_types)]
pub type usizele = LittleEndian<usize>;
/// Shorthand for BigEndian<usize> 
#[allow(non_camel_case_types)]
pub type usizebe = BigEndian<usize>;

/// Shorthand for LittleEndian<i16>
#[allow(non_camel_case_types)]
pub type i16le = LittleEndian<i16>;
/// Shorthand for BigEndian<i16>
#[allow(non_camel_case_types)]
pub type i16be = BigEndian<i16>;
/// Shorthand for LittleEndian<i32>
#[allow(non_camel_case_types)]
pub type i32le = LittleEndian<i32>;
/// Shorthand for BigEndian<i32>
#[allow(non_camel_case_types)]
pub type i32be = BigEndian<i32>;
/// Shorthand for LittleEndian<i64>
#[allow(non_camel_case_types)]
pub type i64le = LittleEndian<i64>;
/// Shorthand for BigEndian<i64>
#[allow(non_camel_case_types)]
pub type i64be = BigEndian<i64>;
/// Shorthand for LittleEndian<i128>
#[allow(non_camel_case_types)]
pub type i128le = LittleEndian<i128>;
/// Shorthand for BigEndian<i128>
#[allow(non_camel_case_types)]
pub type i128be = BigEndian<i128>;
/// Shorthand for LittleEndian<isize>
#[allow(non_camel_case_types)]
pub type isizele = LittleEndian<isize>;
/// Shorthand for BigEndian<isize>
#[allow(non_camel_case_types)]
pub type isizebe = BigEndian<isize>;

/// Shorthand for LittleEndian<f32>
#[allow(non_camel_case_types)]
pub type f32le = LittleEndian<f32>;
/// Shorthand for BigEndian<f32>
#[allow(non_camel_case_types)]
pub type f32be = BigEndian<f32>;

/// Shorthand for LittleEndian<f64>
#[allow(non_camel_case_types)]
pub type f64le = LittleEndian<f64>;
/// Shorthand for BigEndian<f64>
#[allow(non_camel_case_types)]
pub type f64be = BigEndian<f64>;


#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use test::Bencher;
    use std::mem::size_of;


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
            assert_eq!(be.to_bits(), 0xfe);
        }
        else {
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
        }
        else {
            assert_eq!(le.to_bits(), 0xfe);
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
        println!("{:x}, {:x}", be._v, be.to_native());
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

    #[bench]
    fn bench_integer_be(b: &mut Bencher) {
        b.iter(|| {
            let mut a = BigEndian::from(1234567890);
            for _ in 0..10 {
                a += BigEndian::from(101010);
                a &= BigEndian::from(0xf0f0f0);
                a *= BigEndian::from(123);
                a /= BigEndian::from(543);
                println!("{}", a);
            }
        });
    }
    #[bench]
    fn bench_integer_le(b: &mut Bencher) {
        b.iter(|| {
            let mut a = LittleEndian::from(1234567890);
            for _ in 0..10 {
                a += LittleEndian::from(101010);
                a &= LittleEndian::from(0xf0f0f0);
                a *= LittleEndian::from(123);
                a /= LittleEndian::from(543);
                println!("{}", a);
            }
        });
    }
    #[bench]
    fn bench_integer_ne(b: &mut Bencher) {
        b.iter(|| {
            let mut a = 1234567890;
            for _ in 0..10 {
                a += 101010;
                a &= 0xf0f0f0;
                a *= 123;
                a /= 543;
                println!("{}", a);
            }
        });
    }

    #[bench]
    fn bench_fp_be(b: &mut Bencher) {
        b.iter(|| {
            let mut a = BigEndian::from(1234567890.1);
            for _ in 0..10 {
                a += BigEndian::from(101010.0);
                a *= BigEndian::from(123.0);
                a /= BigEndian::from(543.0);
                println!("{}", a);
            }
        });
    }
    #[bench]
    fn bench_fp_le(b: &mut Bencher) {
        b.iter(|| {
            let mut a = LittleEndian::from(1234567890.1);
            for _ in 0..10 {
                a += LittleEndian::from(101010.0);
                a *= LittleEndian::from(123.0);
                a /= LittleEndian::from(543.0);
                println!("{}", a);
            }
        });
    }
    #[bench]
    fn bench_fp_ne(b: &mut Bencher) {
        b.iter(|| {
            let mut a = 1234567890.1;
            for _ in 0..10 {
                a += 101010.0;
                a *= 123.0;
                a /= 543.0;
                println!("{}", a);
            }
        });
    }

    #[bench]
    fn base_endian_test_be(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
               let a = i32::from_be(0xa5a5a5);
               println!("{}", a);
            }
        });
    }
    #[bench]
    fn base_endian_test_le(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
               let a = i32::from_le(0xa5a5a5);
               println!("{}", a);
            }
        });
    }
    #[bench]
    fn base_endian_test_ne(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
               let a = 0xa5a5a5_i32;
               println!("{}", a);
            }
        });
    }
    #[bench]
    fn base_endian_test_structured(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
               let a = LittleEndian{_v: 0xa5a5a5_i32};
               println!("{}", a);
            }
        });
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

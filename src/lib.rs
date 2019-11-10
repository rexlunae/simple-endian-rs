/// Many byte-order-handling libraries focus on providing code to convert to and from big- or little-endian.  However,
/// this requires users of those libraries to use a lot of explicit logic.  This library uses the Rust type system to
/// enforce conversions invisibly, and also ensure that they are done consistently.  A struct member can be read and written
/// simply using the standard From and Into trait methods (from() and into()).  No explicit endian checks are required.
///  
/// # Example 1:
/// 
/// use simple_endian::*;
/// 
///    fn init() {
///        struct BinPacket {
///            a: u64be,
///            b: u32be,
///        }
///        let mut bp = BinPacket{a: 0xfe.into(), b: 10.into()};
///        let new_a = bp.a.to_native() * 1234; 
 
///        bp.a = new_a.into();
///        bp.b = 1234.into();
///    }
/// 
/// Trying to write `bp.a = new_a;` causes an error because the type u64 can't be directly stored.
/// 

use std::{
    cmp::Ordering,
    ops::{BitAnd, Not, AddAssign, BitAndAssign, BitXor, BitXorAssign, BitOr, BitOrAssign},
    fmt::{Formatter, Result, UpperHex, LowerHex, Octal, Binary},
};

/// A type with a specific endian.
pub trait SpecificEndian<T> where Self: Into<T> {
    fn to_native(self) -> T {
        self.into()
    }
}

/// Generates a type with big and little endian variants.  Usually, this will be internal.
macro_rules! make_known_endian {
    ($wrap_ty:ty, $mod_name:ident, $be_name:ident, $le_name:ident) => {
        pub mod $mod_name {
            use super::*;

            #[derive(Clone, Copy, Hash)]
            #[allow(non_camel_case_types)]
            pub struct $be_name ($wrap_ty);
            #[derive(Clone, Copy, Hash)]
            #[allow(non_camel_case_types)]
            pub struct $le_name ($wrap_ty);

            impl $be_name {
                pub fn raw(&self) -> $wrap_ty {
                    self.0
                }
            }

            impl $le_name {
                pub fn raw(&self) -> $wrap_ty {
                    self.0
                }
            }

            impl From<$wrap_ty> for $be_name {
                fn from(v: $wrap_ty) -> Self {
                    Self(v.to_be())
                }
            }
            impl From<$be_name> for $wrap_ty {
                fn from(v: $be_name) -> Self {
                    Self::from_be(v.0)
                }
            }

            impl From<$wrap_ty> for $le_name {
                fn from(v: $wrap_ty) -> Self {
                    Self(v.to_le())
                }
            }
            impl From<$le_name> for $wrap_ty {
                fn from(v: $le_name) -> Self {
                    Self::from_le(v.0)
                }
            }

            impl SpecificEndian<$wrap_ty> for $be_name {}
            impl SpecificEndian<$wrap_ty> for $le_name {}

            impl PartialEq for $be_name {
                fn eq(&self, other: &Self) -> bool {
                    self.0 == other.0
                }
            }
            impl Eq for $be_name {}

            impl PartialEq for $le_name {
                fn eq(&self, other: &Self) -> bool {
                    self.0 == other.0
                }
            }
            impl Eq for $le_name {}

            impl Ord for $be_name {
                fn cmp(&self, other: &Self) -> Ordering {
                    self.to_native().cmp(&other.to_native())
                }
            }
            impl PartialOrd for $be_name {
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    Some(self.cmp(other))
                }
            }

            impl Ord for $le_name {
                fn cmp(&self, other: &Self) -> Ordering {
                    self.to_native().cmp(&other.to_native())
                }
            }
            impl PartialOrd for $le_name {
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    Some(self.cmp(other))
                }
            }

            impl BitAnd for $be_name {
                type Output = Self;
                fn bitand(self, rhs: Self) -> Self::Output {
                    Self(self.0 & rhs.0)
                }
            }
            impl BitAnd for $le_name {
                type Output = Self;
                fn bitand(self, rhs: Self) -> Self::Output {
                    Self(self.0 & rhs.0)
                }
            }

            impl BitAndAssign for $be_name {
               fn bitand_assign(&mut self, rhs: Self) {
                    *self = *self & rhs
                }
            }
            impl BitAndAssign for $le_name {
               fn bitand_assign(&mut self, rhs: Self) {
                    *self = *self & rhs
                }
            }

            impl BitXor for $be_name {
                // We don't need to convert endian for this op.
                type Output = Self;

                fn bitxor(self, rhs: Self) -> Self::Output {
                    Self(self.0 ^ rhs.0)
                }
            }

            impl BitXor for $le_name {
                // We don't need to convert endian for this op.
                type Output = Self;

                fn bitxor(self, rhs: Self) -> Self::Output {
                    Self(self.0 ^ rhs.0)
                }
            }

            impl BitXorAssign for $be_name {
                fn bitxor_assign(&mut self, rhs: Self) {
                    *self = *self ^ rhs
                }
            }
            impl BitXorAssign for $le_name {
                fn bitxor_assign(&mut self, rhs: Self) {
                    *self = *self ^ rhs
                }
            }

            impl BitOr for $be_name {
                type Output = Self;

                fn bitor(self, rhs: Self) -> Self {
                    Self(self.0 | rhs.0)
                }
            }

            impl BitOr for $le_name {
                type Output = Self;

                fn bitor(self, rhs: Self) -> Self {
                    Self(self.0 | rhs.0)
                }
            }

            impl BitOrAssign for $be_name {
                fn bitor_assign(&mut self, rhs: Self) {
                    *self = *self | rhs;
                }
            }

            impl BitOrAssign for $le_name {
                fn bitor_assign(&mut self, rhs: Self) {
                    *self = *self | rhs;
                }
            }

            impl AddAssign for $be_name {
                fn add_assign(&mut self, other: Self) {
                    *self = Self::from(self.to_native() + other.to_native());
                }
            }
            impl AddAssign for $le_name {
                fn add_assign(&mut self, other: Self) {
                    *self = Self::from(self.to_native() + other.to_native());
                }
            }

            impl Not for $be_name {
                type Output = Self;

                fn not(self) -> Self::Output {
                    Self::from(!self.to_native())
                }
            }

            impl Not for $le_name {
                type Output = Self;

                fn not(self) -> Self::Output {
                    Self::from(!self.to_native())
                }
            }

            impl UpperHex for $be_name {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    write!(f, "{:X}", self.to_native()) // delegate to i32's implementation
                }
            }
            impl UpperHex for $le_name {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    write!(f, "{:X}", self.to_native()) // delegate to i32's implementation
                }
            }

            impl LowerHex for $be_name {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    write!(f, "{:x}", self.to_native()) // delegate to i32's implementation
                }
            }
            impl LowerHex for $le_name {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    write!(f, "{:x}", self.to_native()) // delegate to i32's implementation
                }
            }

            impl Octal for $be_name {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    write!(f, "{:o}", self.to_native()) // delegate to i32's implementation
                }
            }
            impl Octal for $le_name {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    write!(f, "{:o}", self.to_native()) // delegate to i32's implementation
                }
            }

            impl Binary for $be_name {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    write!(f, "{:b}", self.to_native()) // delegate to i32's implementation
                }
            }
            impl Binary for $le_name {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                    write!(f, "{:b}", self.to_native()) // delegate to i32's implementation
                }
            }

        }
    }
}

// Generate the actual definitions.
make_known_endian!(isize, isize_endian, isizebe, isizele);
pub use isize_endian::*;
make_known_endian!(usize, usize_endian, usizebe, usizele);
pub use usize_endian::*;
make_known_endian!(i16, i16_endian, i16be, i16le);
pub use i16_endian::*;
make_known_endian!(u16, u16_endian, u16be, u16le);
pub use u16_endian::*;
make_known_endian!(i32, i32_endian, i32be, i32le);
pub use i32_endian::*;
make_known_endian!(u32, u32_endian, u32be, u32le);
pub use u32_endian::*;
make_known_endian!(i64, i64_endian, i64be, i64le);
pub use i64_endian::*;
make_known_endian!(u64, u64_endian, u64be, u64le);
pub use u64_endian::*;
make_known_endian!(i128, i128_endian, i128be, i128le);
pub use i128_endian::*;
make_known_endian!(u128, u128_endian, u128be, u128le);
pub use u128_endian::*;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declare_all() {
        let _a: i16be = 0xfe.into();
        let _a: i16le = 0xfe.into();
        let _a: u16be = 0xfe.into();
        let _a: u16le = 0xfe.into();

        let _a: i32be = 0xfe.into();
        let _a: i32le = 0xfe.into();
        let _a: u32be = 0xfe.into();
        let _a: u32le = 0xfe.into();

        let _a: i64be = 0xfe.into();
        let _a: i64le = 0xfe.into();
        let _a: u64be = 0xfe.into();
        let _a: u64le = 0xfe.into();

        let _a: i128be = 0xfe.into();
        let _a: i128le = 0xfe.into();
        let _a: u128be = 0xfe.into();
        let _a: u128le = 0xfe.into();
    }

    #[test]
    fn make_struct() {
        struct Foo (
            i16be,
            i16le,
            u16be,
            u16le,

            i32be,
            i32le,
            u32be,
            u32le,

            i64be,
            i64le,
            u64be,
            u64le,

            i128be,
            i128le,
            u128be,
            u128le,
        );

        let _foo = Foo(0.into(), 1.into(), 2.into(), 3.into(), 4.into(), 5.into(), 6.into(), 7.into(), 8.into(), 9.into(), 10.into(), 11.into(), 12.into(), 13.into(), 14.into(), 15.into());
    }


    #[test]
    fn store_be() {
        if cfg!(byte_order = "big endian") {
            let be: u64be = 0xfe.into();
            assert_eq!(be.raw(), 0xfe);
        }
        else {
            let be: u64be = 0xfe.into();
            assert_eq!(be.raw(), 0xfe00000000000000);
        }
    }

    #[test]
    fn store_le() {
        if cfg!(byte_order = "big endian") {
            let le: u64le = 0xfe.into();
            assert_eq!(le.raw(), 0xfe00000000000000);
        }
        else {
            let le: u64le = 0xfe.into();
            assert_eq!(le.raw(), 0xfe);
        }
    }


    #[test]
    fn cast() {
        let be = u64be::from(12345);
        let ne: u64 = be.into();
        assert_eq!(ne, 12345);
    }

    #[test]
    fn convert_back() {
        let be = u64be::from(12345);
        println!("{}", u64::from(be));
    }

    #[test]
    fn convert_to_native() {
        let be = u64be::from(12345);
        assert_eq!(12345, be.to_native());
    }

    #[test]
    fn equality_test() {
        let be1 = u64be::from(12345);
        let be2 = u64be::from(12345);
        assert_eq!(true, be1 == be2);
    }

    #[test]
    fn not_equality_test() {
        let be1 = u64be::from(12345);
        let be2 = u64be::from(34565);
        assert_eq!(true, be1 != be2);
    }

    #[test]
    fn lt_test() {
        let be1 = u64be::from(12345);
        let be2 = u64be::from(34565);
        assert_eq!(true, be1 < be2);
    }

    #[test]
    fn bit_and_test() {
        let be1 = u64be::from(0x0f0);
        let be2 = u64be::from(0xff0);
        assert_eq!(0x0f0, u64::from(be1 & be2));
    }

    #[test]
    fn unary_not_test() {
        let be1 = u16be::from(0x0f0);
        assert_eq!(0xff0f, u16::from(!be1));
    }

}

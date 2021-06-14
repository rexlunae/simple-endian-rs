//! Bitwise operations.  These should be equally fast in either endian.
//!
//! ```rust
//! // These should all be basically zero-cost:
//! use simple_endian::*;
//! let mut a = BigEndian::from(0xf8dc);
//! let mask = BigEndian::from(0xf0f0f);
//! a &= mask;
//! a |= BigEndian::from(0xfff0000) | mask;
//! a ^= 0x5555555.into();
//! ```

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use super::*;

/// Implement the bitwise operations on the types.  These should be as fast in either endian, because they are endian-agnostic.
#[allow(unused_macros)]
macro_rules! add_bitwise_ops {
    ($wrap_ty:ty) => {
        impl BitAnd for $wrap_ty {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self::Output {
                Self {
                    _v: self._v & rhs._v,
                }
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
                Self {
                    _v: self._v ^ rhs._v,
                }
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
                Self {
                    _v: self._v | rhs._v,
                }
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
                Self { _v: !self._v }
            }
        }
    };
}

#[cfg(feature = "byte_impls")]
mod bitwise_byte_ops {
    use super::*;
    #[cfg(feature = "big_endian")]
    mod be {
        use super::*;
        add_bitwise_ops!(BigEndian<bool>);
        add_bitwise_ops!(BigEndian<u8>);
        add_bitwise_ops!(BigEndian<i8>);
    }
    #[cfg(feature = "little_endian")]
    mod le {
        use super::*;
        add_bitwise_ops!(LittleEndian<bool>);
        add_bitwise_ops!(LittleEndian<u8>);
        add_bitwise_ops!(LittleEndian<i8>);
    }
}

#[cfg(feature = "integer_impls")]
mod bitwise_integer_ops {
    use super::*;
    #[cfg(feature = "big_endian")]
    mod be {
        use super::*;
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
    }

    #[cfg(feature = "little_endian")]
    mod le {
        use super::*;
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
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use crate::*;

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
}

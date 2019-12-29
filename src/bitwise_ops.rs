use std::{
    cmp::Ordering,
    ops::{BitAnd, Not, BitAndAssign, BitXor, BitXorAssign, BitOr, BitOrAssign},
};

use super::*;

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

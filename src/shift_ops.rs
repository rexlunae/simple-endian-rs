use std::ops::{Shl, ShlAssign, Shr, ShrAssign};

use super::*;

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

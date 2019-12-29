use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use super::*;

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

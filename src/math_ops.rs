//! The math operations.  These all have some cost because they require conversion to native endian.
#[allow(unused_imports)]
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[allow(unused_imports)]
use super::*;

#[allow(unused_macros)]
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
    };
}

#[cfg(feature = "big_endian")]
mod be {
    use super::*;
    #[cfg(feature = "byte_impls")]
    mod bytes {
        use super::*;
        add_math_ops!(BigEndian<u8>);
        add_math_ops!(BigEndian<i8>);
    }

    #[cfg(feature = "integer_impls")]
    mod integers {
        use super::*;
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
    }

    #[cfg(feature = "float_impls")]
    mod floats {
        use super::*;
        add_math_ops!(BigEndian<f32>);
        add_math_ops!(BigEndian<f64>);
    }
}

#[cfg(feature = "little_endian")]
mod le {
    use super::*;
    #[cfg(feature = "byte_impls")]
    mod bytes {
        use super::*;
        add_math_ops!(LittleEndian<u8>);
        add_math_ops!(LittleEndian<i8>);
    }

    #[cfg(feature = "integer_impls")]
    mod integers {
        use super::*;
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
    }

    #[cfg(feature = "float_impls")]
    mod floats {
        use super::*;
        add_math_ops!(LittleEndian<f32>);
        add_math_ops!(LittleEndian<f64>);
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use crate::*;

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
}

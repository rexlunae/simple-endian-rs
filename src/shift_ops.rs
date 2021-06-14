//! Bitshift operations, for integer types only.

#[allow(unused_imports)]
use core::ops::{Shl, ShlAssign, Shr, ShrAssign};

#[allow(unused_imports)]
use super::*;

#[allow(unused_macros)]
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
    };
}

#[cfg(feature = "big_endian")]
mod be {
    use super::*;
    #[cfg(feature = "byte_impls")]
    mod bytes {
        use super::*;
        add_shift_ops!(BigEndian<u8>);
        add_shift_ops!(BigEndian<i8>);
    }

    #[cfg(feature = "integer_impls")]
    mod integers {
        use super::*;
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
    }
}

#[cfg(feature = "big_endian")]
mod le {
    use super::*;
    #[cfg(feature = "byte_impls")]
    mod bytes {
        use super::*;
        add_shift_ops!(LittleEndian<u8>);
        add_shift_ops!(LittleEndian<i8>);
    }

    #[cfg(feature = "integer_impls")]
    mod integers {
        use super::*;
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
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use crate::*;

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
}

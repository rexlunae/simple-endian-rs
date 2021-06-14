//! Comparison ops.
#[allow(unused_imports)]
use core::cmp::Ordering;

#[allow(unused_imports)]
use super::*;

/// For types that have PartialEq.
#[allow(unused_macros)]
macro_rules! add_equality_ops {
    ($wrap_ty:ty) => {
        impl PartialOrd for $wrap_ty {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.to_native().partial_cmp(&other.to_native())
            }
        }
    };
}

// The floats can only have PartialOrd, not Ord, because they only have PartialEq and not Eq.
#[cfg(feature = "float_impls")]
mod float_comps {
    use super::*;
    #[cfg(feature = "big_endian")]
    mod be {
        use super::*;
        add_equality_ops!(BigEndian<f32>);
        add_equality_ops!(BigEndian<f64>);
    }
    #[cfg(feature = "little_endian")]
    mod le {
        use super::*;
        add_equality_ops!(LittleEndian<f32>);
        add_equality_ops!(LittleEndian<f64>);
    }
}

/// For types that implement Eq.
#[allow(unused_macros)]
macro_rules! add_full_equality_ops {
    ($wrap_ty:ty) => {
        impl Ord for $wrap_ty {
            fn cmp(&self, other: &Self) -> Ordering {
                self.to_native().cmp(&other.to_native())
            }
        }
        add_equality_ops!($wrap_ty);
    };
}

// We have to separate Ord because f32/64 don't have Eq trait.
#[cfg(feature = "integer_impls")]
mod integer_comps {
    use super::*;
    #[cfg(feature = "big_endian")]
    mod be {
        use super::*;
        add_full_equality_ops!(BigEndian<u16>);
        add_full_equality_ops!(BigEndian<i16>);
        add_full_equality_ops!(BigEndian<u32>);
        add_full_equality_ops!(BigEndian<i32>);
        add_full_equality_ops!(BigEndian<u64>);
        add_full_equality_ops!(BigEndian<i64>);
        add_full_equality_ops!(BigEndian<u128>);
        add_full_equality_ops!(BigEndian<i128>);
        add_full_equality_ops!(BigEndian<usize>);
        add_full_equality_ops!(BigEndian<isize>);
    }
    #[cfg(feature = "little_endian")]
    mod le {
        use super::*;
        add_full_equality_ops!(LittleEndian<u16>);
        add_full_equality_ops!(LittleEndian<i16>);
        add_full_equality_ops!(LittleEndian<u32>);
        add_full_equality_ops!(LittleEndian<i32>);
        add_full_equality_ops!(LittleEndian<u64>);
        add_full_equality_ops!(LittleEndian<i64>);
        add_full_equality_ops!(LittleEndian<u128>);
        add_full_equality_ops!(LittleEndian<i128>);
        add_full_equality_ops!(LittleEndian<usize>);
        add_full_equality_ops!(LittleEndian<isize>);
    }
}

#[cfg(feature = "byte_impls")]
mod byte_comps {
    use super::*;
    #[cfg(feature = "big_endian")]
    mod be {
        use super::*;
        add_full_equality_ops!(BigEndian<bool>);
        add_full_equality_ops!(BigEndian<u8>);
        add_full_equality_ops!(BigEndian<i8>);
    }
    #[cfg(feature = "little_endian")]
    mod le {
        use super::*;
        add_full_equality_ops!(LittleEndian<bool>);
        add_full_equality_ops!(LittleEndian<u8>);
        add_full_equality_ops!(LittleEndian<i8>);
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use crate::*;

    #[test]
    fn equality_test() {
        let be1 = BigEndian::from(12345);
        let be2 = BigEndian::from(12345);
        assert_eq!(true, be1 == be2);
    }

    #[test]
    fn not_equality_test() {
        let be1 = BigEndian::from(12345);
        let be2 = BigEndian::from(34565);
        assert_eq!(true, be1 != be2);
    }

    #[test]
    fn lt_test() {
        let be1 = BigEndian::from(12345);
        let be2 = BigEndian::from(34565);
        assert_eq!(true, be1 < be2);
    }

    #[test]
    fn gt_test() {
        let be1 = BigEndian::from(34565);
        let be2 = BigEndian::from(12345);
        assert_eq!(true, be1 > be2);
    }

    #[test]
    fn lt_fp_be() {
        let be1 = BigEndian::from(1234.5678);
        let be2 = BigEndian::from(6234.5678);
        assert_eq!(true, be1 < be2);
    }
}

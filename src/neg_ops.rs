//! Module adding negation to the types where it's possible.
#[allow(unused_imports)]
use std::ops::Neg;

#[allow(unused_imports)]
use super::*;

#[allow(unused_macros)]
macro_rules! add_neg_ops {
    ($wrap_ty:ty) => {
        impl Neg for $wrap_ty {
            type Output = Self;

            fn neg(self) -> Self {
                Self::from(-self.to_native())
            }
        }
    }
}

#[cfg(feature = "big_endian")]
mod be {
    #[cfg(feature = "byte_impls")] add_neg_ops!(BigEndian<i8>);

    #[cfg(feature = "integer_impls")]
    mod integers {
        add_neg_ops!(BigEndian<i16>);
        add_neg_ops!(BigEndian<i32>);
        add_neg_ops!(BigEndian<i64>);
        add_neg_ops!(BigEndian<i128>);
        add_neg_ops!(BigEndian<isize>);
    }

    #[cfg(feature = "float_impls")]
    mod floats {
        add_neg_ops!(BigEndian<f32>);
        add_neg_ops!(BigEndian<f64>);
    }
}


#[cfg(feature = "little_endian")]
mod le {
    #[cfg(feature = "byte_impls")] add_neg_ops!(LittleEndian<i8>);

    #[cfg(feature = "integer_impls")]
    mod integers {
        add_neg_ops!(LittleEndian<i16>);
        add_neg_ops!(LittleEndian<i32>);
        add_neg_ops!(LittleEndian<i64>);
        add_neg_ops!(LittleEndian<i128>);
        add_neg_ops!(LittleEndian<isize>);
    }

    #[cfg(feature = "float_impls")]
    mod floats {
        add_neg_ops!(LittleEndian<f32>);
        add_neg_ops!(LittleEndian<f64>);
    }
}


#[cfg(test)]
mod tests {
    extern crate test;
    use crate::*;
    #[test]
    fn negate() {
        let be1 = BigEndian::from(1);
        let be2 = -be1;
        println!("{}, {}", be1, be2);
        assert_eq!(be2, i32be::from(-1));
    }
    #[test]
    fn negate_fp() {
        let be1 = BigEndian::from(1.0);
        let be2 = -be1;
        println!("{}, {}", be1, be2);
        assert_eq!(be2, f64be::from(-1.0));
    }

}
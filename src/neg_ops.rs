//! Module adding negation to the types where it's possible.
use super::*;
use core::ops::Neg;

macro_rules! add_neg_ops {
    ($wrap_ty:ident) => {
        impl<T> Neg for $wrap_ty<T>
        where
            T: Neg<Output = T> + SpecificEndian<T>,
        {
            type Output = Self;

            fn neg(self) -> Self {
                Self::from(-self.to_native())
            }
        }
    };
}

add_neg_ops!(LittleEndian);
add_neg_ops!(BigEndian);

#[cfg(test)]
mod tests {
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

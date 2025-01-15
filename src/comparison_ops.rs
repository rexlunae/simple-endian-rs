//! Comparison ops.
#[allow(unused_imports)]
use core::cmp::Ordering;

#[allow(unused_imports)]
use super::*;

impl<T> PartialOrd for BigEndian<T>
where
    T: PartialOrd + SpecificEndian<T>,
{
    fn partial_cmp(&self, other: &BigEndian<T>) -> Option<Ordering> {
        self.to_native().partial_cmp(&other.to_native())
    }
}

impl<T> PartialOrd for LittleEndian<T>
where
    T: PartialOrd + SpecificEndian<T>,
{
    fn partial_cmp(&self, other: &LittleEndian<T>) -> Option<Ordering> {
        self.to_native().partial_cmp(&other.to_native())
    }
}

impl<T> Ord for BigEndian<T>
where
    T: Ord + SpecificEndian<T>,
{
    fn cmp(&self, other: &BigEndian<T>) -> Ordering {
        self.to_native().cmp(&other.to_native())
    }
}

impl<T> Ord for LittleEndian<T>
where
    T: Ord + SpecificEndian<T>,
{
    fn cmp(&self, other: &LittleEndian<T>) -> Ordering {
        self.to_native().cmp(&other.to_native())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn equality_test() {
        let be1 = BigEndian::from(12345);
        let be2 = BigEndian::from(12345);
        assert!(be1 == be2);
    }

    #[test]
    fn not_equality_test() {
        let be1 = BigEndian::from(12345);
        let be2 = BigEndian::from(34565);
        assert!(be1 != be2);
    }

    #[test]
    fn lt_test() {
        let be1 = BigEndian::from(12345);
        let be2 = BigEndian::from(34565);
        assert!(be1 < be2);
    }

    #[test]
    fn gt_test() {
        let be1 = BigEndian::from(34565);
        let be2 = BigEndian::from(12345);
        assert!(be1 > be2);
    }

    #[test]
    fn lt_fp_be() {
        let be1 = BigEndian::from(1234.5678);
        let be2 = BigEndian::from(6234.5678);
        assert!(be1 < be2);
    }
}

use std::cmp::Ordering;

use super::*;

macro_rules! add_equality_ops {
    ($wrap_ty:ty) => {
        impl PartialEq for $wrap_ty {
            fn eq(&self, other: &Self) -> bool {
                self._v == other._v
            }
        }
        impl Eq for $wrap_ty {}
        impl PartialOrd for $wrap_ty {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.to_native().partial_cmp(&other.to_native())
            }
        }
    
    }
}

add_equality_ops!(BigEndian<bool>);
add_equality_ops!(BigEndian<u8>);
add_equality_ops!(BigEndian<i8>);
add_equality_ops!(BigEndian<u16>);
add_equality_ops!(BigEndian<i16>);
add_equality_ops!(BigEndian<u32>);
add_equality_ops!(BigEndian<i32>);
add_equality_ops!(BigEndian<u64>);
add_equality_ops!(BigEndian<i64>);
add_equality_ops!(BigEndian<u128>);
add_equality_ops!(BigEndian<i128>);
add_equality_ops!(BigEndian<usize>);
add_equality_ops!(BigEndian<isize>);
add_equality_ops!(BigEndian<f32>);
add_equality_ops!(BigEndian<f64>);

add_equality_ops!(LittleEndian<bool>);
add_equality_ops!(LittleEndian<u8>);
add_equality_ops!(LittleEndian<i8>);
add_equality_ops!(LittleEndian<u16>);
add_equality_ops!(LittleEndian<i16>);
add_equality_ops!(LittleEndian<u32>);
add_equality_ops!(LittleEndian<i32>);
add_equality_ops!(LittleEndian<u64>);
add_equality_ops!(LittleEndian<i64>);
add_equality_ops!(LittleEndian<u128>);
add_equality_ops!(LittleEndian<i128>);
add_equality_ops!(LittleEndian<usize>);
add_equality_ops!(LittleEndian<isize>);
add_equality_ops!(LittleEndian<f32>);
add_equality_ops!(LittleEndian<f64>);

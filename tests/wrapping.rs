#![cfg(all(feature = "integer_impls", feature = "wrapping"))]

use core::num::Wrapping;

use simple_endian::{BigEndian, LittleEndian, SpecificEndian};

#[test]
fn wrapping_works_with_endian_wrappers() {
    let v = Wrapping(0x1234_5678u32);

    let be_v: Wrapping<u32> = SpecificEndian::to_big_endian(&v);
    let le_v: Wrapping<u32> = SpecificEndian::to_little_endian(&v);

    let round_be: Wrapping<u32> = SpecificEndian::from_big_endian(&be_v);
    let round_le: Wrapping<u32> = SpecificEndian::from_little_endian(&le_v);
    assert_eq!(round_be, v);
    assert_eq!(round_le, v);

    // Also ensure the endian wrapper types accept Wrapping<T>.
    let _be: BigEndian<Wrapping<u32>> = v.into();
    let _le: LittleEndian<Wrapping<u32>> = v.into();
}

#![cfg(all(feature = "integer_impls", feature = "nonzero"))]

use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroIsize,
    NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroUsize,
};

use simple_endian::SpecificEndian;

#[test]
fn nonzero_u128_roundtrip_big_little() {
    let v = NonZeroU128::new(0x0123_4567_89ab_cdef_fedc_ba98_7654_3211u128).unwrap();

    let be: NonZeroU128 = SpecificEndian::to_big_endian(&v);
    let round: NonZeroU128 = SpecificEndian::from_big_endian(&be);
    assert_eq!(round, v);

    let le: NonZeroU128 = SpecificEndian::to_little_endian(&v);
    let round: NonZeroU128 = SpecificEndian::from_little_endian(&le);
    assert_eq!(round, v);
}

#[test]
fn nonzero_i128_roundtrip_big_little() {
    let v = NonZeroI128::new(-0x0123_4567_89ab_cdef_fedc_ba98_7654_321i128).unwrap();

    let be: NonZeroI128 = SpecificEndian::to_big_endian(&v);
    let round: NonZeroI128 = SpecificEndian::from_big_endian(&be);
    assert_eq!(round, v);

    let le: NonZeroI128 = SpecificEndian::to_little_endian(&v);
    let round: NonZeroI128 = SpecificEndian::from_little_endian(&le);
    assert_eq!(round, v);
}

#[test]
fn nonzero_misc_sizes_compile_and_roundtrip() {
    // This test exists mainly to ensure the impls exist across widths.
    let u16v = NonZeroU16::new(0x1234).unwrap();
    assert_eq!(SpecificEndian::from_big_endian(&SpecificEndian::to_big_endian(&u16v)), u16v);

    let i16v = NonZeroI16::new(-2).unwrap();
    assert_eq!(SpecificEndian::from_little_endian(&SpecificEndian::to_little_endian(&i16v)), i16v);

    let u32v = NonZeroU32::new(0x1234_5678).unwrap();
    assert_eq!(SpecificEndian::from_big_endian(&SpecificEndian::to_big_endian(&u32v)), u32v);

    let i32v = NonZeroI32::new(-0x1234_567).unwrap();
    assert_eq!(SpecificEndian::from_little_endian(&SpecificEndian::to_little_endian(&i32v)), i32v);

    let u64v = NonZeroU64::new(0x0123_4567_89ab_cdef).unwrap();
    assert_eq!(SpecificEndian::from_big_endian(&SpecificEndian::to_big_endian(&u64v)), u64v);

    let i64v = NonZeroI64::new(-0x0123_4567_89ab_cdei64).unwrap();
    assert_eq!(SpecificEndian::from_little_endian(&SpecificEndian::to_little_endian(&i64v)), i64v);

    let usz = NonZeroUsize::new(1).unwrap();
    assert_eq!(SpecificEndian::from_big_endian(&SpecificEndian::to_big_endian(&usz)), usz);

    let isz = NonZeroIsize::new(-1).unwrap();
    assert_eq!(SpecificEndian::from_little_endian(&SpecificEndian::to_little_endian(&isz)), isz);
}

#[test]
fn nonzero_shorthand_types_exist() {
    // Just compile-check a few aliases.
    let _a: simple_endian::nzu32be = NonZeroU32::new(1).unwrap().into();
    let _b: simple_endian::nzi64le = NonZeroI64::new(-1).unwrap().into();
    let _c: simple_endian::nzu128le = NonZeroU128::new(1).unwrap().into();
}

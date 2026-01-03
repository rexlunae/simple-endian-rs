//! Integration tests for the `simple_specific_endian_bridge` feature.
//!
//! These tests validate that when the bridge feature is enabled, types that are
//! `SimpleEndian + Copy` (currently bool/u8/i8) also implement `SpecificEndian` and
//! that endian conversions are no-ops.

use simple_endian::{SimpleEndian, SpecificEndian};

#[test]
fn bridge_bool_noop() {
    // Compile-time assertion: bool implements SpecificEndian<bool>
    fn assert_specific_endian<T: SpecificEndian<T>>() {}
    assert_specific_endian::<bool>();

    let v = true;
    assert_eq!(<bool as SpecificEndian<bool>>::to_big_endian(&v), v);
    assert_eq!(<bool as SpecificEndian<bool>>::to_little_endian(&v), v);
    assert_eq!(<bool as SpecificEndian<bool>>::from_big_endian(&v), v);
    assert_eq!(<bool as SpecificEndian<bool>>::from_little_endian(&v), v);

    // Cross-check against SimpleEndian behavior.
    assert_eq!(v.to_big_endian(), v);
    assert_eq!(v.from_little_endian(), v);
}

#[test]
fn bridge_u8_noop() {
    fn assert_specific_endian<T: SpecificEndian<T>>() {}
    assert_specific_endian::<u8>();

    let v: u8 = 0xfe;
    assert_eq!(<u8 as SpecificEndian<u8>>::to_big_endian(&v), v);
    assert_eq!(<u8 as SpecificEndian<u8>>::to_little_endian(&v), v);
    assert_eq!(<u8 as SpecificEndian<u8>>::from_big_endian(&v), v);
    assert_eq!(<u8 as SpecificEndian<u8>>::from_little_endian(&v), v);

    assert_eq!(v.to_big_endian(), v);
    assert_eq!(v.from_little_endian(), v);
}

#[test]
fn bridge_i8_noop() {
    fn assert_specific_endian<T: SpecificEndian<T>>() {}
    assert_specific_endian::<i8>();

    let v: i8 = -42;
    assert_eq!(<i8 as SpecificEndian<i8>>::to_big_endian(&v), v);
    assert_eq!(<i8 as SpecificEndian<i8>>::to_little_endian(&v), v);
    assert_eq!(<i8 as SpecificEndian<i8>>::from_big_endian(&v), v);
    assert_eq!(<i8 as SpecificEndian<i8>>::from_little_endian(&v), v);
    assert_eq!(v.to_big_endian(), v);
    assert_eq!(v.from_little_endian(), v);
}

#[test]
#[cfg(all(
    feature = "simple_specific_endian_bridge",
    feature = "simple_char_impls"
))]
fn bridge_char_noop() {
    fn assert_specific_endian<T: SpecificEndian<T>>() {}
    assert_specific_endian::<char>();

    let v: char = '🦀';
    assert_eq!(<char as SpecificEndian<char>>::to_big_endian(&v), v);
    assert_eq!(<char as SpecificEndian<char>>::to_little_endian(&v), v);
    assert_eq!(<char as SpecificEndian<char>>::from_big_endian(&v), v);
    assert_eq!(<char as SpecificEndian<char>>::from_little_endian(&v), v);

    assert_eq!(v.to_big_endian(), v);
    assert_eq!(v.from_little_endian(), v);
}

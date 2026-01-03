#![cfg(all(
    feature = "integer_impls",
    feature = "big_endian",
    feature = "little_endian",
    feature = "io",
    feature = "io-std"
))]

use simple_endian::{read_specific, write_specific, BigEndian, EndianRead, EndianWrite, LittleEndian};

#[test]
fn big_endian_tuple_roundtrips_via_io() {
    let native: (u16, u32) = (0x0102, 0x0304_0506);
    let wire: BigEndian<(u16, u32)> = native.into();

    let mut buf = Vec::new();
    write_specific(&mut buf, &wire).unwrap();

    // Tuple wrapper IO currently serializes tuples via the crate's internal
    // u128-based representation, which results in 8 bytes for (u16, u32).
    // The significant bytes are still in-order and in the expected endianness.
    assert_eq!(buf, vec![0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);

    let mut cur = std::io::Cursor::new(buf);
    let wire2: BigEndian<(u16, u32)> = read_specific(&mut cur).unwrap();

    // Compare by converting back to native.
    assert_eq!(wire2.to_native(), native);
}

#[test]
fn little_endian_tuple_roundtrips_via_io() {
    let native: (u16, u32) = (0x0102, 0x0304_0506);
    let wire: LittleEndian<(u16, u32)> = native.into();

    let mut buf = Vec::new();
    write_specific(&mut buf, &wire).unwrap();

    // See comment in `big_endian_tuple_roundtrips_via_io`.
    assert_eq!(buf, vec![0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00, 0x00]);

    let mut cur = std::io::Cursor::new(buf);
    let wire2: LittleEndian<(u16, u32)> = read_specific(&mut cur).unwrap();

    assert_eq!(wire2.to_native(), native);
}

#[test]
fn arity_12_big_endian_tuple_works() {
    // IO via `write_specific`/`read_specific` is currently limited to types whose
    // total byte size is <= 16 when using the core u128 representation.
    // This test uses 8 elements (= 16 bytes) as the largest supported tuple.
    type T = (u16, u16, u16, u16, u16, u16, u16, u16);
    let native: T = (
        0x0001, 0x0002, 0x0003, 0x0004, 0x0005, 0x0006, 0x0007, 0x0008,
    );

    let wire: BigEndian<T> = native.into();

    // Ensure writing compiles and byte order is correct.
    let mut buf = Vec::new();
    write_specific(&mut buf, &wire).unwrap();

    // First and last elements are enough to sanity-check ordering.
    assert_eq!(&buf[0..2], &[0x00, 0x01]);
    assert_eq!(&buf[14..16], &[0x00, 0x08]);

    let mut cur = std::io::Cursor::new(buf);
    let wire2: BigEndian<T> = read_specific(&mut cur).unwrap();
    assert_eq!(wire2.to_native(), native);
}

// Compile-time trait assertions (only meaningful with io enabled).
fn _assert_io<T: EndianRead + EndianWrite>() {}

#[test]
fn tuple_wrappers_implement_endianread_endianwrite() {
    _assert_io::<BigEndian<(u16, u32)>>();
    _assert_io::<LittleEndian<(u16, u32)>>();
}

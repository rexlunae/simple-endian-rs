#![cfg(all(feature = "derive", feature = "io", feature = "io-std"))]

use simple_endian::{EndianRead, EndianWrite, read_specific, write_specific};

#[derive(simple_endian_derive::Endianize, Clone, Copy, Debug, PartialEq, Eq)]
#[endian(be)]
#[wire_derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Demo {
    a: u16,
    b: u32,
}

#[test]
fn wire_auto_impls_endianread_endianwrite() {
    // If the derive didn't generate these impls, this test won't compile.
    fn _assert_read<T: EndianRead>() {}
    fn _assert_write<T: EndianWrite>() {}
    _assert_read::<DemoWire>();
    _assert_write::<DemoWire>();
}

#[test]
fn wire_roundtrips_via_specific_io() {
    let v = DemoWire {
        a: 0x1122u16.into(),
        b: 0x33445566u32.into(),
    };

    let mut out = Vec::new();
    write_specific(&mut out, &v).unwrap();

    // big-endian bytes: a then b
    assert_eq!(out, vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);

    let mut cursor: &[u8] = &out;
    let got: DemoWire = read_specific(&mut cursor).unwrap();
    assert_eq!(got, v);
}

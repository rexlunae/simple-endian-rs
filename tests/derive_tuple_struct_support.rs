#![cfg(all(feature = "derive", feature = "io", feature = "io-std"))]

use simple_endian::{read_specific, write_specific, EndianRead, EndianWrite, Endianize};

#[derive(Endianize, Clone, Copy, Debug, PartialEq, Eq)]
#[endian(be)]
#[wire_derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TupleDemo(u16, u32);

#[test]
fn tuple_wire_has_io_and_roundtrips() {
    // compile-time assertions
    fn _assert_io<T: EndianRead + EndianWrite>() {}
    _assert_io::<TupleDemoWire>();

    let v = TupleDemo(0x0102, 0x0304_0506);
    let wire: TupleDemoWire = v.into();

    let mut buf = Vec::new();
    write_specific(&mut buf, &wire).unwrap();

    // Big-endian u16 then u32
    assert_eq!(buf, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);

    let mut cursor = std::io::Cursor::new(buf);
    let wire2: TupleDemoWire = read_specific(&mut cursor).unwrap();
    assert_eq!(wire2, wire);

    let v2: TupleDemo = wire2.into();
    assert_eq!(v2, v);
}

#[test]
fn tuple_packed_wire_writes_safely() {
    #[derive(Endianize, Clone, Copy, Debug, PartialEq, Eq)]
    #[endian(be)]
    #[wire_repr(packed)]
    #[wire_derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct PackedTuple(u16, u32);

    // Just ensure this compiles and executes without E0793 (unaligned reference to packed field).
    let wire: PackedTupleWire = PackedTuple(0x1122, 0x3344_5566).into();

    let mut buf = Vec::new();
    write_specific(&mut buf, &wire).unwrap();
    assert_eq!(buf, vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66]);
}

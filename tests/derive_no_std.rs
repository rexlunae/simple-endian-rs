// Derived wire types must compile and convert without `io-std` (no `std::io`).
#![cfg(all(feature = "derive", not(feature = "io-std")))]

use simple_endian::Endianize;

#[derive(Endianize, Clone, Copy)]
#[endian(le)]
#[repr(C, packed)]
#[wire_repr(C, packed)]
#[wire_derive(Clone, Copy)]
struct Hdr {
    a: u8,
    b: u16,
    c: u32,
}

#[test]
fn derive_without_io_std() {
    let w = HdrWire::from(Hdr { a: 1, b: 0x0203, c: 0x0405_0607 });
    assert_eq!(core::mem::size_of::<HdrWire>(), 7);
    let n = Hdr::from(w);
    let (a, b, c) = (n.a, n.b, n.c);
    assert_eq!((a, b, c), (1, 0x0203, 0x0405_0607));
}

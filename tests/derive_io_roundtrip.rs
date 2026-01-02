#![cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]

use simple_endian::Endianize;
use simple_endian::{read_specific, write_specific};
use std::io::Cursor;

#[test]
fn derived_wire_struct_round_trips_via_io() {
    #[derive(Endianize)]
    #[endian(be)]
    #[repr(C)]
    #[allow(dead_code)]
    struct Packet {
        id: u32,

        #[text(utf16, units = 8, pad = "space")]
        title: String,
    }

    // Build a wire instance.
    let pkt = PacketWire {
        id: 0x1234_5678u32.into(),
        title: "HI".try_into().unwrap(),
    };

    // Write it using the io helpers.
    let mut buf = Vec::new();
    write_specific(&mut buf, &pkt).unwrap();

    // Read it back.
    let mut cur = Cursor::new(buf);
    let out: PacketWire = read_specific(&mut cur).unwrap();

    assert_eq!(out.id.to_native(), pkt.id.to_native());

    // For fixed text, assert semantic value, not internal wrapper bits.
    // (The wire encoding is standardized to UTF-16BE bytes; the wrapper's stored
    // representation is an internal detail.)
    let title = String::try_from(&out.title).unwrap();
    assert_eq!(title, "HI");
}

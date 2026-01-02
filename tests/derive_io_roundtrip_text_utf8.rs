#![cfg(all(feature = "derive", feature = "io-std", feature = "text_fixed", feature = "text_utf8"))]

use simple_endian::{read_specific, write_specific, Endianize};
use std::io::Cursor;

#[test]
fn derive_io_roundtrip_fixed_utf8_null_padded() {
    #[derive(Endianize, Debug)]
    #[endian(be)]
    #[repr(C)]
    #[allow(dead_code)]
    struct Packet {
        id: u32,

        #[text(utf8, units = 8, pad = "null")]
        name: String,
    }

    // Build a wire value directly (as intended usage).
    let wire = PacketWire {
        id: 0x11223344u32.into(),
        name: "hi".try_into().unwrap(),
    };

    // Serialize.
    let mut buf = Vec::new();
    write_specific(&mut buf, &wire).unwrap();

    // Check the raw bytes (wire format): u32be + 8 bytes UTF-8.
    assert_eq!(&buf[..4], &0x11223344u32.to_be_bytes());
    assert_eq!(&buf[4..6], b"hi");
    assert_eq!(&buf[6..12], &[0u8; 6]);

    // Deserialize and decode.
    let mut cur = Cursor::new(buf);
    let decoded: PacketWire = read_specific(&mut cur).unwrap();
    let name = String::try_from(&decoded.name).unwrap();
    assert_eq!(name, "hi");
}

#[test]
fn derive_io_roundtrip_fixed_utf8_space_padded_trims_only_trailing_spaces() {
    #[derive(Endianize, Debug)]
    #[endian(le)]
    #[repr(C)]
    #[allow(dead_code)]
    struct Packet {
        id: u16,

        #[text(utf8, units = 8, pad = "space")]
        name: String,
    }

    // Include an internal space; it must be preserved. Only trailing spaces are trimmed.
    let wire = PacketWire {
        id: 0x1234u16.into(),
        name: "a b".try_into().unwrap(),
    };

    let mut buf = Vec::new();
    write_specific(&mut buf, &wire).unwrap();

    // u16le + 8 bytes UTF-8
    assert_eq!(&buf[..2], &0x1234u16.to_le_bytes());
    assert_eq!(&buf[2..5], b"a b");
    assert_eq!(&buf[5..10], &[b' '; 5]);

    let mut cur = Cursor::new(buf);
    let decoded: PacketWire = read_specific(&mut cur).unwrap();
    let name = String::try_from(&decoded.name).unwrap();
    assert_eq!(name, "a b");
}

#[test]
fn derive_io_decode_utf8_invalid_bytes_errors() {
    #[derive(Endianize, Debug)]
    #[endian(be)]
    #[repr(C)]
    #[allow(dead_code)]
    struct Packet {
        id: u32,

        #[text(utf8, units = 4, pad = "null")]
        name: String,
    }

    // Build an on-wire buffer directly: u32be + 4 bytes.
    // 0xFF is never valid UTF-8.
    let mut buf = Vec::new();
    buf.extend_from_slice(&0x01020304u32.to_be_bytes());
    buf.extend_from_slice(&[0xFF, 0x00, 0x00, 0x00]);

    let mut cur = Cursor::new(buf);
    let decoded: PacketWire = read_specific(&mut cur).unwrap();
    let err = String::try_from(&decoded.name).unwrap_err();
    assert_eq!(err, simple_endian::FixedUtf8Error::InvalidUtf8);
}

#![cfg(all(feature = "derive", feature = "text_fixed", feature = "text_utf32", feature = "io"))]

use simple_endian::{read_specific, write_specific, Endianize, FixedUtf32BeSpacePadded};

#[test]
fn endianize_text_padding_generates_fixed_utf32_types() {
    #[derive(Endianize)]
    #[endian(be)]
    #[repr(C)]
    #[allow(dead_code)]
    struct Packet {
        id: u32,

        #[text(utf32, units = 8, pad = "space")]
        title: String,
    }

    // The generated wire type should use the fixed, padded UTF32BE type.
    let _wire = PacketWire {
        id: 1u32.into(),
        title: "HI".try_into().unwrap(),
    };
}

#[test]
fn fixed_utf32_space_padded_io_roundtrip() {
    #[derive(Endianize, Clone, Debug, PartialEq, Eq)]
    #[endian(be)]
    #[repr(C)]
    #[allow(dead_code)]
    struct Packet {
        id: u32,

        #[text(utf32, units = 8, pad = "space")]
        title: String,
    }

    let pkt = PacketWire {
        id: 0x11223344u32.into(),
        title: "welcome!".try_into().unwrap(),
    };

    // Encode as bytes ("sender" side).
    let mut buf = Vec::<u8>::new();
    write_specific(&mut buf, &pkt).unwrap();

    // Decode from bytes ("receiver" side).
    let mut cur = std::io::Cursor::new(buf);
    let got: PacketWire = read_specific(&mut cur).unwrap();

    // Decode the title field explicitly via the fixed type.
    let title = String::try_from(&got.title as &FixedUtf32BeSpacePadded<8>).unwrap();
    assert_eq!(title, "welcome!");
    assert_eq!(got.id.to_native(), 0x11223344);
}

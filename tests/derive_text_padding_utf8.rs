#![cfg(all(feature = "derive", feature = "text_fixed", feature = "text_utf8"))]

use simple_endian::Endianize;

#[test]
fn endianize_text_padding_generates_fixed_utf8_types() {
    #[derive(Endianize)]
    #[endian(be)]
    #[repr(C)]
    #[allow(dead_code)]
    struct PacketBe {
        id: u32,

        // Note: UTF-8 is endian-independent; container endian shouldn't matter.
        #[text(utf8, units = 8, pad = "null")]
        name: String,
    }

    #[derive(Endianize)]
    #[endian(le)]
    #[repr(C)]
    #[allow(dead_code)]
    struct PacketLe {
        id: u32,

        #[text(utf8, units = 8, pad = "space")]
        name: String,
    }

    // The generated wire type should use the fixed, padded UTF8 types.
    let _wire_be = PacketBeWire {
        id: 1u32.into(),
        name: "HI".try_into().unwrap(),
    };

    let _wire_le = PacketLeWire {
        id: 2u32.into(),
        name: "HI".try_into().unwrap(),
    };
}

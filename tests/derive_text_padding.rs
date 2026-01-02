#![cfg(all(feature = "derive", feature = "text_fixed", feature = "text_utf16"))]

use simple_endian::Endianize;

#[test]
fn endianize_text_padding_generates_fixed_types() {
    #[derive(Endianize)]
    #[endian(be)]
    #[repr(C)]
    #[allow(dead_code)]
    struct Packet {
        id: u32,

        #[text(utf16, units = 8, pad = "space")]
        title: String,
    }

    // The generated wire type should use the fixed, padded UTF16BE type.
    let _wire = PacketWire {
        id: 1u32.into(),
        title: "HI".try_into().unwrap(),
    };
}

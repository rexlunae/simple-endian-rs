#![cfg(all(feature = "derive", feature = "io-std", feature = "text_fixed", feature = "text_utf8"))]

use simple_endian::{EndianRead, EndianWrite, Endianize, FixedUtf8NullPadded, read_specific, write_specific};

#[test]
fn derived_enum_tuple_variant_supports_tuple_text_utf8() {
    #[derive(Endianize, Debug, PartialEq)]
    #[endian(be)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum Msg {
        // Tuple variant: field 0 is fixed utf8.
        #[tuple_text(idx = 0, utf8, units = 8, pad = "null")]
        Name(String, u16) = 1,
    }

    let msg = MsgWire {
        tag: 1u8.into(),
        payload: MsgWirePayload {
            Name: std::mem::ManuallyDrop::new(MsgWirePayload_Name(
                FixedUtf8NullPadded::<8>::try_from("abc").unwrap(),
                0x1234u16.into(),
            )),
        },
    };

    let mut buf = Vec::new();
    write_specific(&mut buf, &msg).unwrap();

    let mut cursor = std::io::Cursor::new(buf);
    let msg2: MsgWire = read_specific(&mut cursor).unwrap();
    assert_eq!(msg2.tag, msg.tag);

    // SAFETY: Tag chooses active union field.
    let p2 = unsafe { &msg2.payload.Name };
    assert_eq!(&p2.0, &FixedUtf8NullPadded::<8>::try_from("abc").unwrap());
    assert_eq!(p2.1, 0x1234u16.into());

    // Compile-time: ensure traits exist.
    fn _assert_traits<T: EndianRead + EndianWrite>() {}
    _assert_traits::<MsgWire>();
}

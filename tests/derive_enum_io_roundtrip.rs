#![cfg(all(feature = "derive", feature = "io-std"))]

use simple_endian::{Endianize, EndianRead, EndianWrite, read_specific, write_specific};

#[test]
fn derived_enum_wire_round_trips_via_io() {
    #[derive(Endianize, Debug, PartialEq)]
    #[endian(be)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum Msg {
        Ping = 1,
        Data { x: u16, y: u32 } = 2,
    }

    // Ping (unit) variant
    let ping = MsgWire {
        tag: 1u8.into(),
        payload: MsgWirePayload { _unused: [] },
    };

    let mut buf = Vec::new();
    write_specific(&mut buf, &ping).unwrap();

    let mut cursor = std::io::Cursor::new(buf);
    let ping2: MsgWire = read_specific(&mut cursor).unwrap();
    assert_eq!(ping2.tag, ping.tag);

    // Data variant
    let data = MsgWire {
        tag: 2u8.into(),
        payload: MsgWirePayload {
            Data: std::mem::ManuallyDrop::new(MsgWirePayload_Data {
                x: 0x1234u16.into(),
                y: 0xDEADBEEFu32.into(),
            }),
        },
    };

    let mut buf = Vec::new();
    write_specific(&mut buf, &data).unwrap();

    let mut cursor = std::io::Cursor::new(buf);
    let data2: MsgWire = read_specific(&mut cursor).unwrap();
    assert_eq!(data2.tag, data.tag);

    // SAFETY: Tag chooses active union field.
    let p2 = unsafe { &data2.payload.Data };
    assert_eq!(p2.x, 0x1234u16.into());
    assert_eq!(p2.y, 0xDEADBEEFu32.into());

    // Ensure the traits exist (compile-time coverage).
    fn _assert_traits<T: EndianRead + EndianWrite>() {}
    _assert_traits::<MsgWire>();
}

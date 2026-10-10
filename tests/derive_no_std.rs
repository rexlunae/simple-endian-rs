// Derived wire types must compile and convert without `io-std` (no `std::io`).
#![cfg(all(feature = "derive", not(feature = "io-std")))]
// A true no_std crate: any `::std` path left in generated code fails to resolve.
#![no_std]

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
    let w = HdrWire::from(Hdr {
        a: 1,
        b: 0x0203,
        c: 0x0405_0607,
    });
    assert_eq!(core::mem::size_of::<HdrWire>(), 7);
    let n = Hdr::from(w);
    let (a, b, c) = (n.a, n.b, n.c);
    assert_eq!((a, b, c), (1, 0x0203, 0x0405_0607));
}

// Tuple-struct generation path (also had its IO impls gated).
#[derive(Endianize, Clone, Copy)]
#[endian(le)]
#[repr(C, packed)]
#[wire_repr(C, packed)]
#[wire_derive(Clone, Copy)]
struct Pair(u16, u32);

#[test]
fn tuple_struct_without_io_std() {
    let w = PairWire::from(Pair(0x0102, 0x0304_0506));
    assert_eq!(core::mem::size_of::<PairWire>(), 6);
    let n = Pair::from(w);
    let (a, b) = (n.0, n.1);
    assert_eq!((a, b), (0x0102, 0x0304_0506));
}

// Enum generation path, including both payload forms (named and tuple fields),
// where `::std::mem::ManuallyDrop` was replaced with `::core::mem::ManuallyDrop`.
#[derive(Endianize, Clone, Copy)]
#[endian(le)]
#[repr(u8)]
enum Cmd {
    Ping = 1,
    Set { key: u16, value: u32 } = 2,
    Pair(u16, u32) = 3,
}

#[test]
fn enum_payloads_without_io_std() {
    use core::mem::ManuallyDrop;
    // Build wire values for both payload forms. Compiling the generated wire types
    // is the main check: it fails to resolve any `::std` path left in the enum
    // payload (`ManuallyDrop`) or in the gated IO code.
    let set = CmdWire {
        tag: 2u8.into(),
        payload: CmdWirePayload {
            Set: ManuallyDrop::new(CmdWirePayload_Set {
                key: 7u16.into(),
                value: 9u32.into(),
            }),
        },
    };
    let tag: u8 = set.tag.into();
    assert_eq!(tag, 2);
    let pair = CmdWire {
        tag: 3u8.into(),
        payload: CmdWirePayload {
            Pair: ManuallyDrop::new(CmdWirePayload_Pair(1u16.into(), 2u32.into())),
        },
    };
    let tag: u8 = pair.tag.into();
    assert_eq!(tag, 3);
}

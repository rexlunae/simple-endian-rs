//! Test: Wire struct naming edge cases
//!
//! This test documents the behavior when struct names already end in "Wire".
//! While not necessarily a bug, it demonstrates a potential source of confusion.

#![cfg(feature = "derive")]

use simple_endian::Endianize;

#[test]
fn struct_ending_in_wire_generates_double_wire_suffix() {
    // When a struct name already ends with "Wire", the generated type
    // will have a doubled suffix: "WireWire"
    #[derive(Endianize)]
    #[endian(be)]
    #[repr(C)]
    struct DataWire {
        value: u32,
    }

    // The generated type is named DataWireWire (not ideal, but functional)
    let wire = DataWireWire {
        value: 42u32.into(),
    };

    assert_eq!(wire.value.to_native(), 42);
}

#[test]
fn normal_naming_works_as_expected() {
    #[derive(Endianize)]
    #[endian(le)]
    #[repr(C)]
    struct Header {
        id: u16,
    }

    // Generated type is HeaderWire (clean and clear)
    let wire = HeaderWire {
        id: 123u16.into(),
    };

    assert_eq!(wire.id.to_native(), 123);
}

#[test]
fn multiple_word_names_work_correctly() {
    #[derive(Endianize)]
    #[endian(be)]
    #[repr(C)]
    struct PacketHeader {
        version: u8,
        length: u16,
    }

    // Generated type is PacketHeaderWire
    let wire = PacketHeaderWire {
        version: 1u8.into(),
        length: 100u16.into(),
    };

    assert_eq!(wire.version.to_native(), 1);
    assert_eq!(wire.length.to_native(), 100);
}

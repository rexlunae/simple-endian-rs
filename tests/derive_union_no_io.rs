#![cfg(feature = "derive")]

use simple_endian::Endianize;

#[test]
fn endianize_generates_union_wire_type() {
    #[derive(Endianize)]
    #[endian(le)]
    #[allow(dead_code)]
    union U {
        a: u32,
        b: u16,
    }

    // Ensure the generated wire union exists and is usable.
    // (We intentionally do not assert IO traits here; unions do not get EndianRead/EndianWrite.)
    let _ = UWire { a: 0u32.into() };
}

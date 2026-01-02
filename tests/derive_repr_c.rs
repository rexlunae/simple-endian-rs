#![cfg(feature = "derive")]

use simple_endian::Endianize;

// This test can't directly "assert repr(C)" (Rust has no stable reflection API for that),
// but we can sanity-check a few things that strongly indicate we're getting a stable C-like
// layout rather than the default repr(Rust):
//  - field order is preserved (offsets increase in declaration order)
//  - expected padding/alignment behavior matches C rules for simple primitives
//
// We avoid unstable `offset_of!` by using pointer arithmetic.

#[test]
fn generated_wire_struct_has_c_layout_sanity_checks() {
    #[derive(Endianize)]
    #[endian(be)]
    #[allow(dead_code)]
    struct Logical {
        a: u8,
        b: u32,
        c: u16,
    }

    // Offsets within the generated repr(C) struct should match C-like alignment rules.
    // Layout expectation on typical platforms:
    // - a @ 0
    // - padding 3 bytes
    // - b @ 4
    // - c @ 8
    // - trailing padding to align struct to 4 => total size 12
    let wire = LogicalWire {
        a: 0u8.into(),
        b: 0u32.into(),
        c: 0u16.into(),
    };

    let base = core::ptr::addr_of!(wire) as usize;
    let a_off = (core::ptr::addr_of!(wire.a) as usize) - base;
    let b_off = (core::ptr::addr_of!(wire.b) as usize) - base;
    let c_off = (core::ptr::addr_of!(wire.c) as usize) - base;

    assert_eq!(a_off, 0);
    assert_eq!(b_off, 4);
    assert_eq!(c_off, 8);

    assert_eq!(core::mem::align_of::<LogicalWire>(), 4);
    assert_eq!(core::mem::size_of::<LogicalWire>(), 12);
}

#![cfg(feature = "derive")]

use simple_endian::Endianize;

#[test]
fn wire_default_respects_field_default_attribute() {
    #[derive(Endianize, Debug)]
    #[endian(le)]
    #[wire_derive(Default)]
    struct Foo {
        x: u16,
        y: u16,
    }

    // With #[wire_derive(Default)], FooWire should be able to derive Default.
    let w = FooWire::default();

    assert_eq!(u16::from(w.x), 0);
    assert_eq!(u16::from(w.y), 0);
}

#[test]
fn wire_default_respects_enum_default_variant_attribute() {
    #[derive(Endianize, Debug)]
    #[endian(le)]
    #[repr(u8)]
    #[wire_default]
    #[allow(dead_code)]
    enum E {
        #[default]
        A = 1,
        B = 2,
    }

    let w = EWire::default();
    assert_eq!(u8::from(w.tag), 1);
}

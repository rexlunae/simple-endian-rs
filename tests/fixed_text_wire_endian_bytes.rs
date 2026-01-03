#![cfg(all(
    feature = "io",
    feature = "text_fixed",
    feature = "text_utf16",
    feature = "text_utf32"
))]

use simple_endian::{FixedUtf16BeSpacePadded, FixedUtf32BeSpacePadded, write_specific};

// NOTE: These tests assert the crate's *on-wire* encoding as produced by
// `FromSlice::write_to_extend` for the fixed text types.
//
// Fixed UTF types must serialize the *native scalar code units* in the declared
// target endianness (i.e., standard UTF-16BE / UTF-32BE on the wire).

#[test]
fn utf16_be_space_padded_is_be_on_wire_and_roundtrips() {
    // Include an interior space to ensure we don't “trim the middle”.
    let s = "A B";
    let fx: FixedUtf16BeSpacePadded<8> = s.try_into().unwrap();

    // Sender side: serialize in target endian.
    let mut buf = Vec::<u8>::new();
    write_specific(&mut buf, &fx).unwrap();
    assert_eq!(buf.len(), 8 * 2);

    // Receiver side: ensure decode yields the original text.
    let back = String::try_from(&fx).unwrap();
    assert_eq!(back, s);

    // Verify raw bytes match standard UTF-16BE encoding.
    // First 3 code units:
    // 'A' -> [0x00, 0x41], ' ' -> [0x00, 0x20], 'B' -> [0x00, 0x42]
    assert_eq!(&buf[0..2], &[0x00, 0x41]);
    assert_eq!(&buf[2..4], &[0x00, 0x20]);
    assert_eq!(&buf[4..6], &[0x00, 0x42]);

    // Verify padding is spaces.
    for chunk in buf[6..].chunks_exact(2) {
        assert_eq!(chunk, &[0x00, 0x20]);
    }
}

#[test]
fn utf32_be_space_padded_is_be_on_wire_and_roundtrips() {
    // Include an interior space to ensure we don't “trim the middle”.
    let s = "A B";
    let fx: FixedUtf32BeSpacePadded<8> = s.try_into().unwrap();

    // Sender side: serialize in target endian.
    let mut buf = Vec::<u8>::new();
    write_specific(&mut buf, &fx).unwrap();
    assert_eq!(buf.len(), 8 * 4);

    // Receiver side: ensure decode yields the original text.
    let back = String::try_from(&fx).unwrap();
    assert_eq!(back, s);

    // Verify raw bytes match standard UTF-32BE encoding.
    // First 3 code units:
    // 'A' -> [0x00, 0x00, 0x00, 0x41], ' ' -> [0x00, 0x00, 0x00, 0x20], 'B' -> [0x00, 0x00, 0x00, 0x42]
    assert_eq!(&buf[0..4], &[0x00, 0x00, 0x00, 0x41]);
    assert_eq!(&buf[4..8], &[0x00, 0x00, 0x00, 0x20]);
    assert_eq!(&buf[8..12], &[0x00, 0x00, 0x00, 0x42]);

    // Verify padding is spaces.
    for chunk in buf[12..].chunks_exact(4) {
        assert_eq!(chunk, &[0x00, 0x00, 0x00, 0x20]);
    }
}

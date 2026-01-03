#![cfg(all(feature = "text_fixed", feature = "text_utf8"))]

use simple_endian::{FixedUtf8NullPadded, FixedUtf8SpacePadded};

#[test]
fn utf8_null_padded_roundtrip() {
    let v = FixedUtf8NullPadded::<8>::try_from("hi").unwrap();

    let bytes = v.0.as_bytes();
    assert_eq!(&bytes[..2], b"hi");
    assert_eq!(&bytes[2..], &[0u8; 6]);

    let s = String::try_from(&v).unwrap();
    assert_eq!(s, "hi");
}

#[test]
fn utf8_space_padded_roundtrip() {
    let v = FixedUtf8SpacePadded::<8>::try_from("hi").unwrap();

    let bytes = v.0.as_bytes();
    assert_eq!(&bytes[..2], b"hi");
    assert_eq!(&bytes[2..], &[b' '; 6]);

    let s = String::try_from(&v).unwrap();
    assert_eq!(s, "hi");
}

#[test]
fn utf8_too_many_bytes_errors() {
    let err = FixedUtf8NullPadded::<2>::try_from("abc").unwrap_err();
    assert!(matches!(
        err,
        simple_endian::FixedUtf8Error::TooManyBytes { .. }
    ));
}

#[test]
fn utf8_invalid_utf8_errors_on_decode_null_padded() {
    // 0xFF is never valid UTF-8.
    let v = FixedUtf8NullPadded::<4>::from(simple_endian::FixedUtf8Bytes::from([0xFF, 0, 0, 0]));
    let err = String::try_from(&v).unwrap_err();
    assert_eq!(err, simple_endian::FixedUtf8Error::InvalidUtf8);
}

#[test]
fn utf8_invalid_utf8_errors_on_decode_space_padded() {
    // Use a single invalid byte followed by spaces; trim should remove only trailing spaces,
    // leaving the invalid byte to fail decoding.
    let v = FixedUtf8SpacePadded::<4>::from(simple_endian::FixedUtf8Bytes::from([
        0xFF, b' ', b' ', b' ',
    ]));
    let err = String::try_from(&v).unwrap_err();
    assert_eq!(err, simple_endian::FixedUtf8Error::InvalidUtf8);
}

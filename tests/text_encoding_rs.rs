#![cfg(feature = "text_encoding_rs")]

use encoding_rs::WINDOWS_1252;
use encoding_rs::UTF_16LE;

use simple_endian::encoding_rs::{
    decode_null_padded, decode_space_padded, encode_null_padded, encode_space_padded, EncodingRsError,
};
use simple_endian::{FixedUtf8Bytes, FixedUtf8NullPadded, FixedUtf8SpacePadded};

#[test]
fn windows_1252_round_trip_null_padded() {
    // "café" in windows-1252 is: 63 61 66 e9
    let v: FixedUtf8NullPadded<8> = FixedUtf8Bytes::from([b'c', b'a', b'f', 0xE9, 0, 0, 0, 0]).into();

    let s = decode_null_padded::<8>(WINDOWS_1252, &v).unwrap();
    assert_eq!(s, "café");

    let out = encode_null_padded::<8>(WINDOWS_1252, &s).unwrap();
    assert_eq!(out.0.as_bytes()[..4], [b'c', b'a', b'f', 0xE9]);
    assert_eq!(out.0.as_bytes()[4..], [0, 0, 0, 0]);
}

#[test]
fn windows_1252_round_trip_space_padded() {
    let v: FixedUtf8SpacePadded<8> =
        FixedUtf8Bytes::from([b'c', b'a', b'f', 0xE9, b' ', b' ', b' ', b' ']).into();

    let s = decode_space_padded::<8>(WINDOWS_1252, &v).unwrap();
    assert_eq!(s, "café");

    let out = encode_space_padded::<8>(WINDOWS_1252, &s).unwrap();
    assert_eq!(out.0.as_bytes()[..4], [b'c', b'a', b'f', 0xE9]);
    assert_eq!(out.0.as_bytes()[4..], [b' ', b' ', b' ', b' ']);
}

#[test]
fn encode_reports_too_long() {
    let err = encode_null_padded::<3>(WINDOWS_1252, "café").unwrap_err();
    assert_eq!(err, EncodingRsError::TooManyBytes { max: 3, found: 4 });
}

#[test]
fn decode_reports_malformed_input() {
    // A lone high surrogate is invalid UTF-16.
    // We'll model a fixed byte field containing UTF-16LE bytes: 0xD800.
    let v: FixedUtf8NullPadded<4> = FixedUtf8Bytes::from([0x00, 0xD8, 0, 0]).into();
    let err = decode_null_padded::<4>(UTF_16LE, &v).unwrap_err();
    assert_eq!(err, EncodingRsError::MalformedInput);
}

#[test]
fn encode_reports_unmappable_input() {
    // Emoji are not representable in Windows-1252.
    let err = encode_null_padded::<16>(WINDOWS_1252, "🙂").unwrap_err();
    assert_eq!(err, EncodingRsError::MalformedInput);
}

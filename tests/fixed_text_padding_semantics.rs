#![cfg(all(feature = "text_fixed", feature = "text_utf16", feature = "text_utf32"))]

use simple_endian::{
    FixedUtf16BeSpacePadded, FixedUtf16LeSpacePadded, FixedUtf32BeSpacePadded, FixedUtf32LeSpacePadded,
};

#[test]
fn utf32_space_padded_trims_only_trailing_spaces() {
    let v: FixedUtf32BeSpacePadded<5> = "a b".try_into().unwrap();
    let s = String::try_from(&v).unwrap();
    assert_eq!(s, "a b");

    let v: FixedUtf32LeSpacePadded<5> = "a b".try_into().unwrap();
    let s = String::try_from(&v).unwrap();
    assert_eq!(s, "a b");
}

#[test]
fn utf16_space_padded_trims_only_trailing_spaces() {
    let v: FixedUtf16BeSpacePadded<5> = "a b".try_into().unwrap();
    let s = String::try_from(&v).unwrap();
    assert_eq!(s, "a b");

    let v: FixedUtf16LeSpacePadded<5> = "a b".try_into().unwrap();
    let s = String::try_from(&v).unwrap();
    assert_eq!(s, "a b");
}

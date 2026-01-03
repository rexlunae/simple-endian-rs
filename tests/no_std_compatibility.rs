// Test that simple_endian works in a no_std environment with proper features.
// This test uses a lib-based approach to avoid issues with main() and linking.

#![no_std]
#![cfg_attr(not(test), no_main)]

use simple_endian::*;

// Basic smoke test for no_std environment
#[test]
fn test_no_std_basic_types() {
    // Test u16
    let x: u16be = 0x1234.into();
    let y: u16 = x.into();
    assert_eq!(y, 0x1234);

    // Test u32
    let x: u32le = 0xdeadbeef.into();
    let y: u32 = x.into();
    assert_eq!(y, 0xdeadbeef);

    // Test u64
    let x: u64be = 0x0102030405060708.into();
    let y: u64 = x.into();
    assert_eq!(y, 0x0102030405060708);
}

#[test]
fn test_no_std_byte_types() {
    // Test u8
    let x: BigEndian<u8> = 42.into();
    let y: u8 = x.into();
    assert_eq!(y, 42);

    // Test i8
    let x: LittleEndian<i8> = (-42).into();
    let y: i8 = x.into();
    assert_eq!(y, -42);
}

#[test]
fn test_no_std_conversions() {
    // Test that conversions work correctly
    let be: u32be = 0x12345678.into();
    let le: u32le = 0x12345678.into();
    
    let be_val: u32 = be.into();
    let le_val: u32 = le.into();
    
    assert_eq!(be_val, 0x12345678);
    assert_eq!(le_val, 0x12345678);
}

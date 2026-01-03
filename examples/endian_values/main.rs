//! Example: storing integers in big- or little-endian wrappers.
//!
//! This shows how `BigEndian<T>` / `LittleEndian<T>` let you:
//! - store values in a specific endian representation
//! - convert to/from native
//! - do arithmetic/bitwise ops without manually swapping bytes
//!
//! Run with:
//!
//! ```sh
//! cargo run --example endian_values
//! ```

use simple_endian::*;

fn main() {
    println!("=== Endian value wrappers ===\n");

    // Construct endian-tagged integers.
    let a: u32be = 0x1234_5678u32.into();
    let b: u32be = 0x0000_0001u32.into();

    println!("a (native) = 0x{:08x}", a.to_native());
    println!("b (native) = 0x{:08x}", b.to_native());

    // Arithmetic works on same-endian operands.
    let sum: u32be = a + b;
    println!("a + b = 0x{:08x}", sum.to_native());

    // Bitwise ops too (requires the default `bitwise` feature).
    let mask: u32be = 0xff00_0000u32.into();
    let high = a & mask;
    println!("a & 0xff00_0000 = 0x{:08x}", high.to_native());

    // Conversions between wrapper types are explicit: go through a native value.
    let a_le: u32le = a.to_native().into();
    println!("a as little-endian (native) = 0x{:08x}", a_le.to_native());

    // You can also use the generic wrappers directly.
    let x: BigEndian<u16> = BigEndian::from(0xbeefu16);
    let y: LittleEndian<u16> = LittleEndian::from(0xbeefu16);

    println!("BigEndian<u16> native: 0x{:04x}", x.to_native());
    println!("LittleEndian<u16> native: 0x{:04x}", y.to_native());

    println!("\nKey takeaway: endian wrappers are zero-cost tags that keep you honest.");
}

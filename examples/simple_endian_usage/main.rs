//! Example demonstrating the usage of the SimpleEndian trait.
//!
//! The `SimpleEndian` trait is for types that don't change based on endianness, such as
//! single-byte types, the unit type, booleans, and strings. All conversions for these types
//! are no-ops since endianness doesn't affect them.
//!
//! Run with:
//! ```sh
//! cargo run --example simple_endian_usage --all-features
//! ```

use simple_endian::SimpleEndian;

fn main() {
    println!("=== SimpleEndian Trait Examples ===\n");

    // Example 1: Unit type ()
    println!("1. Unit type ():");
    let unit = ();
    println!(
        "   unit.to_big_endian() == unit: {}",
        unit.to_big_endian() == unit
    );
    println!(
        "   unit.to_little_endian() == unit: {}",
        unit.to_little_endian() == unit
    );

    // Example 2: Boolean
    println!("\n2. Boolean:");
    let flag = true;
    println!("   flag = {}", flag);
    println!(
        "   flag.to_big_endian() == flag: {}",
        flag.to_big_endian() == flag
    );
    println!(
        "   flag.from_little_endian() == flag: {}",
        flag.from_little_endian() == flag
    );

    // Example 3: Single-byte integers
    println!("\n3. Single-byte integers:");
    let byte_val: u8 = 42;
    println!("   byte_val = {}", byte_val);
    println!(
        "   byte_val.to_big_endian() == byte_val: {}",
        byte_val.to_big_endian() == byte_val
    );
    println!(
        "   byte_val.to_little_endian() == byte_val: {}",
        byte_val.to_little_endian() == byte_val
    );

    let signed_byte: i8 = -13;
    println!("   signed_byte = {}", signed_byte);
    println!("   signed_byte conversions are also no-ops");

    // Example 4: char (with simple_char_impls feature)
    #[cfg(feature = "simple_char_impls")]
    {
        println!("\n4. char:");
        let c: char = '🦀';
        println!("   c = {}", c);
        println!("   c.to_big_endian() == c: {}", c.to_big_endian() == c);
        println!(
            "   c.to_little_endian() == c: {}",
            c.to_little_endian() == c
        );
    }

    // Example 5: Strings (with simple_string_impls feature)
    #[cfg(feature = "simple_string_impls")]
    {
        println!("\n5. String slices:");
        let text = "hello";
        println!("   text = \"{}\"", text);
        println!(
            "   text.to_big_endian() == text: {}",
            text.to_big_endian() == text
        );
        println!(
            "   text.to_little_endian() == text: {}",
            text.to_little_endian() == text
        );

        println!("\n6. Owned strings:");
        let owned = String::from("world");
        let converted = owned.clone().to_big_endian();
        println!("   owned = \"{}\"", owned);
        println!("   owned.to_big_endian() == owned: {}", converted == owned);
    }

    println!("\n=== Key Takeaway ===");
    println!("SimpleEndian types have no-op conversions because their representation");
    println!("doesn't change based on endianness. This is different from types like u32");
    println!("which require actual byte reordering between big-endian and little-endian.");
}

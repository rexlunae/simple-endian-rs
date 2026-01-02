//! Example: defining a portable `#[repr(C)]` struct with explicit endian fields.
//!
//! This pattern is useful for fixed binary formats where you want a stable layout.
//! The fields are stored in a specific endian representation, but you can still do
//! arithmetic by converting to native and back.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example explicit_struct_endian
//! ```

use simple_endian::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Header {
	magic: u32be,
	version: u16le,
	flags: u16be,
}

fn main() {
	println!("=== Explicit-endian struct ===\n");

	let mut h = Header {
		magic: 0xfeed_faceu32.into(),
		version: 1u16.into(),
		flags: 0u16.into(),
	};

	println!("magic  = 0x{:08x}", h.magic.to_native());
	println!("version= {}", h.version.to_native());
	println!("flags  = 0x{:04x}", h.flags.to_native());

	// Update fields.
	let next_version: u16 = h.version.to_native() + 1;
	h.version = next_version.into();

	// Bitwise ops on same-endian wrappers.
	h.flags |= 0x0001u16.into();

	println!("\nafter update:");
	println!("version= {}", h.version.to_native());
	println!("flags  = 0x{:04x}", h.flags.to_native());

	// If you need raw bytes, you can transmute *as bytes*.
	// (For a demo only. In real code, consider the crate's `io` feature or a serialization layer.)
	let raw: &[u8; core::mem::size_of::<Header>()] = unsafe { core::mem::transmute(&h) };
	println!("\nraw bytes ({}): {:02x?}", raw.len(), raw);
}

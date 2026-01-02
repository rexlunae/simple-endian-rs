//! Example demonstrating UTF-16 and UTF-32 conversions via `text_ops`.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example text_ops_usage --features text_all
//! ```

#[cfg(any(feature = "text_utf16", feature = "text_utf32"))]
use simple_endian::*;

fn main() {
	println!("=== text_ops UTF examples ===\n");

	let s = "Hello 🦀 — こんにちは";
	println!("input: {s}");

	#[cfg(feature = "text_utf16")]
	{
		println!("\n-- UTF-16 --");
		// Host-endian alias: Utf16StringLE on little-endian machines, Utf16StringBE on big-endian machines.
		let u16s = Utf16String::from(s);
		println!("u16 code units: {}", u16s.0.len());

		let back = String::try_from(&u16s).expect("valid UTF-16");
		println!("round-trip ok: {}", back == s);

		// Demonstrate error handling.
		let bad = [BigEndian::from(0xD800u16)];
		let err = String::try_from(Utf16StrBE::from(&bad[..])).unwrap_err();
		println!("invalid UTF-16 rejected: {err}");
	}

	#[cfg(feature = "text_utf32")]
	{
		println!("\n-- UTF-32 --");
		// Host-endian alias: Utf32StringLE on little-endian machines, Utf32StringBE on big-endian machines.
		let u32s = Utf32String::from(s);
		println!("u32 code units: {}", u32s.0.len());

		let back = String::try_from(&u32s).expect("valid UTF-32");
		println!("round-trip ok: {}", back == s);

		let bad = [BigEndian::from(0x11_0000u32)];
		let err = String::try_from(Utf32StrBE::from(&bad[..])).unwrap_err();
		println!("invalid UTF-32 rejected: {err}");
	}
}

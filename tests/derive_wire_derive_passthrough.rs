#![cfg(feature = "derive")]

use simple_endian::Endianize;

#[test]
fn wire_derive_passthrough_derives_traits_on_wire_type() {
	#[derive(Endianize)]
	#[endian(be)]
	#[wire_derive(Clone, Copy, Debug, PartialEq, Eq)]
	struct Logical {
		a: u16,
		b: u32,
	}

	// If the derive pass-through worked, these should compile.
	fn assert_copy<T: Copy>() {}
	fn assert_eq<T: Eq + PartialEq>() {}

	assert_copy::<LogicalWire>();
	assert_eq::<LogicalWire>();

	let w1 = LogicalWire {
		a: 1u16.into(),
		b: 2u32.into(),
	};
	let w2 = w1; // Copy
	let _ = format!("{w2:?}"); // Debug
	assert_eq!(w1, w2);
}

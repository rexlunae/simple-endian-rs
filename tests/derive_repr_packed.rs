#![cfg(feature = "derive")]

use simple_endian::Endianize;

#[test]
fn derive_wire_repr_packed_has_no_padding() {
	use core::mem::size_of;

	#[derive(Endianize)]
	#[endian(be)]
	#[wire_repr(packed)]
	#[allow(dead_code)]
	struct Packed {
		a: u8,
		b: u32,
		c: u16,
	}

	// With repr(packed), the wire layout should be tightly packed.
	assert_eq!(size_of::<PackedWire>(), 1 + 4 + 2);

	// Avoid taking references to packed fields; compute offsets via raw pointers.
	let base = core::ptr::null::<PackedWire>();
	unsafe {
		let base_addr = base as usize;
		let a_off = (core::ptr::addr_of!((*base).a) as usize) - base_addr;
		let b_off = (core::ptr::addr_of!((*base).b) as usize) - base_addr;
		let c_off = (core::ptr::addr_of!((*base).c) as usize) - base_addr;
		assert_eq!(a_off, 0);
		assert_eq!(b_off, 1);
		assert_eq!(c_off, 1 + 4);
	}
}

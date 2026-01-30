#[cfg(feature = "integer_impls")]
mod tests {
    use core::mem::{align_of, size_of};
    use simple_endian::{BigEndian, LittleEndian};

    #[repr(C)]
    struct C1 {
        a: u8,
        b: BigEndian<u32>,
    }

    #[repr(C)]
    struct C2 {
        a: u8,
        b: LittleEndian<u32>,
    }

    #[test]
    fn wrappers_have_same_size_and_align_as_inner() {
        assert_eq!(size_of::<BigEndian<u32>>(), size_of::<u32>());
        assert_eq!(align_of::<BigEndian<u32>>(), align_of::<u32>());

        assert_eq!(size_of::<LittleEndian<u32>>(), size_of::<u32>());
        assert_eq!(align_of::<LittleEndian<u32>>(), align_of::<u32>());
    }

    #[test]
    fn wrappers_do_not_change_c_layout_padding_rules() {
        // In repr(C), a u8 followed by a u32 has 3 bytes of padding, total 8 bytes.
        assert_eq!(size_of::<C1>(), 8);
        assert_eq!(size_of::<C2>(), 8);
    }

    #[test]
    fn packed_structs_stay_packed_with_wrappers() {
        #[repr(C, packed)]
        struct P {
            a: u8,
            b: BigEndian<u32>,
            c: LittleEndian<u16>,
        }

        // packed means no padding is inserted between fields.
        assert_eq!(size_of::<P>(), 1 + 4 + 2);
        assert_eq!(align_of::<P>(), 1);
    }
}

#[cfg(feature = "derive")]
mod endianize_packed_tests {
    use simple_endian::Endianize;
    use std::mem::{align_of, size_of};

    #[repr(C, packed)]
    #[derive(Endianize, Debug, PartialEq)]
    #[endian(le)]
    struct PackedStruct {
        a: u8,
        b: u16,
        c: u8,
    }

    #[repr(C)]
    #[derive(Endianize, Debug, PartialEq)]
    #[endian(le)]
    struct UnpackedStruct {
        a: u8,
        b: u16,
        c: u8,
    }

    #[test]
    fn packed_struct_wire_has_same_size_and_alignment() {
        assert_eq!(size_of::<PackedStruct>(), size_of::<PackedStructWire>());
        assert_eq!(align_of::<PackedStruct>(), align_of::<PackedStructWire>());
    }

    #[test]
    fn unpacked_struct_wire_has_same_size_and_alignment() {
        assert_eq!(size_of::<UnpackedStruct>(), size_of::<UnpackedStructWire>());
        assert_eq!(align_of::<UnpackedStruct>(), align_of::<UnpackedStructWire>());
    }

    #[repr(C, packed)]
    #[derive(Endianize, Debug, PartialEq)]
    #[endian(le)]
    struct PackedWithPadding {
        a: u8,
        b: u32,
        c: u8,
    }

    #[test]
    fn packed_struct_with_padding_wire_size_matches_base() {
        assert_eq!(size_of::<PackedWithPadding>(), size_of::<PackedWithPaddingWire>());
        assert_eq!(align_of::<PackedWithPadding>(), align_of::<PackedWithPaddingWire>());
    }
}

#[cfg(feature = "derive")]
mod endianize_mixed_types_tests {
    use simple_endian::Endianize;
    use std::mem::{align_of, size_of};

    #[repr(C, packed)]
    #[derive(Endianize, Debug, PartialEq, Default, Clone, Copy)]
    #[endian(le)]
    struct MixedInts {
        a: u8,
        b: u16,
        c: u32,
        d: u64,
    }

    #[test]
    fn packed_struct_mixed_ints_wire_size_and_align() {
        assert_eq!(size_of::<MixedInts>(), size_of::<MixedIntsWire>());
        assert_eq!(align_of::<MixedInts>(), align_of::<MixedIntsWire>());
    }

    #[repr(C, packed)]
    #[derive(Endianize, Debug, PartialEq, Default, Clone, Copy)]
    #[endian(le)]
    struct MixedFloats {
        a: f32,
        b: f64,
        c: u32,
        d: u8,
    }

    #[test]
    fn packed_struct_mixed_floats_wire_size_and_align() {
        assert_eq!(size_of::<MixedFloats>(), size_of::<MixedFloatsWire>());
        assert_eq!(align_of::<MixedFloats>(), align_of::<MixedFloatsWire>());
    }

    #[repr(C, packed)]
    #[derive(Endianize, Debug, PartialEq)]
    #[endian(le)]
    struct WithArray {
        a: u8,
        b: [u16; 3],
        c: u32,
    }

    #[test]
    fn packed_struct_with_array_wire_size_and_align() {
        assert_eq!(size_of::<WithArray>(), size_of::<WithArrayWire>());
        assert_eq!(align_of::<WithArray>(), align_of::<WithArrayWire>());
    }

    #[repr(C, packed)]
    #[derive(Endianize, Debug, PartialEq)]
    #[endian(le)]
    struct WithTuple {
        a: u8,
        b: (u16, u32),
        c: u8,
    }

    #[test]
    fn packed_struct_with_tuple_wire_size_and_align() {
        assert_eq!(size_of::<WithTuple>(), size_of::<WithTupleWire>());
        assert_eq!(align_of::<WithTuple>(), align_of::<WithTupleWire>());
    }
}

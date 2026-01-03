#[cfg(feature = "integer_impls")]
mod tests {
    use simple_endian::{BigEndian, LittleEndian};
    use core::mem::{align_of, size_of};

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

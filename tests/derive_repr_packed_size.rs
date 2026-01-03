#[cfg(feature = "derive")]
mod tests {
    use core::mem::size_of;
    use simple_endian::Endianize;

    /// PNG IHDR chunk - Image header
    /// Wire format: 13 bytes (width:4, height:4, bit_depth:1, color_type:1,
    /// compression_method:1, filter_method:1, interlace_method:1)
    #[derive(Endianize, Debug, Clone, Copy)]
    #[endian(be)]
    #[repr(C, packed)]
    pub struct IhdrChunk {
        pub width: u32,
        pub height: u32,
        pub bit_depth: u8,
        pub color_type: u8,
        pub compression_method: u8,
        pub filter_method: u8,
        pub interlace_method: u8,
    }

    #[test]
    fn packed_repr_struct_is_13_bytes() {
        assert_eq!(size_of::<IhdrChunk>(), 13);
    }

    #[test]
    fn generated_wire_type_is_also_13_bytes() {
        // The derived wire type should match the on-wire packed layout.
        assert_eq!(size_of::<IhdrChunkWire>(), 13);
    }
}

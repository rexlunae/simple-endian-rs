#[cfg(feature = "derive")]
mod tests {
    use simple_endian::Endianize;

    #[derive(Endianize, Clone, Debug, PartialEq, Eq)]
    #[endian(le)]
    struct WithBytes {
        magic: [u8; 4],
        n: u16,
        tail: [u8; 2],
    }

    #[test]
    fn wire_type_keeps_u8_arrays_unwrapped() {
        // This is mostly a compile-time test: if `[u8; N]` were wrapped in an endian type,
        // generation would fail or the field types would be surprising.
        let w = WithBytesWire {
            magic: *b"ABCD",
            n: 0x1234u16.into(),
            tail: [0xEE, 0xFF],
        };

        assert_eq!(w.magic, *b"ABCD");
        assert_eq!(w.n.to_native(), 0x1234);
        assert_eq!(w.tail, [0xEE, 0xFF]);
    }

    #[test]
    fn io_roundtrip_preserves_raw_bytes() {
        use simple_endian::{read_specific, write_specific};

        let w = WithBytesWire {
            magic: *b"ABCD",
            n: 0x1234u16.into(),
            tail: [0xEE, 0xFF],
        };

        let mut buf = Vec::new();
        write_specific(&mut buf, &w).unwrap();

        // magic (4 bytes) + n (LE u16) + tail (2 bytes)
        assert_eq!(buf, vec![b'A', b'B', b'C', b'D', 0x34, 0x12, 0xEE, 0xFF]);

        let w2: WithBytesWire = read_specific(&mut buf.as_slice()).unwrap();
        assert_eq!(w2.magic, *b"ABCD");
        assert_eq!(w2.n.to_native(), 0x1234);
        assert_eq!(w2.tail, [0xEE, 0xFF]);
    }
}

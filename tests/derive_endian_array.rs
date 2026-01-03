#[cfg(feature = "derive")]
mod tests {
    use simple_endian::Endianize;

    #[derive(Endianize, Clone, Debug, PartialEq, Eq)]
    #[endian(le)]
    struct ArrayStruct {
        words: [u16; 3],
        tail: [u8; 2],
    }

    #[test]
    fn wire_array_is_endian_wrapped_per_element() {
        let w = ArrayStructWire {
            words: [0x1122u16.into(), 0x3344u16.into(), 0x5566u16.into()],
            tail: [0xAA, 0xBB],
        };

        assert_eq!(w.words[0].to_native(), 0x1122);
        assert_eq!(w.words[1].to_native(), 0x3344);
        assert_eq!(w.words[2].to_native(), 0x5566);
        assert_eq!(w.tail, [0xAA, 0xBB]);
    }

    #[test]
    fn io_roundtrip_uses_element_endianness() {
        use simple_endian::{read_specific, write_specific};

        let w = ArrayStructWire {
            words: [0x1122u16.into(), 0x3344u16.into(), 0x5566u16.into()],
            tail: [0xAA, 0xBB],
        };

        let mut buf = Vec::new();
        write_specific(&mut buf, &w).unwrap();

        // Each u16 is written LE: 0x1122 -> [22 11], etc.
        assert_eq!(buf, vec![0x22, 0x11, 0x44, 0x33, 0x66, 0x55, 0xAA, 0xBB]);

        let w2: ArrayStructWire = read_specific(&mut buf.as_slice()).unwrap();
        assert_eq!(w2.words[0].to_native(), w.words[0].to_native());
        assert_eq!(w2.words[1].to_native(), w.words[1].to_native());
        assert_eq!(w2.words[2].to_native(), w.words[2].to_native());
        assert_eq!(w2.tail, w.tail);

        // Also validate wire->logical conversion for arrays.
        let logical = ArrayStruct::from(w2);
        assert_eq!(logical.words, [0x1122, 0x3344, 0x5566]);
        assert_eq!(logical.tail, [0xAA, 0xBB]);

        // logical->wire conversion uses element-wise Into.
        let w3 = ArrayStructWire::from(logical);
        assert_eq!(w3.words[0].to_native(), w.words[0].to_native());
        assert_eq!(w3.words[1].to_native(), w.words[1].to_native());
        assert_eq!(w3.words[2].to_native(), w.words[2].to_native());
        assert_eq!(w3.tail, w.tail);
    }
}

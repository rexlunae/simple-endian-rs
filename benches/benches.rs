use bencher::{Bencher, benchmark_group, benchmark_main, black_box};

use simple_endian::{BigEndian, LittleEndian};

benchmark_group!(
    benches,
    scalar_add_be,
    scalar_add_le,
    scalar_add_native,
    array_u16_le_endianize,
    array_u16_native,
    array_u8_passthrough,
    core_serialize_array_u16_le,
    core_serialize_array_u16_be,
    core_serialize_array_u32_le,
    core_serialize_array_u32_be,
    core_serialize_array_u64_le,
    core_serialize_array_u64_be,
    io_write_struct_u16_array_le,
    io_write_struct_u16_array_be,
    io_write_struct_u32_array_le,
    io_write_struct_u32_array_be,
    io_write_struct_u64_array_le,
    io_write_struct_u64_array_be,
    io_write_struct_u8_array,
    derive_from_logical_to_wire_numeric_only,
    derive_from_wire_to_logical_numeric_only,
    derive_tryfrom_wire_to_logical_text
);

benchmark_main!(benches);

fn scalar_add_be(b: &mut Bencher) {
    b.iter(|| {
        let mut a = BigEndian::from(black_box(1234567890u32));
        for _ in 0..100 {
            a = a + BigEndian::from(black_box(101010u32));
        }
        black_box(a)
    });
}

fn scalar_add_le(b: &mut Bencher) {
    b.iter(|| {
        let mut a = LittleEndian::from(black_box(1234567890u32));
        for _ in 0..100 {
            a = a + LittleEndian::from(black_box(101010u32));
        }
        black_box(a)
    });
}

fn scalar_add_native(b: &mut Bencher) {
    b.iter(|| {
        let mut a = black_box(1234567890u32);
        for _ in 0..100 {
            a = a + black_box(101010u32);
        }
        black_box(a)
    });
}

fn array_u16_le_endianize(b: &mut Bencher) {
    b.iter(|| {
        let input: [u16; 32] = black_box([0x1122u16; 32]);
        let wire: [LittleEndian<u16>; 32] = input.map(|x| x.into());
        let back: [u16; 32] = wire.map(|x| x.to_native());
        black_box(back)
    });
}

fn array_u16_native(b: &mut Bencher) {
    b.iter(|| {
        let input: [u16; 32] = black_box([0x1122u16; 32]);
        // Simulate a "do something" pass over the array.
        let mut out = [0u16; 32];
        out.copy_from_slice(&input);
        black_box(out)
    });
}

fn array_u8_passthrough(b: &mut Bencher) {
    b.iter(|| {
        let input: [u8; 64] = black_box([0xA5u8; 64]);
        let out = input;
        black_box(out)
    });
}

#[cfg(feature = "io-core")]
fn core_serialize_array_u16_le(b: &mut Bencher) {
    let v: [LittleEndian<u16>; 32] = [0x1122u16.into(); 32];
    b.iter(|| {
        let mut out = Vec::with_capacity(64);
        for x in black_box(&v) {
            out.extend_from_slice(&x.to_native().to_le_bytes());
        }
        black_box(out)
    });
}

#[cfg(not(feature = "io-core"))]
fn core_serialize_array_u16_le(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

#[cfg(feature = "io-core")]
fn core_serialize_array_u16_be(b: &mut Bencher) {
    let v: [BigEndian<u16>; 32] = [0x1122u16.into(); 32];
    b.iter(|| {
        let mut out = Vec::with_capacity(64);
        for x in black_box(&v) {
            out.extend_from_slice(&x.to_native().to_be_bytes());
        }
        black_box(out)
    });
}

#[cfg(not(feature = "io-core"))]
fn core_serialize_array_u16_be(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

#[cfg(feature = "io-core")]
fn core_serialize_array_u32_le(b: &mut Bencher) {
    let v: [LittleEndian<u32>; 32] = [0x1122_3344u32.into(); 32];
    b.iter(|| {
        let mut out = Vec::with_capacity(128);
        for x in black_box(&v) {
            out.extend_from_slice(&x.to_native().to_le_bytes());
        }
        black_box(out)
    });
}

#[cfg(not(feature = "io-core"))]
fn core_serialize_array_u32_le(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

#[cfg(feature = "io-core")]
fn core_serialize_array_u32_be(b: &mut Bencher) {
    let v: [BigEndian<u32>; 32] = [0x1122_3344u32.into(); 32];
    b.iter(|| {
        let mut out = Vec::with_capacity(128);
        for x in black_box(&v) {
            out.extend_from_slice(&x.to_native().to_be_bytes());
        }
        black_box(out)
    });
}

#[cfg(not(feature = "io-core"))]
fn core_serialize_array_u32_be(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

#[cfg(feature = "io-core")]
fn core_serialize_array_u64_le(b: &mut Bencher) {
    let v: [LittleEndian<u64>; 32] = [0x1122_3344_5566_7788u64.into(); 32];
    b.iter(|| {
        let mut out = Vec::with_capacity(256);
        for x in black_box(&v) {
            out.extend_from_slice(&x.to_native().to_le_bytes());
        }
        black_box(out)
    });
}

#[cfg(not(feature = "io-core"))]
fn core_serialize_array_u64_le(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

#[cfg(feature = "io-core")]
fn core_serialize_array_u64_be(b: &mut Bencher) {
    let v: [BigEndian<u64>; 32] = [0x1122_3344_5566_7788u64.into(); 32];
    b.iter(|| {
        let mut out = Vec::with_capacity(256);
        for x in black_box(&v) {
            out.extend_from_slice(&x.to_native().to_be_bytes());
        }
        black_box(out)
    });
}

#[cfg(not(feature = "io-core"))]
fn core_serialize_array_u64_be(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

#[cfg(feature = "io-std")]
fn io_write_struct_u16_array_le(b: &mut Bencher) {
    use simple_endian::write_specific;

    #[cfg(feature = "derive")]
    #[derive(simple_endian::Endianize)]
    #[endian(le)]
    struct Blob {
        words: [u16; 32],
    }

    // In benches we always compile with crate features; if derive isn't on, this bench does nothing.
    #[cfg(feature = "derive")]
    {
        let v = BlobWire {
            words: [0x1122u16.into(); 32],
        };
        b.iter(|| {
            let mut buf = Vec::with_capacity(64);
            write_specific(&mut buf, black_box(&v)).unwrap();
            black_box(buf)
        });
    }

    #[cfg(not(feature = "derive"))]
    {
        b.iter(|| black_box(0usize));
    }
}

#[cfg(not(feature = "io-std"))]
fn io_write_struct_u16_array_le(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

#[cfg(feature = "io-std")]
fn io_write_struct_u16_array_be(b: &mut Bencher) {
    use simple_endian::write_specific;

    #[cfg(feature = "derive")]
    #[derive(simple_endian::Endianize)]
    #[endian(be)]
    struct Blob {
        words: [u16; 32],
    }

    // In benches we always compile with crate features; if derive isn't on, this bench does nothing.
    #[cfg(feature = "derive")]
    {
        let v = BlobWire {
            words: [0x1122u16.into(); 32],
        };
        b.iter(|| {
            let mut buf = Vec::with_capacity(64);
            write_specific(&mut buf, black_box(&v)).unwrap();
            black_box(buf)
        });
    }

    #[cfg(not(feature = "derive"))]
    {
        b.iter(|| black_box(0usize));
    }
}

#[cfg(not(feature = "io-std"))]
fn io_write_struct_u16_array_be(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

#[cfg(feature = "io-std")]
fn io_write_struct_u32_array_le(b: &mut Bencher) {
    use simple_endian::write_specific;

    #[cfg(feature = "derive")]
    #[derive(simple_endian::Endianize)]
    #[endian(le)]
    struct Blob {
        words: [u32; 32],
    }

    #[cfg(feature = "derive")]
    {
        let v = BlobWire {
            words: [0x1122_3344u32.into(); 32],
        };
        b.iter(|| {
            let mut buf = Vec::with_capacity(128);
            write_specific(&mut buf, black_box(&v)).unwrap();
            black_box(buf)
        });
    }

    #[cfg(not(feature = "derive"))]
    {
        b.iter(|| black_box(0usize));
    }
}

#[cfg(not(feature = "io-std"))]
fn io_write_struct_u32_array_le(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

#[cfg(feature = "io-std")]
fn io_write_struct_u32_array_be(b: &mut Bencher) {
    use simple_endian::write_specific;

    #[cfg(feature = "derive")]
    #[derive(simple_endian::Endianize)]
    #[endian(be)]
    struct Blob {
        words: [u32; 32],
    }

    #[cfg(feature = "derive")]
    {
        let v = BlobWire {
            words: [0x1122_3344u32.into(); 32],
        };
        b.iter(|| {
            let mut buf = Vec::with_capacity(128);
            write_specific(&mut buf, black_box(&v)).unwrap();
            black_box(buf)
        });
    }

    #[cfg(not(feature = "derive"))]
    {
        b.iter(|| black_box(0usize));
    }
}

#[cfg(not(feature = "io-std"))]
fn io_write_struct_u32_array_be(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

#[cfg(feature = "io-std")]
fn io_write_struct_u64_array_le(b: &mut Bencher) {
    use simple_endian::write_specific;

    #[cfg(feature = "derive")]
    #[derive(simple_endian::Endianize)]
    #[endian(le)]
    struct Blob {
        words: [u64; 32],
    }

    #[cfg(feature = "derive")]
    {
        let v = BlobWire {
            words: [0x1122_3344_5566_7788u64.into(); 32],
        };
        b.iter(|| {
            let mut buf = Vec::with_capacity(256);
            write_specific(&mut buf, black_box(&v)).unwrap();
            black_box(buf)
        });
    }

    #[cfg(not(feature = "derive"))]
    {
        b.iter(|| black_box(0usize));
    }
}

#[cfg(not(feature = "io-std"))]
fn io_write_struct_u64_array_le(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

#[cfg(feature = "io-std")]
fn io_write_struct_u64_array_be(b: &mut Bencher) {
    use simple_endian::write_specific;

    #[cfg(feature = "derive")]
    #[derive(simple_endian::Endianize)]
    #[endian(be)]
    struct Blob {
        words: [u64; 32],
    }

    #[cfg(feature = "derive")]
    {
        let v = BlobWire {
            words: [0x1122_3344_5566_7788u64.into(); 32],
        };
        b.iter(|| {
            let mut buf = Vec::with_capacity(256);
            write_specific(&mut buf, black_box(&v)).unwrap();
            black_box(buf)
        });
    }

    #[cfg(not(feature = "derive"))]
    {
        b.iter(|| black_box(0usize));
    }
}

#[cfg(not(feature = "io-std"))]
fn io_write_struct_u64_array_be(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

#[cfg(feature = "io-std")]
fn io_write_struct_u8_array(b: &mut Bencher) {
    use simple_endian::write_specific;

    #[cfg(feature = "derive")]
    #[derive(simple_endian::Endianize)]
    #[endian(le)]
    struct Blob {
        bytes: [u8; 64],
    }

    #[cfg(feature = "derive")]
    {
        let v = BlobWire {
            bytes: [0xA5u8; 64],
        };
        b.iter(|| {
            let mut buf = Vec::with_capacity(64);
            write_specific(&mut buf, black_box(&v)).unwrap();
            black_box(buf)
        });
    }

    #[cfg(not(feature = "derive"))]
    {
        b.iter(|| black_box(0usize));
    }
}

#[cfg(not(feature = "io-std"))]
fn io_write_struct_u8_array(b: &mut Bencher) {
    b.iter(|| black_box(0usize));
}

fn derive_from_logical_to_wire_numeric_only(b: &mut Bencher) {
    #[cfg(feature = "derive")]
    #[derive(simple_endian::Endianize, Clone)]
    #[endian(le)]
    struct Header {
        a: u32,
        b: u16,
        words: [u16; 8],
        bytes: [u8; 8],
    }

    #[cfg(feature = "derive")]
    {
        let h = Header {
            a: black_box(1),
            b: black_box(2),
            words: black_box([0x1122u16; 8]),
            bytes: black_box([0xA5u8; 8]),
        };
        b.iter(|| {
            let w = HeaderWire::from(black_box(h.clone()));
            black_box(w)
        });
    }

    #[cfg(not(feature = "derive"))]
    {
        b.iter(|| black_box(0usize));
    }
}

fn derive_from_wire_to_logical_numeric_only(b: &mut Bencher) {
    #[cfg(feature = "derive")]
    #[derive(simple_endian::Endianize, Clone)]
    #[endian(le)]
    struct Header {
        a: u32,
        b: u16,
        words: [u16; 8],
        bytes: [u8; 8],
    }

    #[cfg(feature = "derive")]
    {
        b.iter(|| {
            let w = HeaderWire {
                a: 1u32.into(),
                b: 2u16.into(),
                words: [0x1122u16.into(); 8],
                bytes: [0xA5u8; 8],
            };
            let h = Header::from(black_box(w));
            black_box(h)
        });
    }

    #[cfg(not(feature = "derive"))]
    {
        b.iter(|| black_box(0usize));
    }
}

fn derive_tryfrom_wire_to_logical_text(b: &mut Bencher) {
    #[cfg(all(
        feature = "derive",
        feature = "text_all",
        feature = "simple_string_impls"
    ))]
    {
        #[derive(simple_endian::Endianize, Clone)]
        #[endian(le)]
        struct Header {
            a: u32,
            #[text(utf16, units = 8, pad = "space")]
            title: String,
        }

        b.iter(|| {
            let w = HeaderWire {
                a: 1u32.into(),
                title: "HI".try_into().unwrap(),
            };
            let h = Header::try_from(black_box(w)).unwrap();
            black_box(h)
        });
    }

    #[cfg(not(all(
        feature = "derive",
        feature = "text_all",
        feature = "simple_string_impls"
    )))]
    {
        b.iter(|| black_box(0usize));
    }
}

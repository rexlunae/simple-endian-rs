#![cfg(feature = "io-std")]

use simple_endian::{read_specific, read_specific_dyn, write_specific, write_specific_dyn};
use std::io::{self, Cursor, Read, Write};

pub trait ReadBytesExt: Read {
    fn read_u32_be(&mut self) -> io::Result<u32>;
}

impl<R: Read + ?Sized> ReadBytesExt for R {
    fn read_u32_be(&mut self) -> io::Result<u32> {
        // This is the primary ergonomic blocker described in the feature request:
        // calling the helper from an extension trait implemented for `R: ?Sized`.
        let be: simple_endian::BigEndian<u32> = read_specific(self)?;
        Ok(be.to_native())
    }
}

pub trait WriteBytesExt: Write {
    fn write_u32_be(&mut self, v: u32) -> io::Result<()>;
}

impl<W: Write + ?Sized> WriteBytesExt for W {
    fn write_u32_be(&mut self, v: u32) -> io::Result<()> {
        let be: simple_endian::BigEndian<u32> = v.into();
        write_specific(self, &be)
    }
}

#[test]
fn dyn_read_write_helpers_work() {
    // Write using &mut dyn Write
    let mut buf = Vec::new();
    {
        let w: &mut dyn Write = &mut buf;
        let be: simple_endian::BigEndian<u32> = 0x11223344u32.into();
        write_specific_dyn(w, &be).unwrap();
    }

    // Read using &mut dyn Read
    {
        let mut cur = Cursor::new(buf);
        let r: &mut dyn Read = &mut cur;
        let be: simple_endian::BigEndian<u32> = read_specific_dyn(r).unwrap();
        assert_eq!(be.to_native(), 0x11223344);
    }
}

#[test]
fn extension_traits_work_for_dyn_read_write() {
    let mut buf = Vec::new();
    {
        let w: &mut dyn Write = &mut buf;
        w.write_u32_be(0xAABBCCDD).unwrap();
    }

    {
        let mut cur = Cursor::new(buf);
        let r: &mut dyn Read = &mut cur;
        let n = r.read_u32_be().unwrap();
        assert_eq!(n, 0xAABBCCDD);
    }
}

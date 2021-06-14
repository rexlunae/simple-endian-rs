#![no_std]
#![feature(test)]
//! Many byte-order-handling libraries focus on providing code to convert to and from big- or little-endian.  However,
//! this requires users of those libraries to use a lot of explicit logic.  This library uses the Rust type system to
//! enforce conversions invisibly, and also ensure that they are done consistently.  A struct member can be read and written
//! simply using the standard From and Into trait methods (from() and into()).  No explicit endian checks are required.
//!  
//! # Example 1:
//!
//!```rust
//! use simple_endian::*;
//!
//! fn init() {
//!     #[repr(C)]
//!     struct BinPacket {
//!         a: u64be,
//!         b: u32be,
//!     }
//!     let mut bp = BinPacket{a: 0xfe.into(), b: 10.into()};
//!     let new_a = bp.a.to_native() * 1234;

//!     bp.a = new_a.into();
//!     bp.b = 1234.into();
//! }
//! ```
//!
//! Trying to write `bp.a = new_a;` causes an error because the type u64 can't be directly stored.
//!
//! # Example 2: Writing a portable struct to a file.
//!
//! Of course, just storing things in memory isn't that useful unless you write somewhere.
//!
//! ```rust
//! use simple_endian::*;
//! use std::fs::File;
//! use std::io::prelude::*;
//! use std::mem::{transmute, size_of};
//!
//! // We have to specify a representation in order to define the layout.
//! #[repr(C)]
//! struct BinBEStruct {
//!     pub a: u64be,
//!     b: u64be,
//!     c: f64be,
//! }
//!
//! fn main() -> std::io::Result<()> {
//!    let bin_struct = BinBEStruct{a: 345.into(), b: 0xfee.into(), c: 9.345.into()};
//!
//!    let mut pos = 0;
//!    let mut data_file = File::create(".test.bin")?;
//!    let buffer = unsafe { transmute::<&BinBEStruct, &[u8; size_of::<BinBEStruct>()]>(&bin_struct) };
//!
//!    while pos < buffer.len() {
//!        let bytes_written = data_file.write(&buffer[pos..])?;
//!        pos += bytes_written;
//!    }
//!    Ok(())
//! }
//! ```
//! # Example 3: Mmapping a portable struct with the memmap crate.
//!
//! You'll need to add memmap to your Cargo.toml to get this to actually work:
//!
//! ```rust
//! #![feature(rustc_private)]
//! extern crate memmap;
//!
//!  use std::{
//!     io::Error,
//!     fs::OpenOptions,
//!     mem::size_of,
//! };
//!
//! use memmap::MmapOptions;
//! use simple_endian::*;
//!
//! #[repr(C)]
//! struct MyBEStruct {
//!     header: u64be,
//!     label: [u8; 8],
//!     count: u128be,
//! }
//!
//! fn main() -> Result<(), Error> {
//!     let file = OpenOptions::new()
//!         .read(true).write(true).create(true)
//!         .open(".test.bin")?;
//!
//!     // Truncate the file to the size of the header.
//!     file.set_len(size_of::<MyBEStruct>() as u64)?;
//!     let mut mmap = unsafe { MmapOptions::new().map_mut(&file)? };
//!
//!     let mut ptr = mmap.as_mut_ptr() as *mut MyBEStruct;
//!
//!     unsafe {
//!         // Set the magic number
//!         (*ptr).header = 0xfeedface.into();
//!
//!         // Increment the counter each time we run.
//!         (*ptr).count += 1.into();
//!
//!         (*ptr).label = *b"Iamhere!";
//!     }
//!
//!     println!("done.");
//!     Ok(())
//! }
//! ```
//!

/// The main part of the library.  Contains the trait SpecificEndian<T> and BigEndian<T> and LittleEndian<T> structs, as well as the
/// implementation of those on the primitive types.
mod specific_endian;
pub use specific_endian::*;

/// Bitwise operations.  These should be equally fast in any endian.
#[cfg(feature = "bitwise")]
mod bitwise_ops;

/// Ops for comparisons and ordering.
#[cfg(feature = "comparisons")]
mod comparison_ops;

/// Shift operations.
#[cfg(feature = "shift_ops")]
mod shift_ops;

/// General math operations.
#[cfg(feature = "math_ops")]
mod math_ops;

/// Negations.
#[cfg(feature = "neg_ops")]
mod neg_ops;

/// Formatter impls.
#[cfg(feature = "format")]
mod formatting_ops;

/// The shorthand types (e.g u64be, f32le, etc)
mod shorthand_types;
pub use shorthand_types::*;

#[cfg(test)]
mod tests {
    extern crate test;
    use crate::*;
    use test::Bencher;

    #[bench]
    fn bench_integer_be(b: &mut Bencher) {
        b.iter(|| {
            let mut a = BigEndian::from(1234567890);
            for _ in 0..10 {
                a += BigEndian::from(101010);
                a &= BigEndian::from(0xf0f0f0);
                a *= BigEndian::from(123);
                a /= BigEndian::from(543);
            }
            test::black_box(a);
        });
    }
    #[bench]
    fn bench_integer_le(b: &mut Bencher) {
        b.iter(|| {
            let mut a = LittleEndian::from(1234567890);
            for _ in 0..10 {
                a += LittleEndian::from(101010);
                a &= LittleEndian::from(0xf0f0f0);
                a *= LittleEndian::from(123);
                a /= LittleEndian::from(543);
            }
            test::black_box(a);
        });
    }
    #[bench]
    fn bench_integer_ne(b: &mut Bencher) {
        b.iter(|| {
            let mut a = 1234567890;
            for _ in 0..10 {
                a += 101010;
                a &= 0xf0f0f0;
                a *= 123;
                a /= 543;
            }
            test::black_box(a);
        });
    }

    #[bench]
    fn bench_fp_be(b: &mut Bencher) {
        b.iter(|| {
            let mut a = BigEndian::from(1234567890.1);
            for _ in 0..10 {
                a += BigEndian::from(101010.0);
                a *= BigEndian::from(123.0);
                a /= BigEndian::from(543.0);
            }
            test::black_box(a);
        });
    }
    #[bench]
    fn bench_fp_le(b: &mut Bencher) {
        b.iter(|| {
            let mut a = LittleEndian::from(1234567890.1);
            for _ in 0..10 {
                a += LittleEndian::from(101010.0);
                a *= LittleEndian::from(123.0);
                a /= LittleEndian::from(543.0);
            }
            test::black_box(a);
        });
    }
    #[bench]
    fn bench_fp_ne(b: &mut Bencher) {
        b.iter(|| {
            let mut a = 1234567890.1;
            for _ in 0..10 {
                a += 101010.0;
                a *= 123.0;
                a /= 543.0;
            }
            test::black_box(a);
        });
    }

    #[bench]
    fn base_endian_test_be(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
                let a = i32::from_be(0xa5a5a5);
                test::black_box(a);
            }
        });
    }
    #[bench]
    fn base_endian_test_le(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
                let a = i32::from_le(0xa5a5a5);
                test::black_box(a);
            }
        });
    }
    #[bench]
    fn base_endian_test_ne(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
                let a = 0xa5a5a5_i32;
                test::black_box(a);
            }
        });
    }
    #[bench]
    fn base_endian_test_structured(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
                let a = LittleEndian { _v: 0xa5a5a5_i32 };
                test::black_box(a);
            }
        });
    }
}

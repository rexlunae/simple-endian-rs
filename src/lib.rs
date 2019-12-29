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

/// Ops for comparisons and ordering.
mod comparison_ops;

/// Bitwise operations.  These should be equally fast in any endian.
mod bitwise_ops;

/// Shift operations.
mod shift_ops;

/// General math operations.
mod math_ops;

/// General math operations.
mod formatting_ops;


mod types {
    #![allow(non_camel_case_types)]
    use super::*;
    /// Shorthand for LittleEndian<u16> 
    pub type u16le = LittleEndian<u16>;
    /// Shorthand for BigEndian<u16> 
    pub type u16be = BigEndian<u16>;
    /// Shorthand for LittleEndian<u32> 
    pub type u32le = LittleEndian<u32>;
    /// Shorthand for BigEndian<u32> 
    pub type u32be = BigEndian<u32>;
    /// Shorthand for LittleEndian<u64> 
    pub type u64le = LittleEndian<u64>;
    /// Shorthand for BigEndian<u64> 
    pub type u64be = BigEndian<u64>;
    /// Shorthand for LittleEndian<u128>
    pub type u128le = LittleEndian<u128>;
    /// Shorthand for BigEndian<u128> 
    pub type u128be = BigEndian<u128>;
    /// Shorthand for LittleEndian<usize>
    pub type usizele = LittleEndian<usize>;
    /// Shorthand for BigEndian<usize> 
    pub type usizebe = BigEndian<usize>;

    /// Shorthand for LittleEndian<i16>
    pub type i16le = LittleEndian<i16>;
    /// Shorthand for BigEndian<i16>
    pub type i16be = BigEndian<i16>;
    /// Shorthand for LittleEndian<i32>
    pub type i32le = LittleEndian<i32>;
    /// Shorthand for BigEndian<i32>
    pub type i32be = BigEndian<i32>;
    /// Shorthand for LittleEndian<i64>
    pub type i64le = LittleEndian<i64>;
    /// Shorthand for BigEndian<i64>
    pub type i64be = BigEndian<i64>;
    /// Shorthand for LittleEndian<i128>
    pub type i128le = LittleEndian<i128>;
    /// Shorthand for BigEndian<i128>
    pub type i128be = BigEndian<i128>;
    /// Shorthand for LittleEndian<isize>
    pub type isizele = LittleEndian<isize>;
    /// Shorthand for BigEndian<isize>
    pub type isizebe = BigEndian<isize>;

    /// Shorthand for LittleEndian<f32>
    pub type f32le = LittleEndian<f32>;
    /// Shorthand for BigEndian<f32>
    pub type f32be = BigEndian<f32>;

    /// Shorthand for LittleEndian<f64>
    pub type f64le = LittleEndian<f64>;
    /// Shorthand for BigEndian<f64>
    pub type f64be = BigEndian<f64>;

}

pub use types::*;

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use test::Bencher;
    use std::mem::size_of;


    #[test]
    fn declare_all() {
        let _a: BigEndian<i16> = 0xfe.into();
        let _a: LittleEndian<i16> = 0xfe.into();
        let _a: BigEndian<u16> = 0xfe.into();
        let _a: LittleEndian<u16> = 0xfe.into();

        let _a: BigEndian<i32> = 0xfe.into();
        let _a: LittleEndian<i32> = 0xfe.into();
        let _a: BigEndian<u32> = 0xfe.into();
        let _a: LittleEndian<u32> = 0xfe.into();

        let _a: BigEndian<i64> = 0xfe.into();
        let _a: LittleEndian<i64> = 0xfe.into();
        let _a: BigEndian<u64> = 0xfe.into();
        let _a: LittleEndian<u64> = 0xfe.into();

        let _a: BigEndian<i128> = 0xfe.into();
        let _a: LittleEndian<i128> = 0xfe.into();
        let _a: BigEndian<u128> = 0xfe.into();
        let _a: LittleEndian<u128> = 0xfe.into();
    }

    #[test]
    fn make_struct() {
        #[repr(C)]
        struct Foo (
            BigEndian<i16>,
            LittleEndian<i16>,
            BigEndian<u16>,
            LittleEndian<u16>,

            BigEndian<i32>,
            LittleEndian<i32>,
            BigEndian<u32>,
            LittleEndian<u32>,

            BigEndian<i64>,
            LittleEndian<i64>,
            BigEndian<u64>,
            LittleEndian<u64>,

            BigEndian<i128>,
            LittleEndian<i128>,
            BigEndian<u128>,
            LittleEndian<u128>,

            BigEndian<f32>,
            LittleEndian<f32>,
            BigEndian<f64>,
            LittleEndian<f64>,

        );

        let _foo = Foo(0.into(), 1.into(), 2.into(), 3.into(), 4.into(), 5.into(), 6.into(), 7.into(), 8.into(), 9.into(), 10.into(), 11.into(), 12.into(), 13.into(), 14.into(), 15.into(), (0.1).into(), (123.5).into(), (7.8).into(), (12345.4567).into());
    }

    #[test]
    fn store_be() {
        let be: BigEndian<u64> = 0xfe.into();
        if cfg!(byte_order = "big endian") {
            assert_eq!(be.to_bits(), 0xfe);
        }
        else {
            assert_eq!(be.to_bits(), 0xfe00000000000000);
        }
    }

    #[test]
    fn same_size() {
        assert_eq!(size_of::<u64be>(), size_of::<u64>());
    }

    #[test]
    fn store_le() {
        let le: LittleEndian<u64> = 0xfe.into();
        if cfg!(byte_order = "big endian") {
            assert_eq!(le.to_bits(), 0xfe00000000000000);
        }
        else {
            assert_eq!(le.to_bits(), 0xfe);
        }
    }

    #[test]
    fn cast() {
        let be = BigEndian::from(12345);
        let ne: u64 = be.into();
        assert_eq!(ne, 12345);
    }

    #[test]
    fn convert_back() {
        let be = BigEndian::from(12345);
        println!("{}", u64::from(be));
    }

    #[test]
    fn convert_to_native() {
        let be = BigEndian::from(0xfe);
        println!("{:x}, {:x}", be._v, be.to_native());
        assert_eq!(0xfe, be.to_native());
    }

    #[test]
    fn equality_test() {
        let be1 = BigEndian::from(12345);
        let be2 = BigEndian::from(12345);
        assert_eq!(true, be1 == be2);
    }

    #[test]
    fn not_equality_test() {
        let be1 = BigEndian::from(12345);
        let be2 = BigEndian::from(34565);
        assert_eq!(true, be1 != be2);
    }

    #[test]
    fn lt_test() {
        let be1 = BigEndian::from(12345);
        let be2 = BigEndian::from(34565);
        assert_eq!(true, be1 < be2);
    }

    #[test]
    fn bit_and_test() {
        let be1 = LittleEndian::<u64>::from(0x0f0);
        let be2 = LittleEndian::<u64>::from(0xff0);
        assert_eq!(0x0f0, u64::from(be1 & be2));
    }

    #[test]
    fn unary_not_test() {
        let be1 = BigEndian::<u16>::from(0x0f0);
        assert_eq!(0xff0f, u16::from(!be1));
    }

    #[test]
    fn store_fp_be() {
        let be1 = BigEndian::<f64>::from(1234.5678);
        if cfg!(byte_order = "little endian") {
            assert_ne!(1234.5678, be1.to_bits());
        }
        assert_eq!(1234.5678, f64::from(be1));
    }

    #[test]
    fn store_fp_le() {
        let le1 = LittleEndian::<f64>::from(1234.5678);
        if cfg!(byte_order = "big endian") {
            assert_ne!(1234.5678, le1.to_bits());
        }
        assert_eq!(1234.5678, f64::from(le1));
    }

    #[test]
    fn operate_fp_be() {
        let be1 = BigEndian::<f64>::from(1234.5678);
        let be2 = BigEndian::<f64>::from(6234.5678);
        assert_eq!(true, be1 < be2);
    }

    #[test]
    fn add_fp_be() {
        let mut be1 = f64be::from(1234.5678);
        be1 = be1 + 1.0.into();
        be1 += 1.0.into();
        assert_eq!(be1, 1236.5678.into());
    }

    #[test]
    fn subtract_fp_be() {
        let mut be1 = f64be::from(1234.5678);
        be1 = be1 - 1.0.into();
        be1 -= 1.0.into();
        assert_eq!(be1, 1232.5678.into());
    }

    #[test]
    fn mul_fp_be() {
        let mut be1 = f64be::from(1234.5678);
        be1 = be1 * 10.0.into();
        be1 *= 10.0.into();
        assert_eq!(be1, 123456.78.into());
    }

    #[test]
    fn div_fp_be() {
        let mut ne1: f64 = 1234.5678;
        let mut be1 = f64be::from(ne1);
        be1 = be1 / 10.0.into();
        ne1 = ne1 / 10.0;
        be1 /= 10.0.into();
        ne1 /= 10.0;
        assert_eq!(ne1, be1.into());
    }

    #[test]
    fn shl_be() {
        let mut ne1 = 0xfee1;
        let mut be1 = u64be::from(ne1);
        be1 = be1 << 5.into();
        ne1 = ne1 << 5;
        be1 <<= 5.into();
        ne1 <<= 5;
        assert_eq!(ne1, be1.into());
    }

    #[test]
    fn shr_be() {
        let mut ne1 = 0xfee1;
        let mut be1 = u64be::from(ne1);
        be1 = be1 >> 5.into();
        ne1 = ne1 >> 5;
        be1 >>= 5.into();
        ne1 >>= 5;
        assert_eq!(ne1, be1.into());
    }

    #[test]
    fn inferred_type() {
        let mut be1 = BigEndian::from(1234);
        be1 &= BigEndian::from(5678);
        println!("{} {} {}", be1, be1.to_bits(), be1.to_native());
        assert_eq!(be1, 1026.into());
    }

    #[test]
    fn inferred_type_fp() {
        let mut be1 = BigEndian::from(1234.5);
        be1 += BigEndian::from(5678.1);
        println!("{} {} {}", be1, be1.to_bits(), be1.to_native());
        assert_eq!(be1, 6912.6.into());
    }

    #[test]
    fn inferred_type_bigger() {
        let mut be1 = BigEndian::from(0x0feeddcc);
        be1 &= BigEndian::from(0xff00);
        println!("{} {} {}", be1, be1.to_bits(), be1.to_native());
        assert_eq!(be1, 0xdd00.into());
    }

    #[bench]
    fn bench_integer_be(b: &mut Bencher) {
        b.iter(|| {
            let mut a = BigEndian::from(1234567890);
            for _ in 0..10 {
                a += BigEndian::from(101010);
                a &= BigEndian::from(0xf0f0f0);
                a *= BigEndian::from(123);
                a /= BigEndian::from(543);
                println!("{}", a);
            }
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
                println!("{}", a);
            }
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
                println!("{}", a);
            }
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
                println!("{}", a);
            }
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
                println!("{}", a);
            }
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
                println!("{}", a);
            }
        });
    }

    #[bench]
    fn base_endian_test_be(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
               let a = i32::from_be(0xa5a5a5);
               println!("{}", a);
            }
        });
    }
    #[bench]
    fn base_endian_test_le(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
               let a = i32::from_le(0xa5a5a5);
               println!("{}", a);
            }
        });
    }
    #[bench]
    fn base_endian_test_ne(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
               let a = 0xa5a5a5_i32;
               println!("{}", a);
            }
        });
    }
    #[bench]
    fn base_endian_test_structured(b: &mut Bencher) {
        b.iter(|| {
            for _ in 0..1000 {
               let a = LittleEndian{_v: 0xa5a5a5_i32};
               println!("{}", a);
            }
        });
    }
    
    #[test]
    fn custom_type() {
        #[derive(Copy, Clone, Debug)]
        enum EndianAwareExample {
            BigEndianFunction(u64),
            LittleEndianFunction(u64),
        }
        impl SpecificEndian<EndianAwareExample> for EndianAwareExample {
            fn to_big_endian(&self) -> Self {
                match self {
                    EndianAwareExample::BigEndianFunction(_v) => *self,
                    EndianAwareExample::LittleEndianFunction(v) => EndianAwareExample::BigEndianFunction(v.to_big_endian()),
                }
            }
            fn to_little_endian(&self) -> Self {
                match self {
                    EndianAwareExample::LittleEndianFunction(_v) => *self,
                    EndianAwareExample::BigEndianFunction(v) => EndianAwareExample::BigEndianFunction(v.to_little_endian()),
                }
            }
            fn from_big_endian(&self) -> Self {
                match self {
                    EndianAwareExample::BigEndianFunction(_v) => *self,
                    EndianAwareExample::LittleEndianFunction(v) => EndianAwareExample::BigEndianFunction(v.to_big_endian()),
                }
            }
            fn from_little_endian(&self) -> Self {
                match self {
                    EndianAwareExample::LittleEndianFunction(_v) => *self,
                    EndianAwareExample::BigEndianFunction(v) => EndianAwareExample::BigEndianFunction(v.to_little_endian()),
                }
            }

        }
        let foo: BigEndian<EndianAwareExample> = EndianAwareExample::LittleEndianFunction(0xf0).into();
        #[allow(unused_assignments)]
        let mut value = 0;
        match foo.to_native() {
            EndianAwareExample::BigEndianFunction(v) => { println!("be: {:x}", v); value = v }
            EndianAwareExample::LittleEndianFunction(v) => { println!("le: {:x}", v); value = 0 }
        }
        assert_eq!(value, 0x0f000000000000000);
    }

}

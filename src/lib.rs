#![cfg_attr(all(not(test), not(any(feature = "io-std", feature = "io"))), no_std)]
// Enable `alloc` types (Vec/String) for optional text helpers in this crate.
#[cfg(any(feature = "text_utf16", feature = "text_utf32", feature = "text_fixed"))]
extern crate alloc;

/// Many byte-order-handling libraries focus on providing code to convert to and from big- or little-endian.  However,
/// this requires users of those libraries to use a lot of explicit logic.  This library uses the Rust type system to
/// enforce conversions invisibly, and also ensure that they are done consistently.  A struct member can be read and written
/// simply using the standard From and Into trait methods (from() and into()).  No explicit endian checks are required.
///
/// # Example 1:
///
/// ```rust
/// use simple_endian::*;
///
/// fn init() {
///     #[repr(C)]
///     struct BinPacket {
///         a: u64be,
///         b: u32be,
///     }
///     let mut bp = BinPacket{a: 0xfe.into(), b: 10.into()};
///     let new_a = bp.a.to_native() * 1234;
///
///     bp.a = new_a.into();
///     bp.b = 1234.into();
/// }
/// ```
///
/// Trying to write `bp.a = new_a;` causes an error because the type u64 can't be directly stored.
///
/// # Example 2: Writing a portable struct to a file.
///
/// Of course, just storing things in memory isn't that useful unless you write somewhere.
///
/// ```rust,no_run
/// use simple_endian::*;
/// use std::fs::File;
/// use std::io::prelude::*;
/// use std::mem::{transmute, size_of};
///
/// // We have to specify a representation in order to define the layout.
/// #[repr(C)]
/// struct BinBEStruct {
///     pub a: u64be,
///     b: u64be,
///     c: f64be,
/// }
///
/// fn main() -> std::io::Result<()> {
///     let bin_struct = BinBEStruct{a: 345.into(), b: 0xfee.into(), c: 9.345.into()};
///
///     let mut pos = 0;
///     let mut data_file = File::create(".test.bin")?;
///     let buffer = unsafe { transmute::<&BinBEStruct, &[u8; size_of::<BinBEStruct>()]>(&bin_struct) };
///
///     while pos < buffer.len() {
///         let bytes_written = data_file.write(&buffer[pos..])?;
///         pos += bytes_written;
///     }
///     Ok(())
/// }
/// ```
///
/// # Example 3: Mmapping a portable struct with the memmap crate.
///
/// You'll need to add memmap to your Cargo.toml to get this to actually work:
///
/// ```rust,no_run
/// extern crate memmap;
///
/// use std::{
///     io::Error,
///     fs::OpenOptions,
///     mem::size_of,
/// };
///
/// use memmap::MmapOptions;
/// use simple_endian::*;
///
/// #[repr(C)]
/// struct MyBEStruct {
///     header: u64be,
///     label: [u8; 8],
///     count: u128be,
/// }
///
/// fn main() -> Result<(), Error> {
///     let file = OpenOptions::new()
///         .read(true).write(true).create(true)
///         .open(".test.bin")?;
///
///     // Truncate the file to the size of the header.
///     file.set_len(size_of::<MyBEStruct>() as u64)?;
///     let mut mmap = unsafe { MmapOptions::new().map_mut(&file)? };
///
///     let mut ptr = mmap.as_mut_ptr() as *mut MyBEStruct;
///
///     unsafe {
///         // Set the magic number
///         (*ptr).header = 0xfeedface.into();
///
///         // Increment the counter each time we run.
///         (*ptr).count += 1.into();
///
///         (*ptr).label = *b"Iamhere!";
///     }
///
///     println!("done.");
///     Ok(())
/// }
/// ```
// (docs continue above)
#[warn(soft_unstable)]
/// The main part of the library.  Contains the trait SpecificEndian<T> and BigEndian<T> and LittleEndian<T> structs, as well as the
/// implementation of those on the primitive types.
mod specific_endian;
pub use specific_endian::*;

/// Types that do not change based on endianness and their implementations.
/// These types include the unit type, booleans, single-byte integers, and strings.
mod simple_endian;
pub use simple_endian::SimpleEndian;

/// Text/code-unit conversion helpers (UTF-16/UTF-32 and fixed-codepoint strings), behind feature flags.
#[cfg(any(feature = "text_utf16", feature = "text_utf32", feature = "text_fixed"))]
mod text_ops;

#[cfg(any(feature = "text_utf16", feature = "text_utf32", feature = "text_fixed"))]
pub use text_ops::*;

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

// Additional shorthand types (feature-gated).
#[cfg(all(feature = "integer_impls", feature = "nonzero"))]
mod shorthand_types_nonzero;

#[cfg(all(feature = "integer_impls", feature = "nonzero"))]
pub use shorthand_types_nonzero::*;

// Optional proc-macro derives.
#[cfg(feature = "derive")]
pub use simple_endian_derive::Endianize;

/// Optional IO helpers gated by the `io` feature.
#[cfg(any(feature = "io-core", feature = "io-std"))]
mod io;

#[cfg(feature = "io-core")]
pub use io::core_io::*;

#[cfg(feature = "io-std")]
pub use io::std_io::*;

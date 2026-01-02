# simple-endian

As of the 0.3 release, this library works on stable Rust.

Yet another library for handling endian in Rust.  We use Rust's type system to ensure correct conversions and build data types with explicit endianness defined for more seamless portability of data structures between processor types.  It should be fairly lightweight, and supports `#![no_std]`.

The key difference between this crate and other crates for handling endian is that in this crate, you aren't doing conversions manually at all.  You are just working in the endian that is appropriate to the data structures that you're dealing with, and we try to provide the needed traits and methods to do this in as normal a way as possible.

## Isn't there already a library for this

Yes, there are several.  But I'm not entirely happy with any of them.  Specifically, most of the libraries out there right now focus on providing functions for doing endian conversions.  Here are a few of them:

* The [endian](https://crates.io/crates/endian) crate.
* The [byteorder](https://crates.io/crates/byteorder) crate.
* The [bswap](https://crates.io/crates/bswap) crate.

byteorder has over 11 million downloads, and is clearly the prevailing way to handle endian in Rust.  However, it relies on programmers writing specific logic to swap bytes and requires accessing memory in ways that are unlike normal patterns for Rust.  But really, the only difference between a big- and little-endian value is the interpretation.  It shouldn't require a drastically different pattern of code to access them.

## So, why create another one

Because I think a better approach is to define your endianness as part of your data definition rather than in the logic of your program, and then to make byte order swaps as transparent as possible while still ensuring correctness.  And because the more like normal Rust data types and operations this is, the more likely it is that people will write portable code and data structures in the first place.

The philosophy of this crate is that you define your endian when you write your data structures, and then you use clear, imperative logic to mutate it without needing to think about the details or the host endian.  This makes it fundamentally different from crates that just give you a way to read a `&[u8; 8]` into a `u64`.

## Goals of this project

The goals of this crate are as follows:

1. Safely provide specific-endian types with low or no runtime overhead. There should be no runtime penalty when the host architecture matches the specified endianness, and very low penalty loads and stores otherwise.
2. Straightforward, architecture-independent declarative syntax which ensures that load and store operations as correct.
3. Ergonomic use patterns that maximize clarity and convenience without sacrificing correctness safety or correctness.
4. Incorrect handling of data should generate clear type errors at compile time.
5. Determination of correct endianness should be at declaration, and should not need to be repeated unless converting to a different endianness.
6. Support for all or Rust's built-in types where endianness is relevant.
7. The only dependency needed is the core crate. The std crate is used, however, for tests and benchmarks.

## Examples

```rust
use simple_endian::*;

let foo: u64be = 4.into();    //Stores 4 in foo in big endian.

println!("raw: {:x}, value: {:x}", foo.to_bits(), foo);
```

The output will depend on what sort of computer you're using.  If you're running a little-endian system, such as x86 (PCs, Macs, etc.), you will see the big endian representation interpreted as if little-endian, as it's stored in memory.  Note that the ``.to_bits()` method is mostly there for debugging purposes, and should not be used often.

This works in reverse as well:

```rust
use simple_endian::*;

let foo: u64be = 4.into();
let bar = u64::from(foo);

println!("value: {:x}", bar);
```

If you prefer, there's a convenience method so that you don't need to explicitly convert back to the basic native type.

```rust
use simple_endian::*;

let foo: u64be = 4.into();
let bar = foo.to_native();

println!("value: {:x}", bar);
```

And the type system ensures that native-endian values are never written without being converted into the proper endian.

```rust
let mut foo: u64be = 4.into();
foo = 7;     // Will not compile without .into().
```

## How it works

At its core, this crate centers around one trait, called `SpecificEndian<T>`, and the generic structs `BigEndian<T>` and `LittleEndian<T>`.  `SpecificEndian<T>` is required to make `BigEndian<T>` and `LittleEndian<T>` structs.  Any data type that implements `SpecificEndian`, even if it handles endianness in unusual ways, can be assigned `BigEndian` and `LittleEndian` variants using the structs in this crate, the main possibly limitation being that they need to use the same underlying structure.  In fact, `u64be` is just a type alias for `BigEndian<u64>`.  There is no memory footprint added by the `BigEndian<T>` and `LittleEndian<T>` structs, in fact, in most cases it uses the type T to store the data.  The only purpose of the structs is to tag them for Rust's type system to enforce correct accesses.  This means that it can be used directly within larger structs, and then the entire struct can be written to disk, send over a network socket, and otherwise shared between processor architectures using the same code regardless of host endian using declarative logic without any conditionals.

This crate provides `SpecificEndian` implementations for most of the built-in types in Rust, including:

* Single-byte values (`i8`, `u8`, `bool`), although this really doesn't do much but provide completeness.
* The multi-byte integers: `u16`, `u32`, `u64`, `u128`, `usize`, `i16`, `i32`, `i64`, `i128`, `isize`
* The floats: `f32`, `f64`.

At the time of this writing, the only common built-in type that doesn't have an implementation is char, and this is because some values of char that would be possible from the binary representation would cause a panic.  Usually, you wouldn't want to store a char directly anyway, so this is probably a small limitation.

This crate also provides implementations of a variety of useful traits for the types that it wraps, including boolean logic implementations for the integer types, including bools.  This allows most boolean logic operations to be performed without any endian conversions using ordinary operators.  You are required to use same-endian operands, however, like this:

```rust
use simple_endian::*;

let ip: BigEndian::<u32> = 0x0a00000a.into();
let subnet_mask = BigEndian::from(0xff000000u32);

let network = ip & subnet_mask;

println!("value: {:x}", network);
```

As you see, the network is calculated by masking the IP address with the subnet mask in a way that the programmer barely has to think about the conversion operations.

Alternatively, you might want to define a structure with the elements typed so that it can be moved around as a unit.

```rust
use simple_endian::*;

#[derive(Debug)]
#[repr(C)]
struct NetworkConfig {
    address: BigEndian<u32>,
    mask: BigEndian<u32>,
    network: BigEndian<u32>,
}

let config = NetworkConfig{address: 0x0a00000a.into(), mask: 0xff000000.into(), network: (0x0a00000a & 0xff000000).into()}

println!("value: {:x?}", config);
```

Note that the println! will convert the values to native endian.

And finally, this crate implements a number of traits that allow most of the basic arithmetic operators to be used on the Big- and LittleEndian variants of all of the types, where appropriate, including for the floats.  There is a certain amount of overhead to this, since each operation requires at least one and often two or more endian conversions, however, since this crate aims to minimize the cost of writing portable code, they are provided to reduce friction to adoption.  If you are writing code that is extremely sensitive to such overhead, it might make sense to convert to native endian, do your operations, and then store back in the specified endian using `.into()` or similar.  That said, the overhead is often very small, and Rust's optimizer is very good, so I would encourage you to do some actual benchmarking before taking an unergonomic approach to your code.  There are too many traits implemented to list them here, so I recommend consulting [the documentation](https://docs.rs/simple_endian/).  Alternatively, you could just try what you want to do, and see if it compiles.  It shouldn't ever allow you to compile something that doesn't handle endianness correctly unless you work pretty hard at it.

### Representations and ABI

You might notice that we used `#[repr(C)]` in the data struct above, and you might be wondering why. It is often the case that you want to write a `struct` that has a very specific layout when you are writing structures that will be directly read from and written to some medium. Rust's default ABI does not guarantee this. For that reason, all of the structs defined in this crate are `#[repr(transparent)]`, and it is strongly recommended if you do plan to directly write these structures to disk or the network, that you do something to ensure a consistent layout similar or otherwise guarantee the order in which the fields are stored.

## Operations on Types

In addition to offering support for ensuring that correct endianness is used by leveraging the Rust type system, this crate also provides implementations of a number of traits from the `core` library that allow you to work with values directly without converting them to native endian types first. In many cases, this is literally a zero-cost capability, because bitwise operations are endian-agnostic, and as long as you are using other `SpecificEndian` types, there is no overhead to doing operations on them directly. In cases where a conversion to native endian is necessary, the crate will perform the conversion, and return a value in the same type as the input.

## Features

Although this crate includes a lot of useful functionality up front, including it all can increase your compiled size significantly. For size-conscious applications, I recommend not including everything.

By default, this crate will compile with all supported features. Although this means that in most cases, almost anything you would want to do would work out of the box, in practical terms, this can make the crate rather large. To avoid bloat, it might be best to set `default-features = false` in your "Cargo.toml", and add back in the features you actually need.

The two most useful features are probably the ones that control support for big- and little- endians:

* `big_endian`
* `little_endian`

Others are broken into categories:

* Operations types - These can make the use of `SpecificEndian<T>` types more ergonimic, and allow for some amount of optimization by avoiding unnecessary convertions to and from native endian.
  * `bitwise`
  * `comparisons`
  * `math_ops`
  * `neg_ops`
  * `shift_ops`
* Support for formatting in the `format` feature.
* Support for different types
  * `float_impls`
  * `integer_impls`
  * `byte_impls`

### Text helpers (feature: `text_utf16`, `text_utf32`, `text_fixed`)

This crate also includes **optional, feature-gated helpers** for working with Unicode text in
binary formats and foreign-function interfaces.

These features are designed for cases where a format/API specifies a fixed encoding and/or
endianness (for example: “UTF-16LE code units”, “UTF-32BE code units”, or “exactly 8 UTF-16
code units stored inline in the struct”).

Enable them like this:

```toml
[dependencies.simple_endian]
version = "0.4"
features = ["text_all"]
```

Or pick only what you need:

* `text_utf16` – UTF-16 helper types (`Utf16String*`, `Utf16Str*`) and conversions.
* `text_utf32` – UTF-32 helper types (`Utf32String*`, `Utf32Str*`) and conversions.
* `text_fixed` – fixed-size, inline string helpers.

#### Endianness-aware text buffers

For UTF-16 and UTF-32, there are **explicit endianness** types and **host-endian aliases**:

* Explicit: `Utf16StringBE` / `Utf16StringLE`, `Utf32StringBE` / `Utf32StringLE`
* Host-endian aliases: `Utf16String`, `Utf32String` (pick BE/LE based on `target_endian`)

This lets you keep code portable while still being able to target a stable on-the-wire encoding
when you need it (protocols, file formats, hashing over bytes, etc.).

#### Fixed-size, inline UTF-16 fields in binary structs

Many binary formats (and some ABIs) store strings inline using a fixed number of UTF-16 code
units. For that scenario, enable `text_fixed` + `text_utf16`.

If you know the *wire format* is UTF-16LE (very common on Windows), the most direct way to
model it is to wrap the **host-endian** fixed buffer in `LittleEndian<...>`:

* `FixedUtf16CodeUnits<K>` – stores **exactly K UTF-16 code units inline** (host-endian)
* `LittleEndian<FixedUtf16CodeUnits<K>>` – stores **exactly K UTF-16LE code units inline**

The older explicit-endian names still exist too:

* `FixedUtf16LeCodeUnits<K>` – stores **exactly K UTF-16LE code units inline**
* `FixedUtf16BeCodeUnits<K>` – stores **exactly K UTF-16BE code units inline**

There are also three convention-specific wrappers for common layouts:

* `FixedUtf16LePacked<K>` / `FixedUtf16BePacked<K>`
* `FixedUtf16LeNullPadded<K>` / `FixedUtf16BeNullPadded<K>`
* `FixedUtf16LeSpacePadded<K>` / `FixedUtf16BeSpacePadded<K>`

Example: a C-layout struct containing a fixed-size UTF-16LE name field:

```rust
use simple_endian::{FixedUtf16CodeUnits, FixedUtf16LeNullPadded, LittleEndian, u32le};

const NAME_UNITS: usize = 16;

#[repr(C)]
struct Header {
    id: u32le,
    // Exactly 16 UTF-16LE code units stored inline.
    // (a host-endian fixed buffer tagged as little-endian on the wire)
    name: LittleEndian<FixedUtf16CodeUnits<NAME_UNITS>>,
}

fn build() -> Header {
    // (1) From raw code units (infallible; length is in the type).
    let raw: [LittleEndian<u16>; NAME_UNITS] = [LittleEndian::from(0); NAME_UNITS];
    // Wrap the fixed buffer in `LittleEndian<...>` to state the **wire encoding**.
    let name_from_units: LittleEndian<FixedUtf16CodeUnits<NAME_UNITS>> = raw.into();

    // (2) Or TryFrom<&str> (fails only if `encode_utf16()` is longer than NAME_UNITS;
    //     otherwise it auto-pads with NUL code units).
    let name_from_str: LittleEndian<FixedUtf16CodeUnits<NAME_UNITS>> = "HELLO".try_into().unwrap();

    // If you prefer convention wrappers (packed / NUL-padded / space-padded), they still exist:
    let _compat: FixedUtf16LeNullPadded<NAME_UNITS> = "HELLO".try_into().unwrap();

    let _ = name_from_str;
    Header { id: 1.into(), name: name_from_units }
}
```

Example: a C-layout struct containing a fixed-size **UTF-16BE, space-padded** field:

```rust
use simple_endian::{BigEndian, FixedUtf16BeSpacePadded, u16be, u32be};

const TITLE_UNITS: usize = 12;

#[repr(C)]
struct Record {
    // Some big-endian scalar fields...
    id: u32be,
    flags: u16be,
    // Exactly 12 UTF-16BE code units stored inline.
    // If the encoded string is shorter than TITLE_UNITS, the remainder is padded with U+0020.
    title: FixedUtf16BeSpacePadded<TITLE_UNITS>,
}

fn build_record() -> Record {
    // Initialize from a string input (pads with spaces up to TITLE_UNITS).
    let title: FixedUtf16BeSpacePadded<TITLE_UNITS> = "HELLO".try_into().unwrap();

    Record {
        id: 1u32.into(),
        flags: 0u16.into(),
        title,
    }
}

fn update_title(rec: &mut Record, s: &str) {
    // Replace it later from another string.
    rec.title = s.try_into().unwrap();

    // If you need to tag the field as an endian-wrapped value for a generic API,
    // you can wrap it too (the wrapper uses TryFrom via the inner fixed type).
    let _wire: BigEndian<FixedUtf16BeSpacePadded<TITLE_UNITS>> = s.try_into().unwrap();
}
```

#### Fixed number of Unicode codepoints (for inline tags/labels)

If you need “exactly N Unicode scalar values” inline in a struct (useful for tags, short labels,
or fixed-width identifiers), enable `text_fixed` and use `FixedCodepointString<N>`.

#### Cross-language / FFI notes (JavaScript, Windows, etc.)

* **JavaScript strings** are specified in terms of UTF-16 code units (historically, “UCS-2”,
  but modern JS uses UTF-16 semantics). If you’re bridging to JS via FFI or a binary protocol,
  UTF-16 helpers can be used to make the encoding/decoding explicit.
* Many native APIs and ABIs (notably **Windows wide strings**) use UTF-16LE code units. The
  `Utf16*LE` and `FixedUtf16Le*` types are intended to make those representations easy to model.
* If you need a stable, platform-independent wire format, prefer explicit `*BE`/`*LE` types
  over host-endian aliases.

## Performance

For the most part, the performance of the endian operations are extremely fast, even compared to native operations.  The main exception is the std::fmt implementations, which are in some cases quite a bit slower than default.  I'm open to suggestions on how to improve the performance, but it might be worth using .to_native() instead of directly printing the wrapped types in performance-critical contexts.

## See Also

This crate allows for the manipulation of specific-endian structures in memory.  It does not provide any facility for reading or writing those structures, which would probably be necessary in most use cases.  See the following other crates for that functionality:

* Rust's standard std::mem::[transmute](https://doc.rust-lang.org/std/mem/fn.transmute.html) call:
* [safe-transmute](https://crates.io/crates/safe-transmute)
* [endian-trait](https://crates.io/crates/endian_trait)
* [byteordered](https://crates.io/crates/byteordered)
* [persistance](https://crates.io/crates/persistence) - A library that wraps structs in mmap, and can be used well with this to make those structs portable.
* [endian-type](https://crates.io/crates/endian-type) - The endian-type library.  This appears to be essentially the same approach as this crate, but contains less functionality.
* [endian-types](https://crates.io/crates/endian-types) - Another very similar approach.

## IO helpers (feature: `io`)

This crate provides optional, feature-gated IO helpers for reading and writing endian-aware values directly from `Read`/`Write` streams. Enable them by adding the `io` feature in your `Cargo.toml`:

```toml
[dependencies.simple_endian]
version = "0.3"
features = ["io"]
```

The `io` feature enables `std` for this crate (the library remains `#![no_std]` when the feature is not enabled) and exposes the following helpers in `simple_endian::io`:

* `read_specific<R, E>(reader: &mut R) -> io::Result<E>` — Read an endian-wrapped value of type `E` (for example `BigEndian<u32>`) from `reader`.
* `write_specific<W, E>(writer: &mut W, v: &E) -> io::Result<()>` — Write the endian-wrapped value to `writer`.

Additionally, helper traits are provided so types can implement custom read/write behavior:

* `EndianRead` — types implementing this expose `read_from<R: Read>(reader: &mut R) -> io::Result<Self>`.
* `EndianWrite` — types implementing this expose `write_to<W: Write>(&self, writer: &mut W) -> io::Result<()>`.

Big- and Little-endian wrappers implement those traits for the built-in types, so you can use the generic functions like this:

```rust
use simple_endian::io::{read_specific, write_specific};
use simple_endian::*;
use std::io::Cursor;

fn example() -> std::io::Result<()> {
  let val: BigEndian<u32> = 0x12345678u32.into();
  let mut buf = Vec::new();
  write_specific(&mut buf, &val)?;

  let mut cur = Cursor::new(buf);
  let out: BigEndian<u32> = read_specific(&mut cur)?;
  assert_eq!(out.to_native(), 0x12345678u32);
  Ok(())
}
```

Notes

* The current implementation supports types with sizes 1, 2, 4, 8 and 16 bytes (integers and floats). Attempts to read/write unsupported sizes return an `io::Error`.
* Internally the implementation uses low-level conversions to reconstruct values from bytes; the code uses `unsafe` `transmute_copy` in places for genericity. If you need a fully safe approach, we can add a small trait to provide safe byte conversions for each supported `T`.
* Extensive unit tests for the IO helpers are included and run when you enable the `io` feature (`cargo test --features io`).

If you'd like, I can add example snippets to the crate root docs or add a dedicated `examples/` folder demonstrating reading/writing structs with mixed endian fields.

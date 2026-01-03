# simple-endian

`simple_endian` is a toolkit for **describing binary formats in Rust**: network protocols, file formats, and on-disk/on-wire data structures that must be stored **consistently regardless of host CPU endianness**.

It’s not only about swapping bytes.
The goal is to let you *design* and *implement* binary layouts ergonomically, with the compiler helping enforce that:

* numeric fields are always read/written with the correct byte order
* your in-memory logic can stay close to “ordinary” Rust
* conversions happen explicitly at boundaries (wire ↔ native) so mistakes become type errors

At the core, you declare endianness in the **data definition** (`BigEndian<T>`, `LittleEndian<T>`, and shorthand aliases like `u32be`, `u16le`) and then:

* operate on endian-aware values using normal operators and traits
* convert at boundaries via `.to_native()` / `.into()`

Optional features expand this into a full wire-format toolkit:

* `derive`: generate `*Wire` helper types from logical structs/enums (stable layout, endian-correct fields)
* `io-std` / `io-core`: `read_specific` / `write_specific` for safe endian-aware IO with binary structures.
* `text_*`: fixed-size UTF-16/UTF-32 helpers for formats that standardize on those encodings (e.g. UTF-16LE)

The crate is designed to be lightweight and supports `#![no_std]` (derive/IO/text are feature-gated).

If you’re using LLM-powered tooling, there’s a concise, repository-specific usage guide in `LLMs.txt`.


## No-std Support

`simple_endian` works in `no_std` environments. The library automatically switches to `no_std` mode when:
* You're not running tests
* The `io-std` or `io` features are not enabled

To use `simple_endian` in a `no_std` project, disable default features and enable only what you need:

```toml
[dependencies]
simple_endian = { version = "0.4", default-features = false, features = ["integer_impls", "both_endian", "byte_impls"] }
```

Common feature combinations for `no_std`:
* **Minimal**: `integer_impls`, `both_endian`, `byte_impls` — basic endian types
* **With derive**: add `derive` — for `#[derive(Endianize)]` support
* **With core IO**: add `io-core` — for slice-based read/write helpers (no `std::io` dependency)

The library compiles successfully for embedded targets like `thumbv7m-none-eabi`. See `tests/no_std_compatibility.rs` for examples.
## New Text Handling

New in the `0.4` release is a set of feature-gated types and conversions for handling on-disk/on-wire Unicode encodings **other than UTF-8**.

This matters because a lot of real-world binary formats *standardize* their text fields as:

* **UTF-16LE/BE code units** (common in file formats and OS metadata)
* **UTF-32LE/BE code units** (sometimes used for fixed-width fields)
* fixed-size strings with explicit padding rules

Trying to model those with Rust `String`/`&str` directly usually leads to ad-hoc byte slicing and accidental host-endian assumptions. The text helpers here are meant to keep your code:

* explicit about encoding
* explicit about endianness of the *code units*
* safe (bounded, validated conversions)
* consistent with the rest of the crate’s “wire types are types” approach

### Features

Text support is opt-in:

* `text_utf8` – fixed-size UTF-8 byte field helpers
* `text_utf16` – UTF-16 code unit types and conversions
* `text_utf32` – UTF-32 code unit types and conversions
* `text_fixed` – fixed-size (const-generic) text field wrappers
* `text_all` – convenience alias enabling the above

These are designed to work with:

* `derive` (via `#[text(...)]` on struct fields)
* `io-std`/`io-core` (via `read_specific` / `write_specific` for fixed UTF fields)

### Fixed-size UTF-16/UTF-32 fields (padding semantics)

The fixed types represent **exactly $N$ code units** on the wire. They are not growable strings.
They come in a few common padding styles, for example:

* `FixedUtf16LeSpacePadded<N>` – UTF-16LE, right-padded with the space code unit (`0x0020`); decoding trims trailing spaces
* `FixedUtf16LeNullPadded<N>` – UTF-16LE, right-padded with NUL (`0x0000`); decoding trims trailing NULs

Similar wrappers exist for BE and for UTF-32.

There are also fixed-size **UTF-8 byte** field wrappers:

* `FixedUtf8NullPadded<N>` – right-padded with `0x00`; decoding trims trailing NULs
* `FixedUtf8SpacePadded<N>` – right-padded with `0x20` (space); decoding trims trailing spaces

Note: the text APIs are entirely feature-gated. If you don't enable any `text_*` features, these types and conversions won't be part of your build.

### Using `encoding_rs` (optional)

If your wire format stores a fixed-size byte field that uses a non-UTF8 encoding (e.g. Windows-1252), enable the `text_encoding_rs` feature.

This keeps the crate `no_std` by default; the helper functions return `String` and therefore require `alloc`.

```rust
use encoding_rs::WINDOWS_1252;
use simple_endian::text_ops::encoding_rs::{decode_null_padded, encode_null_padded};
use simple_endian::FixedUtf8NullPadded;

let wire: FixedUtf8NullPadded<8> = encode_null_padded(WINDOWS_1252, "café").unwrap();
let s = decode_null_padded(WINDOWS_1252, &wire).unwrap();
assert_eq!(s, "café");
```

The important endianness point: **the endianness applies to the UTF code units**, not to the host.
So `FixedUtf16Le...` is *always* little-endian on the wire, even on a big-endian CPU.

### Example: a fixed UTF-16LE field in a wire struct

This mirrors formats like FAT long file names (UTF-16LE) or other metadata blocks that store fixed-width UTF-16.

```rust
use simple_endian::{Endianize, FixedUtf16LeSpacePadded};
use simple_endian::{read_specific, write_specific};


#[derive(Endianize, Debug)]
#[endian(le)]
#[repr(C)]
struct Entry {
  id: u16,

  // 8 UTF-16LE code units, padded with spaces on the wire.
  #[text(utf16, units = 8, pad = "space")]
  name: String,
}

fn round_trip() {
  let wire = EntryWire {
    id: 7u16.into(),
    name: "ALICE".try_into().unwrap(),
  };

  let mut buf = Vec::new();
  write_specific(&mut buf, &wire).unwrap();

  let mut cur = std::io::Cursor::new(buf);
  let decoded: EntryWire = read_specific(&mut cur).unwrap();
  let name = String::try_from(&decoded.name).unwrap();
  assert_eq!(name, "ALICE");
}
```

Notes:

* For formats that are *actually ASCII* (e.g. FAT16 short names, many protocol tokens), keep them as `[u8; N]` bytes and validate/trim explicitly.
* Use UTF-16/UTF-32 helpers when the spec calls for them; that’s where they shine.
* The fixed types are great for avoiding variable-length parsing and for guaranteeing layout.

### Example: a fixed UTF-8 field in a wire struct

This is useful for formats that store fixed-width, right-padded UTF-8 bytes.

```rust
use simple_endian::Endianize;
use simple_endian::{read_specific, write_specific};

#[derive(Endianize, Debug)]
#[endian(be)]
#[repr(C)]
struct Entry {
  id: u16,

  // 8 UTF-8 bytes, padded with NULs on the wire.
  #[text(utf8, units = 8, pad = "null")]
  name: String,
}

fn round_trip() {
  let wire = EntryWire {
    id: 7u16.into(),
    name: "ALICE".try_into().unwrap(),
  };

  let mut buf = Vec::new();
  write_specific(&mut buf, &wire).unwrap();

  let mut cur = std::io::Cursor::new(buf);
  let decoded: EntryWire = read_specific(&mut cur).unwrap();
  let name = String::try_from(&decoded.name).unwrap();
  assert_eq!(name, "ALICE");
}
```

## Isn’t there already a library for this?

Yes, there are several that cover at least a part of this functionality. Most focus on *functions* for byte swapping / reading numbers from byte slices. A few well-known ones:

* The [endian](https://crates.io/crates/endian) crate.
* The [byteorder](https://crates.io/crates/byteorder) crate.
* The [bswap](https://crates.io/crates/bswap) crate.

`byteorder` is the prevailing approach (and a great crate), but it tends to push endianness decisions into *parsing logic*. For some codebases, it’s nicer if the endianness is part of the type and your code can stay closer to “ordinary” Rust.

## So, why create another one?

Because the existing crates for handling endianness in Rust require a lot of manual byte mashing, especially if you're trying to write safe code.

This crate aims to make binary formats feel like ordinary Rust types:

* endianness lives in your struct/enum definitions
* the compiler prevents “oops, I wrote native-endian to the wire” mistakes
* optional derive + IO helpers make it practical to build complete protocols and storage formats

That makes it a good fit for packet formats, RPC framing, binary logs, file formats, and any place where a stable representation matters.

## Highlights

This repo includes several runnable examples in `./examples/`.

Notable ones:

* `derive_protocol` / `enum_protocol`: derive-based wire types + `read_specific` / `write_specific`
* `fat16_driver`: a small FAT16 boot-sector / directory walkthrough
* `ethernet_inspector`: an Ethernet II frame inspector that recognizes VLAN/ARP/IPv4/IPv6/TCP/UDP/ICMP

## Performance notes

`simple_endian` is designed to be low-overhead: endian-aware wrappers are `#[repr(transparent)]` and the read/write helpers are optimized to avoid per-value allocations.

There’s a short benchmark-driven writeup (including BE vs LE comparisons and “pure” conversion vs `std::io` overhead) in [`PERFORMANCE.md`](./PERFORMANCE.md).

Note: if you’re formatting these values in hot paths, consider converting to native first (e.g. via `.to_native()`), since formatting overhead can dominate.

## Binary Size Notes

This crate has a lot of functionality. The easiest thing to do is enable all features....and then watch your binary size bloat. Fortunately, it's also designed to scale up and down quite a lot, so that it's suitable both for codebases that need all the features, as well as small lightweight embedded projects.

For that reason, consider it a strong recommendation to use the granular feature flags for what you actually need. If you care about binary size, compile times, or a smaller API surface (especially for `no_std`/embedded), this crate is designed to be “pick what you need”.

Some commonly useful opt-ins:

* `nonzero`: enable `core::num::NonZero*` support (including shorthand aliases like `nzu32be`, `nzi64le`).
* `wrapping`: enable `core::num::Wrapping<T>` support.

Other feature families you may care about:

* `derive`: proc-macro derives for generating wire types.
* `io-core` / `io-std`: endian-aware read/write helpers.
* `text_utf16` / `text_utf32` / `text_fixed` (or `text_all`): fixed-width UTF helpers.

## Goals of this project

The goals of this crate are as follows:

1. Safely provide specific-endian types with low or no runtime overhead. There should be no runtime penalty when the host architecture matches the specified endianness, and very low penalty loads and stores otherwise.
2. Straightforward, architecture-independent declarative syntax which ensures that load and store operations are correct.
3. Ergonomic use patterns that maximize clarity and convenience without sacrificing correctness or safety.
4. Because of the provided classes of operations, many logical, bitwise, and mathematical operations can be performed on the specific-endian types within the crate without explicitly converting to native host endian.
5. Incorrect handling of data should generate clear type errors at compile time.
6. Determination of correct endianness should be at declaration, and should not need to be repeated unless converting to a different endianness.
7. Support for all or Rust's built-in types where endianness is relevant.
8. The only dependency needed is the core crate. The std crate is used, however, for tests and benchmarks, and for some optional features.

## Quick start

### Define endian-aware values

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

### Optional pattern: compute in native, store as endian

While it is possible and often just as fast to do many operations directly and safely on the endianness of the structure, for arithmetic-heavy code, it’s may be clearer (and sometimes faster) to do math in native types and store back.

Either way, the type system ensures that accesses are safe and correct, so this code would be considered optimization:

```rust
use simple_endian::{u16be, BigEndian};

fn add_one(x: u16be) -> u16be {
  let native: u16 = x.to_native();
  (native.wrapping_add(1)).into()
}

// Same idea with the generic wrapper:
fn add_one_generic(x: BigEndian<u16>) -> BigEndian<u16> {
  (x.to_native().wrapping_add(1)).into()
}
```

## How it works

At its core, this crate centers around one trait, `SpecificEndian<T>`, plus the generic wrappers `BigEndian<T>` and `LittleEndian<T>`.

`SpecificEndian<T>` marks a type as safe to store in an endian-tagged wrapper. For primitives, that’s a given; for custom types, you can implement it yourself.

There is no extra memory footprint added by `BigEndian<T>`/`LittleEndian<T>` (they’re `#[repr(transparent)]`); they exist to make endianness explicit and enforce correct reads/writes via the type system.

This crate provides `SpecificEndian` implementations for most of the built-in types in Rust, including:

* Single-byte values (`i8`, `u8`, `bool`), although this really doesn't do much but provide completeness.
* The multi-byte integers: `u16`, `u32`, `u64`, `u128`, `usize`, `i16`, `i32`, `i64`, `i128`, `isize`
* The floats: `f32`, `f64`.

Additionally, `char` is supported (behind the `simple_char_impls` feature, enabled by default via the `simple_all`/default feature set).

Note: even though `char` is supported as an in-memory value, many on-disk / on-wire formats don’t store Rust `char` values directly. When you need a stable binary representation, prefer explicit encodings (e.g. UTF-8 bytes, UTF-16 code units, UTF-32 code points) using the text helpers.

## Derive macro (optional): `derive` Feature

If you enable the `derive` feature, you can generate **wire-format helper types** from a “logical” struct definition. This comes in the form of the `Endianize` macro.

The macro is intentionally conservative:

* It generates a companion `*Wire` struct where fields are endian-wrapped.
* It can generate inline fixed UTF-16/UTF-32 padded fields from `String`/`&str` fields.

It’s designed for “I want a stable on-wire layout, and I want the compiler to help keep it correct.”

```rust
use simple_endian::Endianize;

#[derive(Endianize)]
#[endian(be)]
// Optional: control the generated wire layout. Defaults to repr(C).
// #[wire_repr(packed)]
// Optional: pass additional derives through to the generated *Wire types.
// #[wire_derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Header {
    a: u32,
    b: u16,

    #[text(utf16, units = 8, pad = "space")]
    title: String,
}

// Generated by the derive:
//
// #[repr(C)]
// struct HeaderWire {
//     a: BigEndian<u32>,
//     b: BigEndian<u16>,
//     title: FixedUtf16BeSpacePadded<8>,
// }
let _wire = HeaderWire {
    a: 1u32.into(),
    b: 2u16.into(),
    title: "HI".try_into().unwrap(),
};
```

Notes:

* This currently supports **structs**, **enums**, and **unions**.
* The derive auto-generates common conversions between logical types and their `*Wire` counterparts:
  * `From<Header> for HeaderWire` (for structs without `#[text(..)]` fields)
  * `From<HeaderWire> for Header` (when the struct has no `#[text(..)]` fields)
  * `TryFrom<HeaderWire> for Header` (when the struct contains `#[text(..)]` fields; error type is `FixedTextError`)
* Arrays:
  * Raw byte arrays like `[u8; 8]` are treated as already wire-safe and are passed through unchanged (endianness does not apply to bytes).
  * For other fixed-size arrays, endianness is applied **per element**. For example, under `#[endian(le)]`, a field `words: [u16; 3]` becomes `words: [LittleEndian<u16>; 3]` in the generated `*Wire` type.
* `HeaderWire` is a `#[repr(C)]` “on-wire” type by default, that you can read/write as bytes (often via the IO helpers below).

### Wire layout control: `#[wire_repr(...)]`

By default, `Endianize` generates wire types using `#[repr(C)]`, which can introduce **padding** due to alignment.

If you’re modeling a packed binary format, you can override the representation used for *generated* wire types:

```rust
use simple_endian::Endianize;

#[derive(Endianize)]
#[endian(be)]
#[wire_repr(packed)]
struct PackedHeader {
  a: u8,
  b: u32,
  c: u16,
}

// PackedHeaderWire will be #[repr(packed)] and have no padding.
```

Safety note: `#[repr(packed)]` makes fields potentially **unaligned**. The generated code avoids taking references to packed fields (so it compiles safely), but you should still avoid taking references to packed fields in your own code. Prefer using the IO helpers (`read_specific`/`write_specific`) or copying values out.

### Wire derive pass-through: `#[wire_derive(...)]`

Sometimes you want the generated `*Wire` types to implement extra traits (for example `Debug`, `Copy`, or `PartialEq`).

You can pass these through to the generated wire containers:

```rust
use simple_endian::Endianize;

#[derive(Endianize)]
#[endian(le)]
#[wire_derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Msg {
  id: u16,
  len: u32,
}

// MsgWire derives Clone/Copy/Debug/PartialEq/Eq.
```

### Enum support (tag + payload)

Enums are generated as a stable **tag + payload** wire type:

* The enum must declare `#[repr(u8|u16|u32|u64)]` to select the tag width.
* Unit variants and **named-field** variants like `Variant { a: T, b: U }` are supported.
* Tuple variants like `Variant(T, U)` are supported.
* If the enum has any data-carrying variants, then **all variants must have explicit discriminants** (e.g. `Ping = 1`, `Data { .. } = 2`).

When the `io-std` feature is enabled, the generated `*Wire` enum container also gets `EndianRead`/`EndianWrite` impls.

#### Fixed text in tuple variants: `#[tuple_text(...)]`

Tuple fields don’t have identifiers, so the struct-field `#[text(...)]` attribute can’t be attached to them.
For enum **tuple variants**, you can instead annotate the *variant* with `#[tuple_text(...)]`, selecting a field by index:

```rust
use simple_endian::Endianize;

#[derive(Endianize)]
#[endian(be)]
#[repr(u8)]
enum Msg {
  // Field 0 is a fixed-size UTF-8 wire field.
  #[tuple_text(idx = 0, utf8, units = 8, pad = "null")]
  Name(String, u16) = 1,
}

// Generated (conceptually):
// struct MsgWirePayload_Name(pub FixedUtf8NullPadded<8>, pub BigEndian<u16>);
```

Notes:

* `idx = N` is 0-based.
* You can specify multiple `#[tuple_text(...)]` attributes on the same variant (one per tuple index).
* Supported encodings are `utf8`, `utf16`, `utf32`, with `pad = "null" | "space"`.

### Union support (safe default)

Unions are generated in a **safe default** mode:

* The derive generates a `*Wire` union (`#[repr(C)]`) where each field is endian-wrapped.
* **No IO impls are generated for unions.** A union needs external context (typically a separate tag) to know which field is valid; auto-serializing it would be ambiguous/unsafe.
* `#[text(...)]` is not supported on union fields.

If you need IO for a union-like format, model it as an enum instead (tag + payload), which `Endianize` supports.

## Examples

There are runnable examples in `examples/` (each example is in its own subdirectory as `examples/<name>/main.rs`):

* `endian_values`: store values in `BigEndian<T>`/`LittleEndian<T>`, convert to native, and use arithmetic/bitwise operations.
* `explicit_struct_endian`: define a `#[repr(C)]` struct with explicit endian fields (e.g. `u32be`, `u16le`) and inspect raw bytes.
* `derive_protocol`: a small binary-protocol demo using `#[derive(Endianize)]`, fixed padded text fields, enums (tag + payload), and `io-std` read/write.
  * Requires features: `derive`, `io-std`, `text_all`.
* `enum_protocol`: a framed binary protocol example focusing specifically on `#[derive(Endianize)]` enums (tag + payload).
  * Demonstrates a **multi-byte (u16) discriminator** stored on the wire in a specified endian, and shows why interpreting it with the wrong endian produces the wrong value.
  * Requires features: `derive`, `io-std`, `text_all`.
* `cpu_emulator`: a tiny toy CPU emulator that stores its registers as `BigEndian<u16>` and reads/writes 16-bit words in big-endian byte order.
  * Run with: `cargo run --example cpu_emulator --features "io-std"`
* `fat16_driver`: a tiny FAT16 “driver” that parses a synthetic disk image and prints boot sector + root directory info.
  * Requires features: `derive`, `io-std`, `text_all`.
* `messaging_client` / `messaging_server` (under `examples/messaging_app/`): a more end-to-end demo of designing a small wire protocol and using derive + IO helpers across a client/server boundary.
  * Requires features: `derive`, `io-std`, `text_all`.

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

## Feature flags

Although this crate includes a lot of useful functionality up front, including it all can increase your compiled size significantly. For size-conscious applications, I recommend not including everything.

By default, this crate enables a broad set of convenience features. If you care about binary size or compile time, consider turning defaults off and enabling only what you need.

```toml
[dependencies.simple_endian]
version = "0.4"
default-features = false
features = ["both_endian", "integer_impls"]
```

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

The “simple” feature family enables endianness-invariant built-ins via the `SimpleEndian` trait:

* `simple_bool` – `bool`
* `simple_byte_impls` – `u8`, `i8`
* `simple_char_impls` – `char`
* `simple_string_impls` – `&str`, `String`
* `simple_all` – enables all of the above

### Text helpers (features: `text_utf16`, `text_utf32`, `text_fixed`)

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

This section has moved up to **Performance notes** near the top of the README and is backed by the results in [`PERFORMANCE.md`](./PERFORMANCE.md).

## See Also

This crate allows for the manipulation of specific-endian structures in memory.  It does not provide any facility for reading or writing those structures, which would probably be necessary in most use cases.  See the following other crates for that functionality:

* Rust's standard `std::mem::[transmute](https://doc.rust-lang.org/std/mem/fn.transmute.html)`:
* [safe-transmute](https://crates.io/crates/safe-transmute)
* [endian-trait](https://crates.io/crates/endian_trait)
* [byteordered](https://crates.io/crates/byteordered)
* [persistance](https://crates.io/crates/persistence) - A library that wraps structs in mmap, and can be used well with this to make those structs portable.
* [endian-type](https://crates.io/crates/endian-type) - Essentially the same “typed endianness” idea, but with a different feature set.
* [endian-types](https://crates.io/crates/endian-types) - Another similar “typed” approach.

## Similar tools and when to use them

This crate is a good fit when you want endianness to be *part of the type* and enforced throughout your code.

Depending on your problem, these alternatives may be a better fit:

* [`byteorder`](https://crates.io/crates/byteorder): great for parsing from `&[u8]`/`Read` with explicit read/write calls. Ideal when you don’t want to introduce endian-tagged types into your domain model.
* [`zerocopy`](https://crates.io/crates/zerocopy) and [`bytemuck`](https://crates.io/crates/bytemuck): good when you want safe-ish “view structs as bytes” patterns with strict layout guarantees. `zerocopy` in particular has built-in endian-aware integer wrappers.
* [`binrw`](https://crates.io/crates/binrw), [`scroll`](https://crates.io/crates/scroll), [`nom`](https://crates.io/crates/nom): higher-level parsing frameworks. Often great for file formats where you want declarative parsing and offsets.

If you just need “read a `u32` from bytes” and you don’t need the typed-wrappers approach, start with `byteorder`. If you’re building reusable wire types and want the compiler to keep you honest, `simple_endian` shines.

## IO helpers (features: `io-core`, `io-std`)

This crate provides optional, feature-gated IO helpers for reading and writing endian-aware values directly from `Read`/`Write` streams.

* `io-core` enables the generic machinery (works for `no_std` environments with custom IO traits)
* `io-std` enables `std::io::{Read, Write}` integration
* `io` is a convenience alias for `io-std`

Enable them by adding the `io-std` (or `io`) feature in your `Cargo.toml`:

```toml
[dependencies.simple_endian]
version = "0.4"
features = ["io-std"]
```

With `io-std`, you can use the helper functions:

* `read_specific<R, E>(reader: &mut R) -> io::Result<E>` — Read an endian-wrapped value of type `E` (for example `BigEndian<u32>`) from `reader`.
* `write_specific<W, E>(writer: &mut W, v: &E) -> io::Result<()>` — Write the endian-wrapped value to `writer`.

Additionally, helper traits are provided so types can implement custom read/write behavior:

* `EndianRead` — types implementing this expose `read_from<R: Read>(reader: &mut R) -> io::Result<Self>`.
* `EndianWrite` — types implementing this expose `write_to<W: Write>(&self, writer: &mut W) -> io::Result<()>`.

Big- and Little-endian wrappers implement those traits for the built-in types, so you can use the generic functions like this:

```rust
use simple_endian::{read_specific, write_specific};
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
* Extensive unit tests for the IO helpers are included and run when you enable the IO features.

If you want more realistic demos, check out `derive_protocol`, `enum_protocol` (multi-byte tags), `examples/messaging_app/`, and `fat16_driver`.

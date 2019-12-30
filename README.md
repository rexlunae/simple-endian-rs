# simple-endian

Yet another library for handling endian in Rust.  We use Rust's type system to ensure correct conversions and build data types with explicit endianness defined for more seamless portability of data structures between processor types.

The key difference between this crate and other crates for handling endian is that in this crate, you aren't doing conversions manually at all.  You are just working in the endian that is appropriate to the data structures that you're dealing with, and we try to provide the needed traits and methods to do this in as normal a way as possible.

## Isn't there already a library for this

Yes, there are several.  But I'm not entirely happy with any of them.  Specifically, most of the libraries out there right now focus on providing functions for doing endian conversions.  Here are a few of them:

* https://crates.io/crates/endian
* https://crates.io/crates/byteorder
* https://crates.io/crates/bswap

byteorder has over 11 million downloads, and is clearly the prevailing way to handle endian in Rust.  However, it relies on programmers writing specific logic to swap bytes and requires accessing memory in ways that are unlike normal patterns for Rust.  But really, the only difference between a big- and little-endian value is the interpretation.  It shouldn't require a drastically different pattern of code to access them.

## So, why create another one

Because I think a better approach is to define your endianness as part of your data definition rather than in the logic of your program, and then to make byte order swaps as transparent as possible while still ensuring correctness.  And because the more like normal Rust data types and operations this is, the more likely it is that people will write portable code and data structures in the first place.

The philosophy of this crate is that you define your endian when you write your data structures, and then you use clear, imperative logic to mutate it without needing to think about the details or the host endian.  This makes it fundamentally different from crates that just give you a way to read a &[u8; 8] into a u64.

```rust
use simple_endian::*;

let foo: u64be = 4.into();

println!("raw: {:x}, value: {:x}", foo.to_bits(), foo);
```

The output will depend on what sort of computer you're using.  If you're running a little-endian system, such as x86 (PCs, Macs, etc.), you will see the big endian representation interpreted as if little-endian, as it's stored in memory.  Note that the .to_bits() method is mostly there for debugging purposes, and should not be used often.

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


## Diving in deeper

At its core, this crate centers around one trait, called `SpecificEndian<T>`, and the generic structs `BigEndian<T>` and `LittleEndian<T>`.  `SpecificEndian<T>` is required to make `BigEndian<T>` and `LittleEndian<T>` structs.  Any struct that implements `SpecificEndian`, even if it handles endianness in unusual ways, can be assigned `BigEndian` and `LittleEndian` variants using the structs in this crate.  In fact, `u64be` is just a type alias for `BigEndian<u64>`.  There is no memory footprint added by the `BigEndian<T>` and `LittleEndian<T>` structs, in fact, in most cases it uses the type T to store the data.  The only purpose of the structs is to tag them for Rust's type system to enforce correct accesses.  This means that it can be used directly within larger structs, and then the entire struct can be written to disk, send over a network socket, and otherwise shared between processor architectures using the same code regardless of host endian using declarative logic without any conditionals.

The crate provides SpecificEndian for most of the built-in types in Rust, including:

* Single-byte values (`i8, u8, bool`), although this really doesn't do much but provide completeness.
* The multi-byte integers: `u16, u32, u64, u128, usize, i16, i32, i64, i128, isize`
* The floats: `f32, f64`.

At the time of this writing, the only type that doesn't have an implementation is char, and this is because some values of char that would be possible from the binary representation would cause a panic.  Usually, you wouldn't want to store a char directly anyway, so this is probably a small limitation.

This crate also provides implementations of a variety of useful traits for the types that it wraps, including boolean logic implementations for the integer types.  This allows some amount of logic to be performed directly without byte-swapping overhead.

```rust
use simple_endian::*;

let ip: BigEndian::<u32> = 0x0a00000a.into();
let subnet_mask: BigEndian::<u32> = 0xff000000.into();

let network = ip & subnet_mask;

println!("value: {:x}", network);
```

As you see, the network is calculated by masking the IP address with the subnet mask in a way that the programmer barely has to think about the conversion operations.

Alternatively, you might want to define a structure with the elements typed so that it can be moved around as a unit.

```rust
use simple_endian::*;

#[derive(Debug)]
struct NetworkConfig {
    address: BigEndian<u32>,
    mask: BigEndian<u32>,
    network: BigEndian<u32>,
}

let config = NetworkConfig{address: 0x0a00000a.into(), mask: 0xff000000.into(), network: (0x0a00000a & 0xff000000).into()}

println!("value: {:x?}", config);
```

Note that the println! will interpret the values in native endian.

## Performance

For the most part, the performance of the endian operations are extremely fast, even compared to native operations.  The main exception is the std::fmt implementations, which are in some cases quite a bit slower than default.  I'm open to suggestions on how to improve the performance, but it might be worth using .to_native() instead of directly printing the wrapped types in performance-critical contexts.

## See Also

This crate allows for the manipulation of specific-endian structures in memory.  It does not provide any facility for reading or writing those structures, which would probably be necessary in most use cases.  See the following other crates for that functionality:

* Rust's standard transmute call: https://doc.rust-lang.org/std/mem/fn.transmute.html
* https://crates.io/crates/safe-transmute
* https://crates.io/crates/endian_trait
* https://crates.io/crates/byteordered
* https://crates.io/crates/persistence - A library that wraps structs in mmap, and can be used well with this to make those structs portable.
* https://crates.io/crates/endian-type - The endian-type library.  This appears to be essentially the same approach as this crate, but contains less functionality.
* https://crates.io/crates/endian-types - Another very similar approach.


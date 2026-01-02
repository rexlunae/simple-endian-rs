# Performance notes

This document summarizes the performance characteristics of `simple_endian` features and APIs as of **2026-01-02**, based on local microbenchmarks.

> Benchmarks are always a *directional* signal. Absolute numbers will vary with CPU, Rust version, compiler flags, and surrounding code. What matters most is the *relative* overhead and where it comes from.

## Methodology

Benchmarks live in `benches/benches.rs` and are run via the `bencher` harness (not Criterion).

The results below were collected by running:

```bash
cargo bench --features "derive io-std text_all simple_string_impls"
```

### What each benchmark is measuring

- **scalar_add_{native,le,be}**: repeated integer additions using either native `u32` or endian-wrapped `LittleEndian<u32>` / `BigEndian<u32>`.
- **array_u16_native**: copying a fixed-size `[u16; 32]`.
- **array_u16_le_endianize**: converting `[u16; 32]` → `[LittleEndian<u16>; 32]` → `[u16; 32]`.
- **array_u8_passthrough**: moving a `[u8; 64]` through unchanged.
- **io_write_struct_u16_array**: writing a `#[repr(C)]` derived wire struct containing `[u16; 32]` (element-wise endian) via `write_specific`.
- **io_write_struct_u16_array_le**: same as above, but the array is *little endian on-wire*.
- **io_write_struct_u16_array_be**: same as above, but the array is *big endian on-wire*.
- **io_write_struct_u32_array_le/be**: same idea, but with `[u32; 32]`.
- **io_write_struct_u64_array_le/be**: same idea, but with `[u64; 32]`.
- **io_write_struct_u8_array**: writing a derived wire struct containing `[u8; 64]` (raw bytes) via `write_specific`.
- **core_serialize_array_*_le/be**: “pure” serialization into an in-memory `Vec<u8>` without going through `std::io::Write`.
   This removes syscall/buffering effects and makes the endianness conversion cost more visible.
- **derive_from_logical_to_wire_numeric_only**: `From<Logical> for LogicalWire` on a struct containing scalars + arrays.
- **derive_from_wire_to_logical_numeric_only**: `From<Wire> for Logical` on a struct containing scalars + arrays.
- **derive_tryfrom_wire_to_logical_text**: `TryFrom<Wire> for Logical` for a struct that contains a `#[text(..)] String` field.

## Results (one local run)

| benchmark | time |
| --- | ---: |
| `scalar_add_native` | 52 ns/iter |
| `scalar_add_le` | 52 ns/iter |
| `scalar_add_be` | 263 ns/iter |
| `array_u16_native` | 31 ns/iter |
| `array_u16_le_endianize` | 39 ns/iter |
| `array_u8_passthrough` | 54 ns/iter |
| `derive_from_logical_to_wire_numeric_only` | 16 ns/iter |
| `derive_from_wire_to_logical_numeric_only` | 14 ns/iter |
| `derive_tryfrom_wire_to_logical_text` | 60 ns/iter |
| `io_write_struct_u8_array` | 39 ns/iter |
| `io_write_struct_u16_array_le` | 104 ns/iter |
| `io_write_struct_u16_array_be` | 117 ns/iter |
| `io_write_struct_u32_array_le` | 104 ns/iter |
| `io_write_struct_u32_array_be` | 117 ns/iter |
| `io_write_struct_u64_array_le` | 106 ns/iter |
| `io_write_struct_u64_array_be` | 118 ns/iter |
| `core_serialize_array_u16_le` | 48 ns/iter |
| `core_serialize_array_u16_be` | 88 ns/iter |
| `core_serialize_array_u32_le` | 48 ns/iter |
| `core_serialize_array_u32_be` | 88 ns/iter |
| `core_serialize_array_u64_le` | 58 ns/iter |
| `core_serialize_array_u64_be` | 89 ns/iter |

## Interpreting the results

### 1) Endian wrappers are usually "free" *if you stay in the wrapped domain*

These wrappers are designed so that arithmetic/bitwise ops happen on the host representation (via `SpecificEndian` conversions) while preserving a stable wire representation.

- In this run, `LittleEndian<u32>` arithmetic was the same order as native arithmetic.
- `BigEndian<u32>` arithmetic showed more overhead.

In normal code, this often doesn’t matter because:

- you rarely do tight arithmetic loops over BE-wrapped values, and/or
- you convert at the boundary (read / write), then operate on native numbers.

**Guidance:** prefer converting once at the boundary if doing heavy numeric computation.

### 2) Fixed-size arrays: element-wise endianization is cheap, IO is not

The new derive behavior:

- `[u8; N]` is passed through unchanged.
- `[u16; N]` becomes `[LittleEndian<u16>; N]` (or BE), so endianness applies *per element*.

The microbenchmarks show:

- The *conversion* cost for `[u16; 32]` element-wise wrapping/unwrapping is small (`39 ns` vs `31 ns`).
- The *IO cost* for writing `[u16; 32]` via `write_specific` is now in the same ballpark as the other non-text benches.
- BE vs LE on-wire doesn’t radically change write performance.
   In this run it’s ~10–15% slower for BE across u16/u32/u64 arrays.

Why did `io_write_struct_u16_array_*` used to be expensive?

- The current `EndianWrite for [E; N]` implementation writes element-by-element.
- Previously, each `LittleEndian<u16>` write allocated a tiny `Vec` in `write_le` (via `core_io::write_to_extend`).

That meant 32 elements → 32 small dynamic allocations per call.

That allocation has now been removed for the common primitive wrapper cases (`u16/u32/u64`) in the std-IO fast path, which brings `io_write_struct_u16_array_*` down dramatically.

**Does “host endianness matches the data” matter here?**

Not much.

- On essentially all mainstream Rust targets today, the host is little-endian.
- When you ask for **LE-on-wire**, the conversion step is “cheap” (often a no-op-ish byte order operation).
- When you ask for **BE-on-wire**, there *is* a byte swap.

But for IO microbenches like this, the total cost is a mix of:

- per-element conversion
- per-element trait calls / loop overhead
- the final `Write` target and buffering behavior

So you should expect *some* difference between BE and LE, but not a radical one (especially once the prior allocation footgun is removed).

Also note: in these benches, **u16/u32/u64 arrays in this shape come out quite close to each other**.
That’s a hint that the loop + dispatch + write-to-`Vec<u8>` behavior is dominating over the byte-swap itself.

### 2b) If you remove the std::io layer, the swap cost is much clearer

The `core_serialize_array_*` benches intentionally bypass `std::io::Write` and just build the on-wire bytes in-memory.

In that setup:

- LE-on-wire is materially faster on typical little-endian hosts.
- BE-on-wire pays a per-element byte-swap, and you see that more directly.

This is why the BE-vs-LE spread is larger for `core_serialize_array_*` than for `io_write_struct_*`:

- the IO benches include loop/trait-call overhead and interaction with the write target
- the “pure” benches are closer to measuring just “how expensive is it to swap bytes for N elements?”

**Guidance:**

- Element-wise wrapping is fine.
- Element-wise IO is also fine for primitive arrays now.
- For *very large* arrays or throughput-sensitive pipelines, bulk serialization may still win.

### 3) Raw byte arrays are fast

`io_write_struct_u8_array` is very fast compared to the u16-array path because it’s a single `write_all(&[u8])`.

**Guidance:** keep truly-byte fields as `[u8; N]` (ASCII tokens, opaque identifiers, reserved areas) and parse/validate separately.

### 4) Derive-generated conversions are cheap

For numeric-only structs, `From<Logical> for Wire` and `From<Wire> for Logical` were about ~15ns/iter here.

For text structs, `TryFrom<Wire> for Logical` is higher because it:

- validates text/code unit invariants
- allocates a `String`

**Guidance:**

- For high-frequency paths, keep data in wire form until needed, or decode lazily.
- `#[text(..)]` is best for protocol headers / fixed fields where correctness and clarity matter more than raw throughput.

## Feature-cost overview

Feature flags mostly affect *compile-time* and code size, not runtime hot paths:

- `derive`: adds proc-macro expansion; runtime cost depends on what code you use.
- `io-std`: enables IO helpers and trait impls. The naive per-element array serialization overhead is the main runtime footgun right now.
- `text_*`: adds fixed text types and conversions; conversion runtime cost is dominated by validation + allocation when converting to `String`.

## Practical recommendations

- **Parsing/serialization boundary:** use `read_specific`/`write_specific` for readability and correctness.
- **High-throughput bulk data:** avoid writing thousands of small endian values individually; serialize in bulk.
- **Arrays of primitives:** element-wise wrapping is fine; element-wise IO is currently allocation-heavy.
- **Opaque fields:** keep them as `[u8; N]` and validate separately.

## Possible improvements (future work)

If we want to reduce overhead further:

1. Add true bulk array IO for `[LittleEndian<u16>; N]`, `[BigEndian<u16>; N]`, etc.
   - This would reduce per-element call overhead (even if each element write is now allocation-free).
   - Note: doing this with stack buffers like `[u8; 2*N]` requires generic-const-expr support; a `Vec<u8>` buffer can still be a win.

2. Consider a Criterion-based bench mode for more stable measurements.

---

This document previously recommended implementing an IO optimization for primitive arrays; that optimization is now in place for the std-IO path (no per-element `Vec` allocation for `u16/u32/u64` endian wrappers).

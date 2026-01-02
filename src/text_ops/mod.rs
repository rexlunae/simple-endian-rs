//! Endianness-invariant text/code-unit conversion utilities.
//!
//! This module provides small, **feature-gated** helper types for converting between
//! different Unicode text representations with different *code unit widths*.
//!
//! ## Scope and goals
//!
//! * These types are about **conversion**, not endianness: `u8`, `u16`, and `u32` values
//!   are *numeric* and therefore have a byte order in memory/serialization, but Unicode
//!   text formats define **code units** and their mapping to scalar values.
//! * This crate already provides `BigEndian<T>` / `LittleEndian<T>` wrappers for numeric
//!   types whose in-memory representation changes by endianness.
//! * Unicode conversions are frequent enough that having a small, well-documented set of
//!   helpers here is convenient, but we keep them opt-in via Cargo features.
//!
//! ## Provided conversions
//!
//! The core conversion is: **Rust `str` / `String`  ⇄  UTF-16/UTF-32 code units**.
//!
//! Specifically, we expose:
//!
//! * [`Utf16Str`] – a borrowed view of UTF-16 code units (`&[u16]`).
//! * [`Utf16String`] – an owning UTF-16 buffer (`Vec<u16>`).
//! * [`Utf32Str`] – a borrowed view of UTF-32 code units (`&[u32]`).
//! * [`Utf32String`] – an owning UTF-32 buffer (`Vec<u32>`).
//!
//! and conversions:
//!
//! * `From<&str> for Utf16String` (encode to UTF-16)
//! * `TryFrom<Utf16Str<'_>> for String` and `TryFrom<&Utf16String> for String` (decode)
//! * `From<&str> for Utf32String` (encode to scalar values)
//! * `TryFrom<Utf32Str<'_>> for String` and `TryFrom<&Utf32String> for String` (decode)
//!
//! ## Mapping to `core::str` / `std::str`
//!
//! These helpers intentionally mirror the standard library's string APIs:
//!
//! * **UTF-16 encoding** uses [`str::encode_utf16`](core::primitive::str::encode_utf16), which
//!   yields an iterator of native-endian `u16` code units (the stdlib iterator type is
//!   `core::str::EncodeUtf16<'_>`).
//!
//!   * `Utf16StringLE::from(&str)` and `Utf16StringBE::from(&str)` are thin wrappers around
//!     `s.encode_utf16()`.
//!   * Fixed UTF-16 buffers (`FixedUtf16*`) likewise stream from `encode_utf16()` directly into
//!     their inline `[u16; N]`-shaped storage.
//!
//! * **UTF-16 decoding** uses [`char::decode_utf16`](core::char::decode_utf16), which matches the
//!   stdlib's own decoding logic and error semantics.
//!
//! * **UTF-32 / scalar values** are represented in Rust by iterating Unicode scalar values via
//!   [`str::chars`](core::primitive::str::chars) (an iterator of `char`). When we store UTF-32
//!   code units, we store `char as u32`.
//!
//! A key difference from the stdlib is that our "code unit" buffers are often stored as
//! endian-tagged numeric values (e.g. `BigEndian<u16>` / `LittleEndian<u16>`) so they can be used
//! directly in `#[repr(C)]` structs with a stable on-the-wire byte order.
//!
//! ## Feature flags
//!
//! This module is only compiled when its corresponding Cargo features are enabled:
//!
//! * `text_utf8` – enables UTF-8 helper types.
//! * `text_utf16` – enables UTF-16 helper types.
//! * `text_utf32` – enables UTF-32 helper types.
//! * `text_fixed` – enables fixed-codepoint / fixed-code-unit, inline strings.
//! * `text_all` – convenience feature enabling all of the above.

#[cfg(feature = "text_utf8")]
pub mod utf8;

#[cfg(feature = "text_utf16")]
mod utf16;

#[cfg(feature = "text_utf32")]
mod utf32;

#[cfg(feature = "text_fixed")]
mod fixed;

// `utf8` is a public module; users can access it as `text_ops::utf8::*`.

#[cfg(feature = "text_utf16")]
pub use utf16::*;

#[cfg(feature = "text_utf32")]
pub use utf32::*;

#[cfg(feature = "text_fixed")]
pub use fixed::*;

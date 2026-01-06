//! Proc-macros for [`simple_endian`].
//!
//! You almost never need to depend on this crate directly.
//! Instead, enable the `derive` feature on `simple_endian`:
//!
//! ```toml
//! [dependencies]
//! simple_endian = { version = "0.4", features = ["derive"] }
//! ```
//!
//! That re-exports the [`Endianize`] derive from this crate.
//!
//! ## What `#[derive(Endianize)]` generates
//!
//! For a type like `MyType`, the macro generates:
//!
//! * a `MyTypeWire` companion type with a stable, endian-aware representation suitable for IO
//! * endian wrapper aliases for convenient field typing (e.g. `u32le`, `u16be`) when needed
//!
//! The intended workflow is:
//!
//! * use the native type in your application logic
//! * read/write the wire type (or use the native-first helpers in `simple_endian::io`)
//!
//! ## Supported helper attributes
//!
//! Container-level:
//!
//! * `#[endian(le)]` / `#[endian(be)]` (required)
//! * `#[wire_repr(...)]` to control the generated wire layout (`#[repr(C)]`, `#[repr(C, packed)]`, etc.)
//! * `#[wire_derive(...)]` to add derives to the generated wire type
//! * `#[wire_default]` / `#[wire_default(...)]` to control wire `Default` generation
//!
//! Field-level:
//!
//! * `#[text(...)]` for fixed-size text fields
//! * `#[tuple_text]` for tuple enum variants
//!
//! ## Important limitation: enum wire derives
//!
//! Enum wire types are represented as `tag + union payload` in order to match on-wire layouts.
//! Unions cannot derive many common traits on stable Rust (notably `Debug`, `PartialEq`, `Eq`, `Hash`, ...).
//! If you use `#[wire_derive(...)]` on an enum, keep that in mind.
//!
//! In practice, it's best to operate on the native enum in your code and only convert at IO boundaries.
//! See the `simple_endian` README for the recommended "native-first" pattern.

use proc_macro::TokenStream;

mod endianize;

/// Generate a “wire-format” struct and endian wrapper aliases.
///
/// This derive requires a container-level endianness annotation:
///
/// * `#[endian(le)]` for little-endian
/// * `#[endian(be)]` for big-endian
///
/// See the `simple_endian` crate documentation and README for examples and the recommended workflow.
#[proc_macro_derive(Endianize, attributes(endian, text, tuple_text, wire_repr, wire_derive, default, wire_default))]
pub fn derive_endianize(input: TokenStream) -> TokenStream {
    endianize::derive_endianize(input)
}

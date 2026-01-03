//! Proc-macros for `simple_endian`.
//!
//! This crate is intentionally small and only provides codegen helpers.

use proc_macro::TokenStream;

mod endianize;

/// Generate a “wire-format” struct and endian wrapper aliases.
///
/// See the `simple_endian` crate docs for examples.
#[proc_macro_derive(Endianize, attributes(endian, text, tuple_text, wire_repr, wire_derive))]
pub fn derive_endianize(input: TokenStream) -> TokenStream {
    endianize::derive_endianize(input)
}

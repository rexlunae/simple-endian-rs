//! Proc-macros for `simple_endian`.
//!
//! This crate is intentionally small and only provides codegen helpers.

use proc_macro::TokenStream;

mod endianize;

/// Generate a “wire-format” struct and endian wrapper aliases.
///
/// See the `simple_endian` crate docs for examples.
#[proc_macro_derive(Endianize, attributes(endian, text, wire_repr))]
pub fn derive_endianize(input: TokenStream) -> TokenStream {
    endianize::derive_endianize(input)
}

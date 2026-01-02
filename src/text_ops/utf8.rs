//! UTF-8 helpers.
//!
//! This module exists to parallel the UTF-16/UTF-32 helpers, but UTF-8 code units
//! are just bytes (`u8`) and have no endianness.
//!
//! The main motivation is fixed-size on-wire fields that are stored as UTF-8 bytes
//! with explicit padding rules.

#[cfg(feature = "text_fixed")]
pub mod fixed {
	pub use crate::text_ops::fixed::utf8::*;
}

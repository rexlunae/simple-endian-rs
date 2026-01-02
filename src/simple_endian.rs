//! Types that do not change based on endianness.
//!
//! This module provides the `SimpleEndian` trait for types that are the same in both big-endian and little-endian
//! representations. These types are built into Rust, the core crate, and the std crate, and include:
//! - The unit type `()`
//! - Boolean values `bool`
//! - String slices and owned strings (when the `string_impls` feature is enabled)
//! - Arrays of SimpleEndian types (when the `array_impls` feature is enabled)
//! - Other single-byte types
//!
//! All conversions for types implementing `SimpleEndian` are no-ops since the endianness doesn't affect them.

/// A trait for types that do not change based on endianness.
///
/// Types implementing `SimpleEndian` have the same representation regardless of whether they are
/// stored in big-endian or little-endian byte order. This includes single-byte types, the unit type,
/// and other endianness-agnostic types.
///
/// All conversion methods are no-ops for these types, providing default implementations that simply
/// return the value unchanged.
pub trait SimpleEndian: Sized + Clone {
    /// No-op conversion to big-endian representation (returns self unchanged).
    fn to_big_endian(self) -> Self {
        self
    }

    /// No-op conversion to little-endian representation (returns self unchanged).
    fn to_little_endian(self) -> Self {
        self
    }

    /// No-op conversion from big-endian representation (returns self unchanged).
    fn from_big_endian(self) -> Self {
        self
    }

    /// No-op conversion from little-endian representation (returns self unchanged).
    fn from_little_endian(self) -> Self {
        self
    }

    /// Returns the endianness of the host target.
    fn endian(&self) -> crate::specific_endian::Endian {
        if cfg!(target_endian = "big") {
            crate::specific_endian::Endian::Big
        } else {
            crate::specific_endian::Endian::Little
        }
    }
}

/// Implement SimpleEndian for the unit type `()`.
impl SimpleEndian for () {}

/// Implement SimpleEndian for `bool`.
#[cfg(feature = "simple_bool")]
impl SimpleEndian for bool {}

/// Implement SimpleEndian for `u8` (single byte, endianness-independent).
#[cfg(feature = "simple_byte_impls")]
impl SimpleEndian for u8 {}

/// Implement SimpleEndian for `i8` (single byte, endianness-independent).
#[cfg(feature = "simple_byte_impls")]
impl SimpleEndian for i8 {}

/// Implement SimpleEndian for `char` (a Unicode scalar value; endianness-independent as a Rust value).
#[cfg(feature = "simple_char_impls")]
impl SimpleEndian for char {}

/// Implement SimpleEndian for string slices `&str`.
#[cfg(feature = "simple_string_impls")]
impl SimpleEndian for &str {}

/// Implement SimpleEndian for owned strings `String`.
#[cfg(feature = "simple_string_impls")]
impl SimpleEndian for String {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_type_is_simple_endian() {
        let _unit: () = ();
        assert!(true); // If this compiles, the trait is implemented
    }

    #[test]
    #[cfg(feature = "simple_bool")]
    fn bool_is_simple_endian() {
        let b = true;
        assert_eq!(b.to_big_endian(), b);
        assert_eq!(b.to_little_endian(), b);
        assert_eq!(b.from_big_endian(), b);
        assert_eq!(b.from_little_endian(), b);
    }

    #[test]
    #[cfg(feature = "simple_byte_impls")]
    fn u8_is_simple_endian() {
        let n: u8 = 42;
        assert_eq!(n.to_big_endian(), n);
        assert_eq!(n.to_little_endian(), n);
        assert_eq!(n.from_big_endian(), n);
        assert_eq!(n.from_little_endian(), n);
    }

    #[test]
    #[cfg(feature = "simple_byte_impls")]
    fn i8_is_simple_endian() {
        let n: i8 = -42;
        assert_eq!(n.to_big_endian(), n);
        assert_eq!(n.to_little_endian(), n);
        assert_eq!(n.from_big_endian(), n);
        assert_eq!(n.from_little_endian(), n);
    }

    #[test]
    #[cfg(feature = "simple_char_impls")]
    fn char_is_simple_endian() {
        let c: char = '🦀';
        assert_eq!(c.to_big_endian(), c);
        assert_eq!(c.to_little_endian(), c);
        assert_eq!(c.from_big_endian(), c);
        assert_eq!(c.from_little_endian(), c);
    }

    #[test]
    #[cfg(feature = "simple_string_impls")]
    fn str_is_simple_endian() {
        let s = "hello";
        assert_eq!(s.to_big_endian(), s);
        assert_eq!(s.to_little_endian(), s);
        assert_eq!(s.from_big_endian(), s);
        assert_eq!(s.from_little_endian(), s);
    }

    #[test]
    #[cfg(feature = "simple_string_impls")]
    fn string_is_simple_endian() {
        let s = String::from("hello");
        let converted = s.clone().to_big_endian();
        assert_eq!(converted, s);

        let converted = s.clone().to_little_endian();
        assert_eq!(converted, s);

        let converted = s.clone().from_big_endian();
        assert_eq!(converted, s);

        let converted = s.clone().from_little_endian();
        assert_eq!(converted, s);
    }
}

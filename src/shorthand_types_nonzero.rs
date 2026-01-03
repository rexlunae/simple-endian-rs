#![cfg(feature = "nonzero")]

/*!
NonZero shorthand type names.

These follow the pattern:

- `nzu<bits><le|be>` for `NonZeroU*`
- `nzi<bits><le|be>` for `NonZeroI*`
- `nzusize<le|be>` / `nzisize<le|be>` for the pointer-sized variants

All are gated behind the `nonzero` feature.
*/

#![allow(non_camel_case_types)]

use core::num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
};

use crate::{BigEndian, LittleEndian};

/// Shorthand for `LittleEndian<NonZeroU8>`
pub type nzu8le = LittleEndian<NonZeroU8>;
/// Shorthand for `BigEndian<NonZeroU8>`
pub type nzu8be = BigEndian<NonZeroU8>;

/// Shorthand for `LittleEndian<NonZeroU16>`
pub type nzu16le = LittleEndian<NonZeroU16>;
/// Shorthand for `BigEndian<NonZeroU16>`
pub type nzu16be = BigEndian<NonZeroU16>;

/// Shorthand for `LittleEndian<NonZeroU32>`
pub type nzu32le = LittleEndian<NonZeroU32>;
/// Shorthand for `BigEndian<NonZeroU32>`
pub type nzu32be = BigEndian<NonZeroU32>;

/// Shorthand for `LittleEndian<NonZeroU64>`
pub type nzu64le = LittleEndian<NonZeroU64>;
/// Shorthand for `BigEndian<NonZeroU64>`
pub type nzu64be = BigEndian<NonZeroU64>;

/// Shorthand for `LittleEndian<NonZeroU128>`
pub type nzu128le = LittleEndian<NonZeroU128>;
/// Shorthand for `BigEndian<NonZeroU128>`
pub type nzu128be = BigEndian<NonZeroU128>;

/// Shorthand for `LittleEndian<NonZeroUsize>`
pub type nzusizele = LittleEndian<NonZeroUsize>;
/// Shorthand for `BigEndian<NonZeroUsize>`
pub type nzusizebe = BigEndian<NonZeroUsize>;

/// Shorthand for `LittleEndian<NonZeroI8>`
pub type nzi8le = LittleEndian<NonZeroI8>;
/// Shorthand for `BigEndian<NonZeroI8>`
pub type nzi8be = BigEndian<NonZeroI8>;

/// Shorthand for `LittleEndian<NonZeroI16>`
pub type nzi16le = LittleEndian<NonZeroI16>;
/// Shorthand for `BigEndian<NonZeroI16>`
pub type nzi16be = BigEndian<NonZeroI16>;

/// Shorthand for `LittleEndian<NonZeroI32>`
pub type nzi32le = LittleEndian<NonZeroI32>;
/// Shorthand for `BigEndian<NonZeroI32>`
pub type nzi32be = BigEndian<NonZeroI32>;

/// Shorthand for `LittleEndian<NonZeroI64>`
pub type nzi64le = LittleEndian<NonZeroI64>;
/// Shorthand for `BigEndian<NonZeroI64>`
pub type nzi64be = BigEndian<NonZeroI64>;

/// Shorthand for `LittleEndian<NonZeroI128>`
pub type nzi128le = LittleEndian<NonZeroI128>;
/// Shorthand for `BigEndian<NonZeroI128>`
pub type nzi128be = BigEndian<NonZeroI128>;

/// Shorthand for `LittleEndian<NonZeroIsize>`
pub type nzisizele = LittleEndian<NonZeroIsize>;
/// Shorthand for `BigEndian<NonZeroIsize>`
pub type nzisizebe = BigEndian<NonZeroIsize>;

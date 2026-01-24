//! Optional `encoding_rs`-backed helpers.
//!
//! These functions are intended for wire formats that store *bytes* in a fixed-size field,
//! but interpret those bytes using a non-UTF8 encoding (e.g. Windows-1252, Shift_JIS).
//!
//! Design goals:
//! - Keep the core crate `no_std` friendly: this module is entirely feature-gated.
//! - Be strict by default: error on malformed sequences and on data that doesn't fit.
//! - Preserve `simple_endian`'s fixed-field padding conventions.

#![cfg(feature = "text_encoding_rs")]

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use core::fmt;

use encoding_rs::Encoding;
use encoding_rs::{DecoderResult, EncoderResult};

use crate::{FixedUtf8Bytes, FixedUtf8NullPadded, FixedUtf8SpacePadded};

/// Errors returned by the `encoding_rs` helpers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodingRsError {
    /// Encoded/decoded data would exceed the fixed field size.
    TooManyBytes { max: usize, found: usize },
    /// Input bytes were not valid for the selected encoding.
    MalformedInput,
}

impl fmt::Display for EncodingRsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodingRsError::TooManyBytes { max, found } => {
                write!(f, "encoded text too long (max {max} bytes, found {found})")
            }
            EncodingRsError::MalformedInput => write!(f, "malformed input for encoding"),
        }
    }
}

fn trim_null(bytes: &[u8]) -> &[u8] {
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == 0 {
        end -= 1;
    }
    &bytes[..end]
}

fn trim_space(bytes: &[u8]) -> &[u8] {
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == b' ' {
        end -= 1;
    }
    &bytes[..end]
}

/// Decode a NUL-padded fixed byte field using the provided `encoding_rs` encoding.
///
/// This is strict: it returns an error if the byte sequence is malformed.
pub fn decode_null_padded<const N: usize>(
    encoding: &'static Encoding,
    v: &FixedUtf8NullPadded<N>,
) -> Result<String, EncodingRsError> {
    let bytes = trim_null(&v.0.bytes);
    // Allocate enough UTF-8 bytes for the worst case. This keeps the API strict and simple.
    let mut out = vec![0u8; bytes.len().saturating_mul(3).saturating_add(16)];
    let mut decoder = encoding.new_decoder_without_bom_handling();
    let (result, _read, written) =
        decoder.decode_to_utf8_without_replacement(bytes, &mut out, true);

    match result {
        DecoderResult::InputEmpty => {
            out.truncate(written);
            // Safety: encoding_rs guarantees produced output is valid UTF-8.
            Ok(unsafe { String::from_utf8_unchecked(out) })
        }
        DecoderResult::OutputFull => {
            // Our buffer sizing should make this effectively unreachable.
            Err(EncodingRsError::MalformedInput)
        }
        DecoderResult::Malformed(_, _) => Err(EncodingRsError::MalformedInput),
    }
}

/// Decode a space-padded fixed byte field using the provided `encoding_rs` encoding.
///
/// This is strict: it returns an error if the byte sequence is malformed.
pub fn decode_space_padded<const N: usize>(
    encoding: &'static Encoding,
    v: &FixedUtf8SpacePadded<N>,
) -> Result<String, EncodingRsError> {
    let bytes = trim_space(&v.0.bytes);
    let mut out = vec![0u8; bytes.len().saturating_mul(3).saturating_add(16)];
    let mut decoder = encoding.new_decoder_without_bom_handling();
    let (result, _read, written) =
        decoder.decode_to_utf8_without_replacement(bytes, &mut out, true);

    match result {
        DecoderResult::InputEmpty => {
            out.truncate(written);
            Ok(unsafe { String::from_utf8_unchecked(out) })
        }
        DecoderResult::OutputFull => Err(EncodingRsError::MalformedInput),
        DecoderResult::Malformed(_, _) => Err(EncodingRsError::MalformedInput),
    }
}

/// Encode a Rust `&str` into a fixed NUL-padded byte field using the provided `encoding_rs` encoding.
///
/// This is strict:
/// - returns an error if the encoder reports malformed input
/// - returns an error if the encoded bytes don't fit the field
pub fn encode_null_padded<const N: usize>(
    encoding: &'static Encoding,
    s: &str,
) -> Result<FixedUtf8NullPadded<N>, EncodingRsError> {
    // Worst-case byte expansion is encoding-specific; allocate a conservative upper bound.
    // (Most legacy encodings are <= 2 bytes per scalar, but we size for up to 4.)
    let mut tmp = vec![0u8; s.len().saturating_mul(4).saturating_add(16)];
    let mut encoder = encoding.new_encoder();
    let (result, _read, written) = encoder.encode_from_utf8_without_replacement(s, &mut tmp, true);

    match result {
        EncoderResult::InputEmpty => {
            if written > N {
                return Err(EncodingRsError::TooManyBytes {
                    max: N,
                    found: written,
                });
            }
            let mut out = [0u8; N];
            out[..written].copy_from_slice(&tmp[..written]);
            Ok(FixedUtf8NullPadded(FixedUtf8Bytes { bytes: out }))
        }
        EncoderResult::OutputFull => Err(EncodingRsError::TooManyBytes {
            max: N,
            found: written,
        }),
        EncoderResult::Unmappable(_) => Err(EncodingRsError::MalformedInput),
    }
}

/// Encode a Rust `&str` into a fixed space-padded byte field using the provided `encoding_rs` encoding.
///
/// This is strict:
/// - returns an error if the encoder reports malformed input
/// - returns an error if the encoded bytes don't fit the field
pub fn encode_space_padded<const N: usize>(
    encoding: &'static Encoding,
    s: &str,
) -> Result<FixedUtf8SpacePadded<N>, EncodingRsError> {
    let mut tmp = vec![0u8; s.len().saturating_mul(4).saturating_add(16)];
    let mut encoder = encoding.new_encoder();
    let (result, _read, written) = encoder.encode_from_utf8_without_replacement(s, &mut tmp, true);

    match result {
        EncoderResult::InputEmpty => {
            if written > N {
                return Err(EncodingRsError::TooManyBytes {
                    max: N,
                    found: written,
                });
            }
            let mut out = [b' '; N];
            out[..written].copy_from_slice(&tmp[..written]);
            Ok(FixedUtf8SpacePadded(FixedUtf8Bytes { bytes: out }))
        }
        EncoderResult::OutputFull => Err(EncodingRsError::TooManyBytes {
            max: N,
            found: written,
        }),
        EncoderResult::Unmappable(_) => Err(EncodingRsError::MalformedInput),
    }
}

//! Messaging app protocol definitions.
//!
//! This example is intentionally **safe Rust only**.
//!
//! Design notes:
//! - We avoid derived enum wires here, because the enum wire format uses a union
//!   internally and would require `unsafe` to read the active payload.
//! - Instead we use a simple fixed header + fixed-size payloads and a
//!   `msg_type` tag.
//! - All wire structs are generated with `#[repr(C)]` by `#[derive(Endianize)]`
//!   and are readable/writable via `simple_endian::io::{read_specific, write_specific}`.

#![allow(dead_code)]

use simple_endian::Endianize;

pub const USERNAME_UNITS: usize = 16;
pub const TEXT_UNITS: usize = 64;

/// Message type tags (what `FrameWire::msg_type` contains).
///
/// These are raw numbers on the wire. We keep them as constants to keep the
/// example simple.
pub mod msg_type {
    pub const HELLO: u16 = 1;
    pub const CHAT: u16 = 2;
    pub const QUIT: u16 = 3;
    pub const SERVER_MSG: u16 = 4;
    pub const ERROR: u16 = 5;
}

/// Common frame header.
///
/// `len_bytes` is the total frame length in bytes (header + payload).
#[derive(Endianize, Clone, Copy, Debug, Default)]
#[endian(be)]
pub struct FrameHeader {
    pub msg_type: u16,
    pub len_bytes: u16,
    pub request_id: u32,
}

/// HELLO payload: client introduces itself.
#[derive(Endianize, Clone, Debug, Default)]
#[endian(be)]
pub struct Hello {
    #[text(utf32, units = 16, pad = "space")]
    pub username: String,
}

/// CHAT payload: client sends a message.
#[derive(Endianize, Clone, Debug, Default)]
#[endian(be)]
pub struct Chat {
    #[text(utf32, units = 16, pad = "space")]
    pub from: String,

    #[text(utf32, units = 64, pad = "space")]
    pub text: String,
}

/// QUIT payload: client disconnects.
#[derive(Endianize, Clone, Copy, Debug, Default)]
#[endian(be)]
pub struct Quit {
    pub reason_code: u16,
    pub _reserved: u16,
}

/// SERVER_MSG payload: server sends informational text.
#[derive(Endianize, Clone, Debug, Default)]
#[endian(be)]
pub struct ServerMsg {
    #[text(utf32, units = 64, pad = "space")]
    pub text: String,
}

/// ERROR payload: server errors.
#[derive(Endianize, Clone, Debug, Default)]
#[endian(be)]
pub struct ErrorMsg {
    pub code: u16,
    pub _reserved: u16,

    #[text(utf32, units = 64, pad = "space")]
    pub message: String,
}

// Wire field visibility: the derive generates `*Wire` structs in this module.
// We want the examples to be able to inspect scalar tags and decode fixed UTF
// payloads **without unsafe**, so we keep the logical structs ergonomic
// (String fields) but we also make the generated wire fields public.
//
// The simplest way is to define the logical structs with `pub` fields (above)
// and rely on the derive macro generating `pub` fields for the wire structs.

impl FrameHeader {
    pub fn with_len(msg_type: u16, request_id: u32, len_bytes: u16) -> Self {
        Self {
            msg_type,
            len_bytes,
            request_id,
        }
    }
}

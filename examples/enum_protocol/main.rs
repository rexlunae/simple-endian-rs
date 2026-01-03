//! Example: a tiny framed binary protocol using `#[derive(Endianize)]` with an enum.
//!
//! This demonstrates:
//! - generating `*Wire` types from logical definitions
//! - an enum encoded as a stable tag + payload (union)
//! - fixed-size padded UTF-16 fields via `#[text(...)]`
//! - reading/writing using `read_specific`/`write_specific`
//!
//! Run with:
//!
//! ```sh
//! cargo run --example enum_protocol --features "derive io-std text_all"
//! ```

#![cfg_attr(
    not(all(feature = "derive", feature = "io-std", feature = "text_all")),
    allow(dead_code, unused_imports)
)]

#[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
mod demo {
    use simple_endian::{Endianize, FixedUtf16BeSpacePadded, read_specific, write_specific};

    /// A tiny frame header.
    ///
    /// The wire format is always big-endian.
    #[allow(dead_code)]
    #[derive(Endianize, Debug)]
    #[endian(be)]
    #[repr(C)]
    struct FrameHeader {
        magic: u32,
        version: u8,
        // Reserved for future flags.
        flags: u8,
        // Payload length in bytes (not including header).
        len: u16,
    }

    /// Commands: encoded as (tag + payload-union).
    ///
    /// Notes:
    /// - `#[repr(u16)]` selects a **16-bit** tag width
    /// - with `#[endian(be)]` the tag is written/read as **big-endian** on the wire
    /// - data-carrying variants must have explicit discriminants
    /// - tuple variants are not supported by the derive
    #[allow(dead_code)]
    #[derive(Endianize, Debug)]
    #[endian(be)]
    #[repr(u16)]
    enum Command {
        Nop = 0,
        Ping {
            nonce: u32,
        } = 1,
        SetName {
            #[text(utf16, units = 12, pad = "space")]
            name: String,
        } = 2,
        Add {
            a: u16,
            b: u16,
        } = 3,
    }

    /// A full frame: header + command.
    fn encode(cmd: CommandWire) -> Vec<u8> {
        // Serialize cmd first so we can fill in header.len.
        let mut payload = Vec::new();
        write_specific(&mut payload, &cmd).unwrap();

        let hdr = FrameHeaderWire {
            magic: 0x5345_4E44u32.into(), // "SEND"
            version: 1u8.into(),
            flags: 0u8.into(),
            len: (payload.len() as u16).into(),
        };

        let mut out = Vec::new();
        write_specific(&mut out, &hdr).unwrap();
        out.extend_from_slice(&payload);
        out
    }

    fn handle_one(hdr: &FrameHeaderWire, cmd: &CommandWire) {
        assert_eq!(hdr.magic.to_native(), 0x5345_4E44);
        assert_eq!(hdr.version.to_native(), 1);

        let tag = cmd.tag.to_native();
        println!("decoded tag: {tag}");

        match tag {
            0 => println!("Nop"),
            1 => {
                // SAFETY: tag selects active union field.
                let p = unsafe { &cmd.payload.Ping };
                println!("Ping.nonce = {}", p.nonce.to_native());
            }
            2 => {
                // SAFETY: tag selects active union field.
                let p = unsafe { &cmd.payload.SetName };
                let got = String::try_from(&p.name).unwrap();
                println!("SetName.name = {got}");

                let expected: FixedUtf16BeSpacePadded<12> = "ALICE".try_into().unwrap();
                // This is just a demo assertion for the example's first SetName.
                if got == "ALICE" {
                    assert_eq!(p.name, expected);
                }
            }
            3 => {
                // SAFETY: tag selects active union field.
                let p = unsafe { &cmd.payload.Add };
                let sum = p.a.to_native() as u32 + p.b.to_native() as u32;
                println!("Add: {} + {} = {}", p.a.to_native(), p.b.to_native(), sum);
            }
            _ => {
                // Forward-compatible behavior: we don't know how to interpret the payload,
                // but we can still use the header length to skip it.
                println!(
                    "unknown tag {tag}; skipping {} payload bytes",
                    hdr.len.to_native()
                );
            }
        }
    }

    pub fn run() {
        println!("=== enum protocol demo ===\n");

        // Encode a tiny stream containing multiple frames.
        let mut stream = Vec::new();

        let set_name = CommandWire {
            tag: 2u16.into(),
            payload: CommandWirePayload {
                SetName: std::mem::ManuallyDrop::new(CommandWirePayload_SetName {
                    name: "ALICE".try_into().unwrap(),
                }),
            },
        };
        stream.extend_from_slice(&encode(set_name));

        let add = CommandWire {
            tag: 3u16.into(),
            payload: CommandWirePayload {
                Add: std::mem::ManuallyDrop::new(CommandWirePayload_Add {
                    a: 10u16.into(),
                    b: 32u16.into(),
                }),
            },
        };
        stream.extend_from_slice(&encode(add));

        // An unknown-tag frame (pretend it's from a newer version of the protocol).
        // We still include a header with a length so receivers can skip it.
        //
        // IMPORTANT: our command tag is `#[repr(u16)]` with `#[endian(be)]`, so the
        // first two payload bytes are the big-endian discriminator.
        let unknown_hdr = FrameHeaderWire {
            magic: 0x5345_4E44u32.into(),
            version: 1u8.into(),
            flags: 0u8.into(),
            // 2 bytes tag + 5 bytes unknown payload.
            len: 7u16.into(),
        };
        let mut unknown_payload = Vec::new();
        write_specific(&mut unknown_payload, &unknown_hdr).unwrap();
        // Unknown tag = 0xFE01 (BE).
        unknown_payload.extend_from_slice(&[0xFE, 0x01]);
        unknown_payload.extend_from_slice(&[1, 2, 3, 4, 5]);
        stream.extend_from_slice(&unknown_payload);

        println!("encoded stream {} bytes\n", stream.len());

        // Decode the stream.
        let mut cur = std::io::Cursor::new(stream);
        while (cur.position() as usize) < cur.get_ref().len() {
            let hdr: FrameHeaderWire = read_specific(&mut cur).unwrap();
            let payload_len = hdr.len.to_native() as usize;

            // Read *exactly* the payload bytes for this frame.
            let mut payload = vec![0u8; payload_len];
            std::io::Read::read_exact(&mut cur, &mut payload).unwrap();

            // Try to parse the payload as a CommandWire. If it fails, treat it as
            // forward-compat/unknown.
            let mut pcur = std::io::Cursor::new(&payload);
            match read_specific::<_, CommandWire>(&mut pcur) {
                Ok(cmd) => handle_one(&hdr, &cmd),
                Err(_) => {
                    // Tag is 16-bit BE on the wire.
                    let (tag, raw) = if payload.len() >= 2 {
                        (
                            u16::from_be_bytes([payload[0], payload[1]]),
                            [payload[0], payload[1]],
                        )
                    } else {
                        (0, [0, 0])
                    };
                    println!(
                        "(unknown frame) tag raw bytes: {:02X} {:02X} (BE)",
                        raw[0], raw[1]
                    );
                    let tag_if_le = u16::from_le_bytes(raw);
                    println!(
                        "(contrast, WRONG for this protocol) same bytes as LE u16: {tag_if_le}"
                    );
                    println!("decoded tag: {tag}");
                    println!("unknown tag {tag}; skipping {payload_len} payload bytes");
                }
            }

            println!();
        }

        println!("\nOK");
    }
}

fn main() {
    #[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
    demo::run();

    #[cfg(not(all(feature = "derive", feature = "io-std", feature = "text_all")))]
    eprintln!(
        "This example requires features: derive, io-std, text_all\n\n  cargo run --example enum_protocol --features \"derive io-std text_all\""
    );
}

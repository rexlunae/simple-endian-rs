//! Example: using `#[derive(Endianize)]` to define a small binary protocol.
//!
//! This demonstrates:
//! - generating `*Wire` types from logical definitions
//! - fixed-size padded UTF-16 fields via `#[text(...)]`
//! - enum tag+payload wire format
//! - reading/writing using the `io-std` helpers (`read_specific`/`write_specific`)
//!
//! Run with:
//!
//! ```sh
//! cargo run --example derive_protocol --features "derive io-std text_all"
//! ```

#![cfg_attr(
    not(all(feature = "derive", feature = "io-std", feature = "text_all")),
    allow(dead_code, unused_imports)
)]

#[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
mod demo {
    use simple_endian::{Endianize, FixedUtf16BeSpacePadded, read_specific, write_specific};

    #[derive(Endianize, Debug)]
    #[endian(be)]
    #[repr(C)]
    #[allow(dead_code)]
    struct Packet {
        // Normal fields become BigEndian<..> in PacketWire.
        msg_id: u32,

        // A fixed-size, padded text field in the wire struct.
        #[text(utf16, units = 12, pad = "space")]
        user: String,
    }

    #[derive(Endianize, Debug)]
    #[endian(be)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum Command {
        // For enums with payload, all variants must have discriminants.
        Ping = 1,
        SetValue { key: u16, value: u32 } = 2,
    }

    pub fn run() {
        println!("=== derive-based protocol demo ===\n");

        // Build a packet in wire form.
        let pkt = PacketWire {
            msg_id: 0x1122_3344u32.into(),
            user: "alice".try_into().unwrap(),
        };

        // Build a command in wire form.
        let cmd = CommandWire {
            tag: 2u8.into(),
            payload: CommandWirePayload {
                SetValue: std::mem::ManuallyDrop::new(CommandWirePayload_SetValue {
                    key: 0x0042u16.into(),
                    value: 0xDEAD_BEEFu32.into(),
                }),
            },
        };

        // Serialize both to a buffer.
        let mut buf = Vec::new();
        write_specific(&mut buf, &pkt).unwrap();
        write_specific(&mut buf, &cmd).unwrap();

        println!("encoded {} bytes", buf.len());

        // Deserialize.
        let mut cur = std::io::Cursor::new(buf);
        let pkt2: PacketWire = read_specific(&mut cur).unwrap();
        let cmd2: CommandWire = read_specific(&mut cur).unwrap();

        println!("decoded pkt msg_id: 0x{:08x}", pkt2.msg_id.to_native());

        let expected_user: FixedUtf16BeSpacePadded<12> = "alice".try_into().unwrap();
        assert_eq!(pkt2.user, expected_user);

        println!("decoded cmd tag: {}", cmd2.tag.to_native());

        // SAFETY: tag selects active union field.
        let payload = unsafe { &cmd2.payload.SetValue };
        println!("decoded SetValue.key: 0x{:04x}", payload.key.to_native());
        println!(
            "decoded SetValue.value: 0x{:08x}",
            payload.value.to_native()
        );

        println!("\nOK");
    }
}

fn main() {
    #[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
    demo::run();

    #[cfg(not(all(feature = "derive", feature = "io-std", feature = "text_all")))]
    eprintln!(
        "This example requires features: derive, io-std, text_all\n\n  cargo run --example derive_protocol --features \"derive io-std text_all\""
    );
}

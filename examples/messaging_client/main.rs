//! Messaging app example (client).
//!
//! See `examples/messaging_app/README.md` for how to run the server and client.

#![cfg_attr(
    not(all(feature = "derive", feature = "io-std", feature = "text_all")),
    allow(dead_code, unused_imports)
)]

#[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
#[path = "../messaging_app/protocol.rs"]
mod protocol;

#[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
mod real {
    use super::protocol::*;
    use simple_endian::{read_specific, write_specific};
    use simple_endian::FixedUtf32BeSpacePadded;
    use std::io::Write;
    use std::net::TcpStream;

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let addr = std::env::args()
            .nth(1)
            .unwrap_or_else(|| "127.0.0.1:7777".to_string());

        let user = std::env::args().nth(2).unwrap_or_else(|| "alice".to_string());
        let message = std::env::args()
            .nth(3)
            .unwrap_or_else(|| "hello from client".to_string());

        let mut stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;

        // 1) Send HELLO frame header + payload.
        let hello_payload = HelloWire {
            username: user.as_str().try_into()?,
        };
        let hello_header = FrameHeaderWire {
            msg_type: msg_type::HELLO.into(),
            len_bytes: ((std::mem::size_of::<FrameHeaderWire>() + std::mem::size_of::<HelloWire>())
                as u16)
                .into(),
            request_id: 1u32.into(),
        };
        write_specific(&mut stream, &hello_header)?;
        write_specific(&mut stream, &hello_payload)?;
        stream.flush()?;

        // 2) Send CHAT frame header + payload.
        let chat_payload = ChatWire {
            from: user.as_str().try_into()?,
            text: message.as_str().try_into()?,
        };
        let chat_header = FrameHeaderWire {
            msg_type: msg_type::CHAT.into(),
            len_bytes: ((std::mem::size_of::<FrameHeaderWire>() + std::mem::size_of::<ChatWire>())
                as u16)
                .into(),
            request_id: 2u32.into(),
        };
        write_specific(&mut stream, &chat_header)?;
        write_specific(&mut stream, &chat_payload)?;
        stream.flush()?;

        // 3) Read a SERVER_MSG (header + payload).
        let header: FrameHeaderWire = read_specific(&mut stream)?;
        let msg_type = header.msg_type.to_native();
        if msg_type != msg_type::SERVER_MSG {
            return Err(format!("expected SERVER_MSG, got type {msg_type}").into());
        }

    let payload: ServerMsgWire = read_specific(&mut stream)?;

    // Be explicit about the wire type we're decoding here.
    // (The derive-generated field type is fixed UTF-32BE, 64 code units, space padded.)
    let text = String::try_from(&payload.text as &FixedUtf32BeSpacePadded<64>)?;
        println!("server: {text}");

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
    return real::run();

    #[cfg(not(all(feature = "derive", feature = "io-std", feature = "text_all")))]
    {
        eprintln!(
            "This example requires features: derive, io-std, text_all\n\n  cargo run --example messaging_client --features \"derive io-std text_all\" -- 127.0.0.1:7777 alice \"hello\""
        );
        Ok(())
    }
}

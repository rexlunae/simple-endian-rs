//! Messaging app example (server).
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
    use std::io::Write;
    use std::net::{TcpListener, TcpStream};

    fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        stream.set_nodelay(true)?;

        // Read HELLO header + payload.
        let hello_hdr: FrameHeaderWire = read_specific(&mut stream)?;
        if hello_hdr.msg_type.to_native() != msg_type::HELLO {
            return Err("expected HELLO".into());
        }

        let hello: HelloWire = read_specific(&mut stream)?;
        let username = String::try_from(&hello.username)?;

        // Send SERVER_MSG welcome.
        let welcome_text = format!("welcome, {username}!");
        let welcome_payload = ServerMsgWire {
            text: welcome_text.as_str().try_into()?,
        };
        let welcome_hdr = FrameHeaderWire {
            msg_type: msg_type::SERVER_MSG.into(),
            len_bytes: ((std::mem::size_of::<FrameHeaderWire>()
                + std::mem::size_of::<ServerMsgWire>()) as u16)
                .into(),
            request_id: 1u32.into(),
        };
        write_specific(&mut stream, &welcome_hdr)?;
        write_specific(&mut stream, &welcome_payload)?;
        stream.flush()?;

        // Read CHAT header + payload.
        let chat_hdr: FrameHeaderWire = read_specific(&mut stream)?;
        if chat_hdr.msg_type.to_native() != msg_type::CHAT {
            return Err("expected CHAT".into());
        }

        let chat: ChatWire = read_specific(&mut stream)?;
        let text = String::try_from(&chat.text)?;

        // Send SERVER_MSG ack.
        let ack_text = format!("ack: {text}");
        let ack_payload = ServerMsgWire {
            text: ack_text.as_str().try_into()?,
        };
        let ack_hdr = FrameHeaderWire {
            msg_type: msg_type::SERVER_MSG.into(),
            len_bytes: ((std::mem::size_of::<FrameHeaderWire>()
                + std::mem::size_of::<ServerMsgWire>()) as u16)
                .into(),
            request_id: 2u32.into(),
        };
        write_specific(&mut stream, &ack_hdr)?;
        write_specific(&mut stream, &ack_payload)?;
        stream.flush()?;

        Ok(())
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let addr = std::env::args()
            .nth(1)
            .unwrap_or_else(|| "127.0.0.1:7777".to_string());
        let listener = TcpListener::bind(addr)?;

        println!("server listening");

        for conn in listener.incoming() {
            match conn {
                Ok(stream) => {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("client error: {e:#}");
                    }
                }
                Err(e) => eprintln!("accept error: {e:#}"),
            }
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
    return real::run();

    #[cfg(not(all(feature = "derive", feature = "io-std", feature = "text_all")))]
    {
        eprintln!(
            "This example requires features: derive, io-std, text_all\n\n  cargo run --example messaging_server --features \"derive io-std text_all\""
        );
        Ok(())
    }
}

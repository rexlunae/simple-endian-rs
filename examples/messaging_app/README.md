# Messaging app example (client + server)

This directory contains a tiny TCP **server** and **client** example that use `simple_endian`'s `#[derive(Endianize)]` to define a fixed-size, endian-stable wire protocol.

## Goals

- **Safe Rust only** in the example code.
- Use `#[derive(Endianize)]` to generate `*Wire` types.
- Read/write messages using the crate's `io-std` feature (`read_specific` / `write_specific`).

## Protocol shape

The protocol is intentionally simple:

- Every frame begins with a `FrameHeaderWire`.
- Then, based on `msg_type`, a fixed-size payload follows (`HelloWire`, `ChatWire`, etc.).
- Text fields on the wire are fixed-width, UTF-16, space-padded.

## Run it

In two terminals:

```bash
cargo run --example messaging_server --features "derive io-std text_all"
```

```bash
cargo run --example messaging_client --features "derive io-std text_all" -- alice
```

The client takes optional args:

1) server address (default `127.0.0.1:7777`)
2) username (default `alice`)
3) message text (default `hello from client`)

After sending a HELLO + CHAT, the client reads one `SERVER_MSG` response and exits.

## Notes

- The server is single-threaded and handles one client at a time.
- Message lengths are not currently enforced (we use fixed-size payloads), so `len_bytes` is reserved for future extension.

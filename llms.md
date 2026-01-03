# LLM guide

This repository includes a concise, LLM-oriented usage guide in [`LLMs.txt`](./LLMs.txt).

That file covers:

- What `simple_endian` is for (binary formats + explicit endianness)
- Which Cargo features enable which APIs (`derive`, `io-std`/`io-core`, `text_*`)
- Recommended patterns (logical types + generated `*Wire` types)
- `#[derive(Endianize)]` usage, constraints for enums/unions
- Wire layout control: `#[wire_repr(...)]` (e.g. `packed`)
- Wire trait passthrough: `#[wire_derive(...)]`
- Safety notes (packed/unaligned fields)

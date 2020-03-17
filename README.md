# ages-prs: PRS de/compression for Rust

[![CI](https://github.com/HybridEidolon/rust-ages-prs/workflows/CI/badge.svg)](https://github.com/HybridEidolon/rust-ages-prs/actions?query=workflow%3ACI)
[![Crate](https://img.shields.io/crates/v/ages-prs.svg)](https://crates.io/crates/ages-prs)
[![API](https://docs.rs/ages-prs/badge.svg)](https://docs.rs/ages-prs)

IO types for compressing and decompressing PRS encoded buffers.

PRS is an LZ77 encoding used by several games made published by SEGA. It is
mostly used for compressing game assets e.g. textures and game data, and
occasionally used for message compression in network games' application
protocols.

The API surface is intentionally minimal. The underlying LZ77 implementation is
not exposed; currently, this crate uses
[libflate's LZ77 encoder](https://crates.io/crates/libflate_lz77).

This crate should work out-of-the-box when targeting WebAssembly, though it is
not tested yet.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
ages-prs = "0.1"
```

Within your code:

```rust
use std::io::{self, Cursor, Read};
use ages_prs::LegacyPrsDecoder;

// unitxt_j.prs contains localized strings used in Phantasy Star Online's UI.
// PSO uses "legacy" variant PRS.

fn decompress_unitxt() -> io::Result<Vec<u8>> {
    let mut data = Vec::with_capacity(2048);
    let file = File::open("unitxt_js.prs")?;
    let mut decoder = LegacyPrsDecoder::new(file);

    decoder.read_to_end(&mut data)?;

    Ok(data)
}
```

## Games supported

For the "Legacy" variant:

- Phantasy Star Online (all versions)
- Sonic Adventure
- Sonic Adventure 2

For the "Modern" variant:

- Phantasy Star Universe
- Phantasy Star Online 2

These lists are not comprehensive. SEGA has used PRS in many games since as
early as the SEGA Saturn, and it has received various alterations over the
years.

## License

`ages-prs` is dual-licensed for compatibility with the rest of the Rust public
ecosystem.

`ages-prs` is distributed under the terms of both the MIT license and the Apache
License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

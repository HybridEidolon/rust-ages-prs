# sega-prs: PRS de/compression for Rust

[![Build Status](https://travis-ci.org/HybridEidolon/rust-sega-prs.svg?branch=master)](https://travis-ci.org/HybridEidolon/rust-sega-prs)
[![Crate](https://img.shields.io/crates/v/sega-prs.svg)](https://crates.io/crates/sega-prs)
[![API](https://docs.rs/sega-prs/badge.svg)](https://docs.rs/sega-prs)

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
sega-prs = "0.1"
```

Within your code:

```rust
use std::io::{Cursor, Read};
use sega_prs::LegacyPrsDecoder;

// unitxt_j.prs contains localized strings used in Phantasy Star Online's UI.
// PSO uses "legacy" variant PRS.
static UNITXT: &'static [u8] = include_bytes!("./unitxt_j.prs");

fn decompress_unitxt() {
    let mut unitxt = Vec::new();
    let mut decoder = LegacyPrsDecoder::new(
        File::open("unitxt_j.prs").unwrap(),
    );
    decoder.read_to_end(&mut unitxt).unwrap();
    drop(decoder);
    println!("{:02x?}", unitxt);
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

`sega-prs` is distributed under the terms of both the MIT license and the Apache
License (Version 2.0).

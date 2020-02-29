//! Compression and decompression of SEGA's LZ77 encoding, PRS, named after the
//! file extension typically used for data encoded with it.
//!
//! There are two supported variants of PRS: Legacy and Modern. The compression
//! and decompression routines are split between these two variants. Which
//! variant to use depends on what game you are targeting the data for. For
//! example, _Phantasy Star Online_ (2000) uses Legacy variant, and _Phantasy
//! Star Online 2_ (2012) uses Modern variant. Not using the correct variant
//! will likely result in undefined behavior in the targeted game, but this
//! library will try to produce an Error if there would result in memory-unsafe
//! copies in the command stream. That said, there is no way to _detect_ what
//! kind of PRS variant a given buffer is in.
//!
//! # Examples
//!
//! Compress and decompress a buffer:
//!
//! ```
//! use std::io::{Cursor, Read, Write};
//!
//! use ages_prs::{ModernPrsDecoder, ModernPrsEncoder};
//!
//! let input = b"Hello Hello Hello ";
//! let mut encoder = ModernPrsEncoder::new(Vec::new());
//! encoder.write_all(input).unwrap();
//! let compressed = encoder.into_inner().unwrap();
//!
//! let mut decoder = ModernPrsDecoder::new(Cursor::new(&compressed[..]));
//! let mut decomp = Vec::new();
//! decoder.read_to_end(&mut decomp).unwrap();
//! assert_eq!(&decomp[..], &input[..]);
//! ```

mod compress;
mod decompress;
mod variant;

pub use self::compress::{PrsEncoder, IntoInnerError};
pub use self::decompress::PrsDecoder;

pub use self::variant::{
    Variant,
    Legacy,
    Modern,
};

pub type ModernPrsEncoder<W> = PrsEncoder<W, Modern>;
pub type LegacyPrsEncoder<W> = PrsEncoder<W, Legacy>;
pub type ModernPrsDecoder<R> = PrsDecoder<R, Modern>;
pub type LegacyPrsDecoder<R> = PrsDecoder<R, Legacy>;

#[cfg(test)]
mod test;

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
//! # use sega_prs::{PrsEncoder, Modern, decompress_modern};
//! use std::io::Write;
//!
//! let input = b"Hello Hello Hello ";
//! let mut encoder = PrsEncoder::<_, Modern>::new(Vec::new());
//! encoder.write_all(input).unwrap();
//! let compressed = encoder.into_inner().unwrap();
//!
//! let decompressed = decompress_modern(&compressed[..]).unwrap();
//! assert_eq!(&decompressed[..], &input[..]);
//! ```

mod compress;
mod decompress;
mod variant;

pub use self::compress::{PrsEncoder, IntoInnerError};
use self::decompress::decompress;

pub use self::variant::{
    Variant,
    Legacy,
    Modern,
};

pub use self::decompress::DecompressError;

/// Decompress a Legacy-encoded buffer.
pub fn decompress_legacy<B>(buf: B) -> Result<Vec<u8>, DecompressError>
where
    B: AsRef<[u8]>,
{
    decompress::<Legacy, B>(buf)
}

/// Decompress a Modern-encoded buffer.
pub fn decompress_modern<B>(buf: B) -> Result<Vec<u8>, DecompressError>
where
    B: AsRef<[u8]>,
{
    decompress::<Modern, B>(buf)
}

#[cfg(test)]
mod test;

//! Compression and decompression of SEGA's LZ77 encoding, PRS, named after the
//! file extension typically used for data encoded with it.
//!
//! There are two supported forms of PRS: Legacy and Modern. The compression and
//! decompression routines are split between these two forms. Which form to use
//! depends on what game you are targeting the data for. For example, _Phantasy
//! Star Online_ (2000) uses Legacy encoding, and _Phantasy Star Online 2_
//! (2012) uses Modern encoding. Not using the correct encoding will likely
//! result in undefined behavior in the targeted game, but this library will try
//! to produce an Error if there would result in memory-unsafe copies in the
//! command stream. That said, there is no way to _detect_ what kind of PRS
//! variation a given buffer is in.
//!
//! The routines are not generic over the PRS form because it would not be
//! useful to allow downstream consumers to provide "alternate" variations. If
//! your code needs to be generically variant over the PRS form, you can wrap
//! the functions in trait impls over your own trait. An example is provided.
//!
//! # Examples
//!
//! Compress and decompress an input buffer:
//!
//! ```
//! # use sega_prs::{compress_modern, decompress_modern};
//! let input = b"Hello Hello Hello ";
//! let compressed = compress_modern(&input[..]);
//! assert!(compressed.len() < input.len());
//!
//! let decompressed = decompress_modern(&compressed[..]).unwrap();
//! assert_eq!(&decompressed[..], &input[..]);
//! ```
//!
//! A generic compression routine:
//!
//! ```
//! # use sega_prs::{compress_modern, compress_legacy};
//! trait Compress {
//!     fn compress<B: AsRef<[u8]>>(buf: B) -> Vec<u8>;
//! }
//! 
//! enum Modern {};
//! enum Legacy {};
//!
//! impl Compress for Modern {
//!     fn compress<B: AsRef<[u8]>>(buf: B) -> Vec<u8> { compress_modern(buf) }
//! }
//! impl Compress for Legacy {
//!     fn compress<B: AsRef<[u8]>>(buf: B) -> Vec<u8> { compress_legacy(buf) }
//! }
//!
//! fn compress<C, B>(buf: B) -> Vec<u8>
//! where
//!     C: Compress,
//!     B: AsRef<[u8]>,
//! {
//!     <C as Compress>::compress::<B>(buf)
//! }
//!
//! compress::<Legacy, _>(b"Hello Hello Hello Hello ");
//! compress::<Modern, _>(b"Hello Hello Hello Hello ");
//! ```

mod compress;
mod decompress;
pub(crate) mod flavor;

use self::compress::compress;
use self::decompress::decompress;

use self::flavor::{
    Legacy,
    Modern,
};

pub use self::decompress::DecompressError;

/// Compress a buffer using Legacy encoding.
pub fn compress_legacy<B: AsRef<[u8]>>(buf: B) -> Vec<u8> {
    compress::<Legacy, B>(buf)
}

/// Compress a buffer using Modern encoding.
pub fn compress_modern<B: AsRef<[u8]>>(buf: B) -> Vec<u8> {
    compress::<Modern, B>(buf)
}

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

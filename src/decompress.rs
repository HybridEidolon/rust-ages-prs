//! Decompression of PRS buffers.

use crate::flavor::Flavor;

use std::error::Error;
use std::fmt;
use std::io::{Read, Cursor};

/// An Error returned during decompression.
#[derive(Debug)]
pub enum DecompressError {
    /// Reached end of buffer prematurely
    Eof,
    /// A Pointer command was invalid (not enough written bytes)
    InvalidPointer {
        /// The distance backwards in the output buffer to copy from
        dist: usize,
        /// The number of bytes to copy
        len: usize,
        /// The current length of the output buffer
        current_len: usize,
    },
    #[doc(hidden)]
    __Nonexhaustive,
}

impl fmt::Display for DecompressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecompressError::Eof => {
                write!(f, "Reached end of buffer prematurely")
            },
            DecompressError::InvalidPointer { dist, len, current_len } => {
                write!(
                    f,
                    "Invalid pointer command: {} bytes {} away, {} available",
                    len,
                    dist,
                    current_len
                )
            },
            _ => unimplemented!()
        }
    }
}

impl Error for DecompressError {}

/// Decompress a byte buffer, as a particular Flavor.
pub fn decompress<F, B>(buf: B) -> Result<Vec<u8>, DecompressError>
where
    F: Flavor,
    B: AsRef<[u8]>,
{
    decompress_buf::<F>(buf.as_ref())
}

fn decompress_buf<F: Flavor>(buf: &[u8]) -> Result<Vec<u8>, DecompressError> {
    if buf.is_empty() {
        // empty buffer; return empty result
        return Ok(Vec::new());
    }

    let mut ctx: Ctx<F> = Ctx::new(buf);

    let mut out = Vec::with_capacity(buf.len().next_power_of_two());

    loop {
        let cmd = ctx.next_cmd()?;
        match cmd {
            Some(Cmd::Literal(b)) => out.push(b),
            Some(Cmd::Pointer(dist, len)) => {
                for _ in 0..len {
                    if dist == 0 || out.len() < dist {
                        return Err(DecompressError::InvalidPointer {
                            dist,
                            len,
                            current_len: out.len(),
                        });
                    }
                    out.push(out[out.len()-dist]);
                }
            },
            None => break
        }
    }
    Ok(out)
}

struct Ctx<'a, F> {
    cursor: Cursor<&'a [u8]>,
    cmds: u8,
    rem: u8,
    pd: std::marker::PhantomData<F>,
}

// LZ77 commands
#[derive(Debug)]
enum Cmd {
    Literal(u8),
    Pointer(usize, usize),
}

impl<'a, F> Ctx<'a, F> {
    fn new(src: &'a [u8]) -> Ctx<'a, F> {
        Ctx {
            cursor: Cursor::new(src),
            cmds: 0,
            rem: 0,
            pd: std::marker::PhantomData,
        }
    }

    #[inline(always)]
    fn read_bit(&mut self) -> Result<bool, DecompressError> {
        if self.rem == 0 {
            let mut buf = [0; 1];
            if self.cursor.read_exact(&mut buf).is_err() {
                return Err(DecompressError::Eof);
            }
            self.cmds = buf[0];
            self.rem = 8;
        }

        let ret = self.cmds & 1;
        self.cmds >>= 1;
        self.rem -= 1;

        match ret { 0 => Ok(false), _ => Ok(true) }
    }
}

impl<'a, F> Ctx<'a, F> where F: Flavor {
    fn next_cmd(&mut self) -> Result<Option<Cmd>, DecompressError> {
        if self.read_bit()? {
            // literal
            let mut buf = [0; 1];
            if self.cursor.read_exact(&mut buf).is_err() {
                return Err(DecompressError::Eof);
            }
            return Ok(Some(Cmd::Literal(buf[0])));
        }

        if self.read_bit()? {
            // long ptr
            let mut buf = [0; 2];
            let mut offset = match self.cursor.read_exact(&mut buf) {
                Err(_) => return Err(DecompressError::Eof),
                _ => i16::from_le_bytes(buf) as i32,
            };

            if offset == 0 {
                return Ok(None);
            }

            let mut size = (offset & 0b111) as usize;
            offset >>= 3;

            if size == 0 {
                // next byte is real size
                size = match self.cursor.read_exact(&mut buf[..1]) {
                    Err(_) => return Err(DecompressError::Eof),
                    _ => buf[0] as usize,
                };
                // it's probably the minimum long-long-copy size
                size += F::MIN_LONG_COPY_LENGTH as usize;
            } else {
                size += 2;
            }
            offset |= -8192i32;

            Ok(Some(Cmd::Pointer((-offset) as usize, size)))
        } else {
            // short ptr
            let mut buf = [0; 1];
            let flag = if self.read_bit()? { 1 } else { 0 };
            let bit = if self.read_bit()? { 1 } else { 0 };
            let size = (bit | (flag << 1)) + 2;
            let offset = match self.cursor.read_exact(&mut buf) {
                Err(_) => return Err(DecompressError::Eof),
                _ => buf[0] as i32,
            };
            let offset = offset | -256i32;
            
            Ok(Some(Cmd::Pointer((-offset) as usize, size)))
        }
    }
}

//! Decompression of PRS buffers.

use crate::Variant;

use std::collections::VecDeque;
use std::io::{self, Cursor, Read, Write};

/// An IO source for decoding a PRS stream.
pub struct PrsDecoder<R: Read, V: Variant> {
    inner: R,
    cmds: u8,
    rem: u8,
    copy_buf: VecDeque<u8>,
    eof: bool,
    pd: std::marker::PhantomData<V>,
}

// LZ77 commands
#[derive(Debug)]
enum Cmd {
    Literal(u8),
    Pointer(usize, usize),
}

impl<R: Read, V: Variant> PrsDecoder<R, V> {
    pub fn new(inner: R) -> PrsDecoder<R, V> {
        PrsDecoder {
            inner,
            cmds: 0,
            rem: 0,
            copy_buf: VecDeque::with_capacity(8191),
            eof: false,
            pd: std::marker::PhantomData,
        }
    }

    fn read_bit(&mut self) -> io::Result<bool> {
        if self.rem == 0 {
            let mut buf = [0; 1];
            self.inner.read_exact(&mut buf)?;
            self.cmds = buf[0];
            self.rem = 8;
        }

        let ret = self.cmds & 1;
        self.cmds >>= 1;
        self.rem -= 1;

        match ret { 0 => Ok(false), _ => Ok(true) }
    }

    fn next_cmd(&mut self) -> io::Result<Option<Cmd>> {
        if self.read_bit()? {
            // literal
            let mut buf = [0; 1];
            self.inner.read_exact(&mut buf)?;
            return Ok(Some(Cmd::Literal(buf[0])));
        }

        if self.read_bit()? {
            // long ptr
            let mut buf = [0; 2];
            self.inner.read_exact(&mut buf)?;
            let mut offset = i16::from_le_bytes(buf) as i32;

            if offset == 0 {
                return Ok(None);
            }

            let mut size = (offset & 0b111) as usize;
            offset >>= 3;

            if size == 0 {
                // next byte is real size
                self.inner.read_exact(&mut buf[..1])?;
                size = buf[0] as usize;
                // it's probably the minimum long-long-copy size
                size += V::MIN_LONG_COPY_LENGTH as usize;
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
            self.inner.read_exact(&mut buf)?;
            let offset = buf[0] as i32;
            let offset = offset | -256i32;
            
            Ok(Some(Cmd::Pointer((-offset) as usize, size)))
        }
    }
}

impl<R: Read, V: Variant> Read for PrsDecoder<R, V> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // first, fill the copy buffer as much as possible
        while self.copy_buf.len() < 8191 + buf.len() && !self.eof {
            match self.next_cmd()? {
                None => {
                    self.eof = true;
                    break;
                },
                Some(Cmd::Literal(b)) => {
                    self.copy_buf.push_back(b);
                },
                Some(Cmd::Pointer(offset, size)) => {
                    for _ in 0..size {
                        if offset == 0 || self.copy_buf.len() < offset {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "bad pointer copy in stream"
                            ));
                        }
                        self.copy_buf.push_back(self.copy_buf[self.copy_buf.len() - offset]);
                    }
                },
            }
        }

        // then, drain the amount of the copy buffer that is necessary to read
        let bytes_read = std::cmp::min(buf.len(), self.copy_buf.len());
        let mut cursor = Cursor::new(buf);
        self.copy_buf.drain(..bytes_read).for_each(|b| { cursor.write_all(&[b]).unwrap(); });

        Ok(bytes_read)
    }
}

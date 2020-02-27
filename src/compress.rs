//! Compression routine for PRS

use crate::Variant;

use std::fmt;
use std::error;
use std::io::{self, Write};

use libflate_lz77::{
    Code,
    DefaultLz77Encoder,
    DefaultLz77EncoderBuilder,
    Lz77Encode,
    Sink,
    MAX_LENGTH,
};

/// An IO sink for compressing and encoding a stream to PRS.
pub struct PrsEncoder<W: Write, V: Variant> {
    sink: Option<PrsSink<V>>,
    inner: Option<W>,
    encoder: DefaultLz77Encoder,
    _pd: std::marker::PhantomData<V>,
}

/// Error returned when `PrsEncoder::into_inner` fails.
#[derive(Debug)]
pub struct IntoInnerError<W>(W, io::Error);

impl<W: Write, V: Variant> PrsEncoder<W, V> {
    /// Wraps a Write sink, initializing the encoder state
    pub fn new(inner: W) -> PrsEncoder<W, V> {
        let encoder = DefaultLz77EncoderBuilder::new()
            .window_size(8191)
            .max_length(std::cmp::min(MAX_LENGTH, V::MAX_COPY_LENGTH))
            .build();
        
        PrsEncoder {
            sink: Some(PrsSink::new(32)),
            inner: Some(inner),
            encoder,
            _pd: std::marker::PhantomData,
        }
    }

    /// Finish encoding the PRS stream, returning the inner Write.
    ///
    /// Errors will leave the PRS stream in an incomplete state; the E type is
    /// only present to capture the inner Write for inspection. There is no way
    /// to recover the broken PRS stream if this operation fails.
    pub fn into_inner(mut self) -> Result<W, IntoInnerError<W>> {
        match self.flush_buf() {
            Err(e) => Err(IntoInnerError(self.inner.take().unwrap(), e)),
            Ok(()) => {
                let mut sink = self.sink.take().unwrap();
                let mut inner = self.inner.take().unwrap();
                self.encoder.flush(&mut sink);
                let buf = sink.finish();

                match inner.write_all(&buf[..]) {
                    Err(e) => Err(IntoInnerError(inner, e)),
                    Ok(_) => Ok(inner),
                }
            },
        }
    }

    /// Attempt to flush the intermediary buffer to the sink
    fn flush_buf(&mut self) -> io::Result<()> {
        let mut sink = self.sink.as_mut().unwrap();
        let inner = self.inner.as_mut().unwrap();

        // everything before the current cmd index is safe to write
        let high_water = sink.cmd_index;
        if high_water == 0 {
            // don't flush; we don't have a saturated command byte yet
            return Ok(());
        }

        let mut written = 0;
        let len = high_water;
        let mut ret: io::Result<()> = Ok(());

        while written < len {
            // only write up to len bytes this flush
            let r = inner.write(&sink.out[written..len]);

            match r {
                Ok(0) => {
                    ret = Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "failed to write the buffered data"
                    ));
                    break;
                },
                Ok(n) => written += n,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {},
                Err(e) => {
                    ret = Err(e);
                    break;
                }
            }
        }
        if written > 0 {
            sink.out.drain(..written);
            sink.cmd_index -= written;
        }
        ret
    }
}

impl<W: Write, V: Variant> Write for PrsEncoder<W, V> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // unlike BufWriter we can't flush when buffer capacity is hit
        {
            self.encoder.encode(buf, self.sink.as_mut().unwrap());
        }
        // we'll try to flush as much as possible since buffer perf is not
        // the goal here; PrsEncoder<BufWriter<_>, _> is fine for that
        self.flush_buf()?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_buf().and_then(|()| self.inner.as_mut().unwrap().flush())
    }
}

impl<W: Write, V: Variant> Drop for PrsEncoder<W, V> {
    fn drop(&mut self) {
        if self.inner.is_some() && self.sink.is_some() {
            let _r = self.flush_buf();
            let mut sink = self.sink.take().unwrap();
            let mut inner = self.inner.take().unwrap();
            self.encoder.flush(&mut sink);
            let buf = sink.finish();

            // we'll try to finish the stream but it is impossible to report
            // errors from a Drop
            let _r = inner.write_all(&buf[..]);
        }
    }
}

impl<W: Write, V: Variant> fmt::Debug for PrsEncoder<W, V>
where
    W: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("PrsEncoder")
            .field("writer", &self.inner.as_ref().unwrap())
            .field("buffer", &self.sink.as_ref().unwrap().out)
            .finish()
    }
}

impl<W> fmt::Display for IntoInnerError<W> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "Failed to complete PRS stream: {}", self.1)
    }
}

impl<W: Send + fmt::Debug> error::Error for IntoInnerError<W> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.1)
    }
}

impl<W> IntoInnerError<W> {
    /// Reference the IO error that failed the operation.
    pub fn error(&self) -> &io::Error {
        &self.1
    }

    /// Retrieve the inner type.
    pub fn into_inner(self) -> W {
        self.0
    }
}

// ---- LZ77 Sink implementation ----

struct PrsSink<V: Variant> {
    /// index into `out` which is the current cmd stream head
    cmd_index: usize,
    /// how many cmd bits can we still write
    cmd_bits_rem: u8,
    /// the output buffer
    out: Vec<u8>,

    _pd: std::marker::PhantomData<V>,
}

impl<V: Variant> PrsSink<V> {
    fn new(capacity: usize) -> PrsSink<V> {
        PrsSink {
            cmd_index: 0,
            cmd_bits_rem: 0,
            out: Vec::with_capacity(capacity),
            _pd: std::marker::PhantomData,
        }
    }

    fn write_bit(&mut self, bit: bool) {
        if self.cmd_bits_rem == 0 {
            self.cmd_index = self.out.len();
            self.cmd_bits_rem = 8;
            self.out.push(0);
        }

        if bit {
            self.out[self.cmd_index] |= 1 << (8 - self.cmd_bits_rem);
        }

        self.cmd_bits_rem -= 1;
    }

    fn finish(mut self) -> Vec<u8> {
        self.write_bit(false);
        self.write_bit(true); // long ptr
        self.out.push(0); // zero offset = EOF
        self.out.push(0);

        self.out
    }
}

impl<V: Variant> Sink for PrsSink<V> {
    fn consume(&mut self, code: Code) {
        match code {
            Code::Literal(b) => {
                self.write_bit(true);
                self.out.push(b);
            },
            Code::Pointer { length, backward_distance } => {
                // preconditions
                if length < 2 {
                    panic!("copy length too small (< 2)");
                }
                if length > V::MAX_COPY_LENGTH {
                    panic!("copy length too large");
                }
                if backward_distance >= 8192 {
                    panic!("copy distance too far (>8191)");
                }

                if backward_distance >= 256 || length > 5 {
                    // long ptr
                    self.write_bit(false);
                    self.write_bit(true);

                    let mut offset = backward_distance as i32;
                    
                    offset = -offset;
                    offset <<= 3;
                    if (length - 2) < 8 {
                        offset |= (length - 2) as i32;
                    }

                    self.out.extend_from_slice(&(offset as u16).to_le_bytes());
                    
                    if (length - 2) >= 8 {
                        let size = (
                            length - (V::MIN_LONG_COPY_LENGTH as u16)
                        ) as u8;
                        self.out.push(size);
                    }
                } else {
                    // short ptr
                    self.write_bit(false);
                    self.write_bit(false);

                    let offset = backward_distance as i32;
                    let size = (length - 2) as i32;
                    
                    self.write_bit(size & 0b10 > 0);
                    self.write_bit(size & 0b01 > 0);
                    self.out.push((-offset & 0xFF) as u8);
                }
            },
        }
    }
}

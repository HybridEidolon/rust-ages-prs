//! Compression routine for PRS

use crate::flavor::Flavor;

use libflate_lz77::{
    Code,
    DefaultLz77EncoderBuilder,
    Lz77Encode,
    Sink,
    MAX_LENGTH,
};

pub fn compress<F, B>(buf: B) -> Vec<u8>
where
    F: Flavor,
    B: AsRef<[u8]>,
{
    compress_buf::<F>(buf.as_ref())
}

fn compress_buf<F: Flavor>(buf: &[u8]) -> Vec<u8> {
    if buf.is_empty() {
        return Vec::new();
    }

    let mut encoder = DefaultLz77EncoderBuilder::new()
        .window_size(8191)
        .max_length(std::cmp::min(MAX_LENGTH, F::MAX_COPY_LENGTH))
        .build();
    let mut sink = PrsSink::<F>::new(
        // reasonable capacity? 3000 bytes => 2048 output buffer
        buf.len().next_power_of_two().overflowing_shr(2).0
    );
    encoder.encode(buf, &mut sink);
    encoder.flush(&mut sink);

    sink.finish()
}

struct PrsSink<F> {
    /// index into `out` which is the current cmd stream head
    cmd_index: usize,
    /// how many cmd bits can we still write
    cmd_bits_rem: u8,
    /// the output buffer
    out: Vec<u8>,

    _pd: std::marker::PhantomData<F>,
}

impl<F> PrsSink<F> {
    fn new(capacity: usize) -> PrsSink<F> {
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

impl<F> Sink for PrsSink<F> where F: Flavor {
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
                if length > F::MAX_COPY_LENGTH {
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
                            length - (F::MIN_LONG_COPY_LENGTH as u16)
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

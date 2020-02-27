use crate::{
    PrsEncoder,
    PrsDecoder,
    Variant,
    Legacy,
    Modern,
};

use std::io::{Cursor, Read, Write};

static TEST_DATA: &'static [u8] = include_bytes!("./test.txt");

fn compress<V, B>(buf: B) -> Vec<u8>
where
    V: Variant,
    B: AsRef<[u8]>,
{
    compress_buf::<V>(buf.as_ref())
}

fn compress_buf<V: Variant>(buf: &[u8]) -> Vec<u8> {
    let out = Vec::with_capacity(
        buf.len().next_power_of_two().overflowing_shr(2).0
    );
    let mut encoder: PrsEncoder<_, V> = PrsEncoder::new(out);
    encoder.write_all(buf).unwrap();
    encoder.into_inner().unwrap()
}

fn decompress<V, B>(buf: B) -> Vec<u8>
where
    V: Variant,
    B: AsRef<[u8]>,
{
    decompress_buf::<V>(buf.as_ref())
}

fn decompress_buf<V: Variant>(buf: &[u8]) -> Vec<u8> {
    if buf.is_empty() {
        // empty buffer; return empty result
        return Vec::new();
    }

    let mut out = Vec::with_capacity(buf.len().next_power_of_two());
    let mut decoder = PrsDecoder::<_, V>::new(Cursor::new(buf));
    decoder.read_to_end(&mut out).unwrap();
    out
}

#[test]
fn test_compress_decompress_legacy() {
    let mut data = Vec::with_capacity(TEST_DATA.len() * 100);
    for _ in 0..100 {
        data.extend_from_slice(TEST_DATA);
    }
    let compressed = compress::<Legacy, _>(&data[..]);
    let decompressed = decompress::<Legacy, _>(&compressed);

    assert!(compressed.len() < data.len());
    assert!(decompressed == data);
}

#[test]
fn test_compress_decompress_modern() {
    let mut data = Vec::with_capacity(TEST_DATA.len() * 100);
    for _ in 0..100 {
        data.extend_from_slice(TEST_DATA);
    }
    let compressed = compress::<Modern, _>(&data[..]);
    let decompressed = decompress::<Modern, _>(&compressed);

    assert!(compressed.len() < data.len());
    assert!(decompressed == data);
}

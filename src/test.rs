use crate::{
    PrsEncoder,
    Variant,
    Legacy,
    Modern,
    decompress_legacy,
    decompress_modern,
};

use std::io::Write;

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

#[test]
fn test_compress_decompress_legacy() {
    let mut data = Vec::with_capacity(TEST_DATA.len() * 100);
    for _ in 0..100 {
        data.extend_from_slice(TEST_DATA);
    }
    let compressed = compress::<Legacy, _>(&data[..]);
    let decompressed = decompress_legacy(&compressed)
        .expect("unable to decompress");

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
    let decompressed = decompress_modern(&compressed)
        .expect("unable to decompress");

    assert!(compressed.len() < data.len());
    assert!(decompressed == data);
}

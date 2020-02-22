use crate::{
    compress_legacy,
    decompress_legacy,
    compress_modern,
    decompress_modern,
};

static TEST_DATA: &'static [u8] = include_bytes!("./test.txt");

#[test]
fn test_compress_decompress_legacy() {
    let compressed = compress_legacy(TEST_DATA);
    let decompressed = decompress_legacy(&compressed)
        .expect("unable to decompress");

    assert!(decompressed == TEST_DATA);
}

#[test]
fn test_compress_decompress_modern() {
    let compressed = compress_modern(TEST_DATA);
    let decompressed = decompress_modern(&compressed)
        .expect("unable to decompress");

    assert!(decompressed == TEST_DATA);
}

extern crate graph_layout;
use graph_layout::layout::*;
use graph_layout::compression::*;

#[test]
fn encode_decode_byte() {
    let hilbert = Hilbert::new();
    for i in 0 .. (1 << 20) {
        assert_eq!(hilbert.entangle(hilbert.detangle(i)), i);
    }
}

#[test]
fn compress_decompress() {
    let source = vec![0,1,2,4, 100, 123412, 1543245423];
    let compressed = Compressed::from(source.iter().map(|&x|x));
    let decompressor = compressed.decompress();
    let result = decompressor.collect::<Vec<_>>();
    assert_eq!(result, source);
}

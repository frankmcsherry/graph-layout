extern crate hilbert_curve;
extern crate test;
use test::Bencher;

use hilbert_curve::BytewiseHilbert;

#[bench]
fn encode_decode_byte(bencher: &mut Bencher) {
    let hilbert = hilbert_curve::BytewiseHilbert::new();

    let mut index = 0;
    bencher.iter(|| {
        let z = hilbert.entangle((index, 7u32));
        let (x,y) = hilbert.detangle(z);
        // println!("({}, {}) -> {} -> ({}, {})", i, 7, z, x, y);
        assert!((x,y) == (index, 7u32));
        index += 1;
    });
}

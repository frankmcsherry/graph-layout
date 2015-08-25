#![feature(test)]
extern crate test;
use test::Bencher;

extern crate graph_layout;
use graph_layout::layout::*;
use graph_layout::compression::*;

#[bench]
fn encode_decode_h(bencher: &mut Bencher) {
    let tangler = Hilbert::new();
    let mut index = 0;
    bencher.iter(|| { index += 1; assert!(index == tangler.entangle(tangler.detangle(index))) });
}

#[bench]
fn encode_h(bencher: &mut Bencher) {
    let tangler = Hilbert::new();
    let mut index = 0;
    bencher.iter(|| { index += 1; tangler.entangle((index, 7u32)) });
}

#[bench]
fn decode_h(bencher: &mut Bencher) {
    let tangler = Hilbert::new();
    let mut index = 0;
    bencher.iter(|| { index += 1; tangler.detangle(index) });
}

#[bench]
fn encode_decode_z(bencher: &mut Bencher) {
    let tangler = ZOrder::new();
    let mut index = 0;
    bencher.iter(|| { index += 1; assert!(index == tangler.entangle(tangler.detangle(index))) });
}

#[bench]
fn encode_z(bencher: &mut Bencher) {
    let tangler = ZOrder::new();
    let mut index = 0;
    bencher.iter(|| { index += 1; tangler.entangle((index, 7u32)) });
}

#[bench]
fn decode_z(bencher: &mut Bencher) {
    let tangler = ZOrder::new();
    let mut index = 0;
    bencher.iter(|| { index += 1; tangler.detangle(index) });
}

#[bench]
fn compress_e(bencher: &mut Bencher) {
    let mut compressor = Compressor::with_capacity(1_000_000);
    let mut index = 0u64;
    bencher.iter(|| {
        index += 1;
        compressor.push(index)
    })
}

#[bench]
fn compress_d(bencher: &mut Bencher) {
    let compressed = Compressed::from(0..1_000_000);
    let mut decompressor = compressed.decompress();
    bencher.iter(|| {
        decompressor.next()
    });
}

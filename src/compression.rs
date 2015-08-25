//! Compression for strictly increasing sequences of `u64` values

/// A compressed stream of strictly increasing `u64` values.
///
/// We optimistically assume that the differences will fit in a byte, with a zero byte to indicate
/// that this is not the case, and one should consult the next byte to determine which of `u16`,
/// `u32`, and `u64` should actually be used.
pub struct Compressed {
    bytes: Vec<u8>,
    other: Vec<Others>,
    u16s: Vec<u16>,
    u32s: Vec<u32>,
    u64s: Vec<u64>,
}

impl Compressed {
    fn push(&mut self, delta: u64) {
        if 0 < delta && delta < 256 { self.bytes.push(delta as u8); }
        else {
            self.bytes.push(0);
            if delta < (1 << 16) {
                self.other.push(Others::Unsigned16);
                self.u16s.push(delta as u16);
            }
            else if delta < (1 << 32) {
                self.other.push(Others::Unsigned32);
                self.u32s.push(delta as u32);
            }
            else {
                self.other.push(Others::Unsigned64);
                self.u64s.push(delta);
            }
        }
    }
    pub fn from<I: Iterator<Item=u64>>(iterator: I) -> Compressed {
        let mut compressor = Compressor::with_capacity(iterator.size_hint().1.unwrap_or(0));
        for item in iterator {
            compressor.push(item);
        }
        compressor.done()
    }
    pub fn decompress(&self) -> Decompressor {
        Decompressor {
            current: 0,
            bytes: self.bytes.iter(),
            other: self.other.iter(),
            u16s: self.u16s.iter(),
            u32s: self.u32s.iter(),
            u64s: self.u64s.iter(),
        }
    }
}

enum Others {
    Unsigned16,
    Unsigned32,
    Unsigned64,
}

pub struct Compressor {
    current: u64,
    compressed: Compressed,
}

impl Compressor {
    pub fn with_capacity(size: usize) -> Compressor {
        Compressor {
            current: 0,
            compressed: Compressed {
                bytes: Vec::with_capacity(size),
                other: vec![],
                u16s: vec![],
                u32s: vec![],
                u64s: vec![],
            },
        }
    }
    pub fn new() -> Compressor {
        Compressor::with_capacity(0)
    }
    /// Pushes the next value in the sequence. Does not check that the sequence is ordered, because
    /// we don't want to explode if you start with zero.
    pub fn push(&mut self, next: u64) {
        self.compressed.push(next - self.current);
        self.current = next;
    }
    pub fn done(self) -> Compressed {
        self.compressed
    }
}

pub struct Decompressor<'a> {
    current: u64,
    bytes: ::std::slice::Iter<'a, u8>,
    other: ::std::slice::Iter<'a, Others>,
    u16s: ::std::slice::Iter<'a, u16>,
    u32s: ::std::slice::Iter<'a, u32>,
    u64s: ::std::slice::Iter<'a, u64>,
}

impl<'a> Iterator for Decompressor<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        self.bytes.next().map(|&byte| {
            if byte > 0 {
                self.current += byte as u64;
            }
            else {
                self.current += match *self.other.next().unwrap() {
                    Others::Unsigned16 => { *self.u16s.next().unwrap() as u64 },
                    Others::Unsigned32 => { *self.u32s.next().unwrap() as u64 },
                    Others::Unsigned64 => { *self.u64s.next().unwrap() },
                }
            }

            self.current
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.bytes.len(), Some(self.bytes.len()))
    }
}

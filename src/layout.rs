//! Locality-preserving maps between `(u32,u32)` and `u64`.
//!
//! The Hilbert space-filling curve and Z-Order are two maps between pairs of `u32` values and a
//! single `u64` value with the property that locality in both elements of the pair results in
//! locality in the larger value. Technically, the Z-Order does skip around a bit, but does so
//! infrequently.
//!
//! The Z-Order is easily understood as interleaving the bits of the `(u32, u32)` pair into a `u64`,
//! and de-interleaving the bits from the `u64` back to a `(u32, u32)` pair. The Hilbert curve is
//! more complicated.


/// Map between `(u32, u32)` and `u64`.
pub trait Tangle {
    /// Maps a `(u32, u32)` pair to a `u64`.
    fn entangle(&self, (u32, u32)) -> u64;
    /// Maps a `u64` to a `(u32, u32)` pair.
    fn detangle(&self, u64) -> (u32, u32);
}

/// Tangles u32 pairs by interleaving their bits
pub struct ZOrder {
    entangle: Vec<u16>,
    detangle: Vec<(u8, u8)>,
}

impl ZOrder {
    // creates a new ZOrder tangler
    pub fn new() -> ZOrder {
        let mut entangle = vec![0u16; 65536];
        let mut detangle = vec![(0u8, 0u8); 65536];

        for x in 0..256 {
            for y in 0..256 {
                let mut z = 0;
                for b in 0..8 {
                    z += ((x >> b) & 0x01) << (2 * b);
                    z += ((y >> b) & 0x01) << ((2 * b) + 1);
                }
                detangle[z] = (x as u8, y as u8);
                entangle[(x << 8) + y] = z as u16;
            }
        }

        ZOrder {
            entangle: entangle,
            detangle: detangle,
        }
    }
}
impl Tangle for ZOrder {
    // entangles byte at a time
    #[inline]
    fn entangle(&self, (x, y): (u32, u32)) -> u64 {
        let x = x as usize;
        let y = y as usize;
          ((self.entangle[(((x >>  0) % 256) << 8) + ((y >>  0) % 256)] as u64) << 0)
        + ((self.entangle[(((x >>  8) % 256) << 8) + ((y >>  8) % 256)] as u64) << 16)
        + ((self.entangle[(((x >> 16) % 256) << 8) + ((y >> 16) % 256)] as u64) << 32)
        + ((self.entangle[(((x >> 24) % 256) << 8) + ((y >> 24) % 256)] as u64) << 48)
    }
    // detangles byte at a time
    #[inline]
    fn detangle(&self, tangle: u64) -> (u32, u32) {
        let (x0,y0) = self.detangle[(tangle as usize >>  0) % 65536];
        let (x1,y1) = self.detangle[(tangle as usize >> 16) % 65536];
        let (x2,y2) = self.detangle[(tangle as usize >> 32) % 65536];
        let (x3,y3) = self.detangle[(tangle as usize >> 48) % 65536];
        (x0 as u32 + ((x1 as u32) << 8) + ((x2 as u32) << 16) + ((x3 as u32) << 24),
         y0 as u32 + ((y1 as u32) << 8) + ((y2 as u32) << 16) + ((y3 as u32) << 24))
    }
}

/// Tangles u32 pairs along a Hilbert space-filling curve
pub struct Hilbert {
    entangle: Vec<u16>,         // entangle[x_byte << 16 + y_byte] -> tangle
    detangle: Vec<(u8, u8)>,    // detangle[tangle] -> (x_byte, y_byte)
    rotation: Vec<u8>,          // info on rotation, keyed per self.entangle
}

impl Hilbert {
    pub fn new() -> Hilbert {
        let mut entangle = Vec::new();
        let mut detangle: Vec<_> = (0..65536).map(|_| (0u8, 0u8)).collect();
        let mut rotation = Vec::new();
        for x in (0u32..256) {
            for y in (0u32..256) {
                let entangled = Hilbert::bit_entangle(((x << 24), (y << 24) + (1 << 23)));
                entangle.push((entangled >> 48) as u16);
                detangle[(entangled >> 48) as usize] = (x as u8, y as u8);
                rotation.push(((entangled >> 44) & 0x0F) as u8);

                //  note to self: math is hard.
                //  rotation decode:    lsbs
                //  0100 -N--> 0100 --> 0100
                //  0100 -S--> 1000 --> 1110
                //  0100 -F--> 1011 --> 1100
                //  0100 -FS-> 0111 --> 0110
            }
        }

        return Hilbert {entangle: entangle, detangle: detangle, rotation: rotation};
    }

    // entangle operator implemented bitwise
    fn bit_entangle(mut pair: (u32, u32)) -> u64 {
        let mut result = 0u64;
        for log_s_rev in (0 .. 32) {
            let log_s = 31 - log_s_rev;
            let rx = (pair.0 >> log_s) & 1u32;
            let ry = (pair.1 >> log_s) & 1u32;
            result += (((3 * rx) ^ ry) as u64) << (2 * log_s);
            pair = Hilbert::bit_rotate(log_s, pair, rx, ry);
        }

        return result;
    }

    // detangle operator implemented bitwise
    fn bit_detangle(tangle: u64) -> (u32, u32) {
        let mut result = (0u32, 0u32);
        for log_s in (0 .. 32) {
            let shifted = ((tangle >> (2 * log_s)) & 3u64) as u32;

            let rx = (shifted >> 1) & 1u32;
            let ry = (shifted ^ rx) & 1u32;
            result = Hilbert::bit_rotate(log_s, result, rx, ry);
            result = (result.0 + (rx << log_s), result.1 + (ry << log_s));
        }

        return result;
    }

    // rotation of pair based on residual bits rx and ry
    fn bit_rotate(logn: usize, pair: (u32, u32), rx: u32, ry: u32) -> (u32, u32) {
        if ry == 0 {
            if rx != 0 {
                let ::std::num::Wrapping(off) = (::std::num::Wrapping(1u32) << logn) - ::std::num::Wrapping(1u32);
                (off - pair.1, off - pair.0)
            }
            else { (pair.1, pair.0) }
        }
        else { pair }
    }
}

impl Tangle for Hilbert {
    // entangles byte at a time
    #[inline]
    fn entangle(&self, (mut x, mut y): (u32, u32)) -> u64 {
        let init_x = x;
        let init_y = y;
        let mut result = 0u64;
        for i in 0..4 {
            let x_byte = (x >> (24 - (8 * i))) as u8;
            let y_byte = (y >> (24 - (8 * i))) as u8;
            result = (result << 16) + self.entangle[(((x_byte as u16) << 8) + y_byte as u16) as usize] as u64;
            let rotation = self.rotation[(((x_byte as u16) << 8) + y_byte as u16) as usize];
            if (rotation & 0x2) > 0 { let temp = x; x = y; y = temp; }
            if rotation == 12 || rotation == 6 { x = 0xFFFFFFFF - x; y = 0xFFFFFFFF - y }
        }

        debug_assert!(Hilbert::bit_entangle((init_x, init_y)) == result);
        return result;
    }

    // detangles byte at a time
    #[inline]
    fn detangle(&self, tangle: u64) -> (u32, u32) {
        let init_tangle = tangle;
        let mut result = (0u32, 0u32);
        for log_s in 0..4 {
            let shifted = (tangle >> (16 * log_s)) as u16;
            let (x_byte, y_byte) = self.detangle[shifted as usize];
            let rotation = self.rotation[(((x_byte as u16) << 8) + y_byte as u16) as usize];
            if rotation == 12 || rotation == 6 {
                result.0 = (1 << 8 * log_s) - result.0 - 1;
                result.1 = (1 << 8 * log_s) - result.1 - 1;
            }
            if (rotation & 0x2) > 0 {
                let temp = result.0; result.0 = result.1; result.1 = temp;
            }

            result.0 += (x_byte as u32) << (8 * log_s);
            result.1 += (y_byte as u32) << (8 * log_s);
        }

        debug_assert!(Hilbert::bit_detangle(init_tangle) == result);
        return result;
    }
}

pub struct BytewiseCached {
    hilbert:    Hilbert,
    prev_hi:    u64,
    prev_out:   (u32, u32),
    prev_rot:   (bool, bool),
}

impl BytewiseCached {
    #[inline(always)]
    pub fn detangle(&mut self, tangle: u64) -> (u32, u32) {
        let (mut x_byte, mut y_byte) = unsafe { *self.hilbert.detangle.get_unchecked(tangle as u16 as usize) };

        // validate self.prev_rot, self.prev_out
        if self.prev_hi != (tangle >> 16) {
            self.prev_hi = tangle >> 16;

            // detangle with a bit set to see what happens to it
            let low = 255; //self.hilbert.entangle((0xF, 0)) as u16;
            let (x, y) = self.hilbert.detangle((self.prev_hi << 16) + low as u64);

            let value = (x as u8, y as u8);
            self.prev_rot = match value {
                (0x0F, 0x00) => (false, false), // nothing
                (0x00, 0x0F) => (true, false),  // swapped
                (0xF0, 0xFF) => (false, true),  // flipped
                (0xFF, 0xF0) => (true, true),   // flipped & swapped
                val => panic!(format!("Found : ({:x}, {:x})", val.0, val.1)),
            };
            self.prev_out = (x & 0xFFFFFF00, y & 0xFFFFFF00);
        }


        if self.prev_rot.1 {
            x_byte = 255 - x_byte;
            y_byte = 255 - y_byte;
        }
        if self.prev_rot.0 {
            let temp = x_byte; x_byte = y_byte; y_byte = temp;
        }

        return (self.prev_out.0 + x_byte as u32, self.prev_out.1 + y_byte as u32);
    }
    pub fn new() -> BytewiseCached {
        let mut result = BytewiseCached {
            hilbert: Hilbert::new(),
            prev_hi: 0xFFFFFFFFFFFFFFFF,
            prev_out: (0,0),
            prev_rot: (false, false),
        };

        result.detangle(0); // ensures that we set the cached stuff correctly
        return result;
    }
}

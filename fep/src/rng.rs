//! Randomness.
//!
//! DDD draws every random number of its effects from one game-wide generator: `0x613020` jumps
//! to `0x6196a0`, which returns the next word of a 624-word table refilled by `0x619730`
//! (INFERRED a Mersenne Twister: 624 = 0x270 words). The whole game shares it, so its sequence
//! cannot be reproduced here: NOT MATCHED. Commands draw through `Rng` instead, and turn a draw
//! into a number exactly as DDD does: `u = (r >> 1) * 2^-31` (`shr eax, 1; cvtdq2ps; mulss
//! [0x7ecf78]`), a value in `[-r, r)` as `(u + u) * r - r`.

/// A source of 32-bit random words.
pub trait Rng {
    fn next_u32(&mut self) -> u32;
}

/// The default: xorshift64*. Not DDD's generator (see above).
#[derive(Debug, Clone)]
pub struct XorShift(pub u64);

impl Default for XorShift {
    fn default() -> Self {
        XorShift(0x9e37_79b9_7f4a_7c15)
    }
}

impl Rng for XorShift {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        (x.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 32) as u32
    }
}

/// `2^-31` (`[0x7ecf78]`).
const INV_2_31: f32 = 1.0 / 2_147_483_648.0;

/// A draw in `[0, 1)`, as DDD makes it.
pub(crate) fn unit(rng: &mut dyn Rng) -> f32 {
    ((rng.next_u32() >> 1) as i32 as f32) * INV_2_31
}

/// A draw in `[-r, r)` (`(u + u) * r - r`, e.g. the generators' cube `0x580485..0x5804a5`).
pub(crate) fn signed(rng: &mut dyn Rng, r: f32) -> f32 {
    let u = unit(rng);
    (u + u) * r - r
}

/// A draw by the Rnd* mode byte (`0x5a9fd0..0x5aa0bb`): 1 -> `[0, r)`, 2 -> `[-r, 0)`, other ->
/// `[-r, r)`. The half-range arithmetic is DDD's.
pub(crate) fn by_mode(u: f32, r: f32, mode: u8) -> f32 {
    match mode {
        1 => {
            let h = r * 0.5;
            h + ((u + u) * h - h)
        }
        2 => {
            let h = r * 0.5;
            (u + u) * h - h - h
        }
        _ => (u + u) * r - r,
    }
}

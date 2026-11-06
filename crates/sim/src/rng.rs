//! SplitMix64 (Steele, Lea and Flood, "Fast Splittable Pseudorandom Number
//! Generators", OOPSLA 2014), as in Vigna's public-domain reference
//! `splitmix64.c`. Small, fast and fully determined by its seed, which is all
//! simulation needs; it is not for cryptographic use.

/// A seeded pseudorandom number generator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// A generator whose outputs are determined by `seed`.
    pub const fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A number in `0..bound`; `bound` must not be 0. The tiny modulo bias is
    /// irrelevant for fault injection.
    pub fn below(&mut self, bound: u64) -> u64 {
        assert!(bound > 0, "Rng::below(0)");
        self.next_u64() % bound
    }

    /// True with probability one half.
    pub fn coin(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
}

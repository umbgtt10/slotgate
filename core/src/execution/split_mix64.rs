// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The numbers a shuffle draws: SplitMix64, an endless stream from one seed.
//
// SplitMix64 rather than a dependency. It is a few lines, its constants are
// published, and the alternative is taking `rand` into a tool whose whole job
// is spawning processes. Nothing here is cryptographic; the requirement is that
// a seed picks one stream and picks the same one twice.
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;
    const MIX_A: u64 = 0xbf58_476d_1ce4_e5b9;
    const MIX_B: u64 = 0x94d0_49bb_1331_11eb;

    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn mix(state: u64) -> u64 {
        let mut mixed = state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(Self::MIX_A);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(Self::MIX_B);
        mixed ^ (mixed >> 31)
    }
}

impl Iterator for SplitMix64 {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.state = self.state.wrapping_add(Self::GAMMA);
        Some(Self::mix(self.state))
    }
}

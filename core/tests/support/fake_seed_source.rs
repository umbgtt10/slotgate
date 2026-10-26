// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::traits::seed_source::SeedSource;

// The seam a shuffle's seed is drawn through, faked so the order's tests can
// name the seed a run draws.
pub struct FakeSeedSource {
    seed: u64,
}

impl FakeSeedSource {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }
}

impl SeedSource for FakeSeedSource {
    fn draw(&self) -> u64 {
        self.seed
    }
}

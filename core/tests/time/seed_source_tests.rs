// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::time::seed_source::SeedSource;

// The seam a shuffle's seed is drawn through. Lives here, beside the trait it
// implements, so the order's tests can name the seed a run draws.
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

#[test]
fn draw_of_a_fake_returns_its_seed_every_time() {
    // Arrange
    let seeds = FakeSeedSource::new(4242);

    // Act
    let drawn = [seeds.draw(), seeds.draw()];

    // Assert
    assert_eq!(drawn, [4242, 4242]);
}

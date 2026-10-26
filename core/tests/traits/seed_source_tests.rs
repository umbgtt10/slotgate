// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::fake_seed_source::FakeSeedSource;
use slotgate::traits::seed_source::SeedSource;

#[test]
fn draw_of_a_fake_returns_its_seed_every_time() {
    // Arrange
    let seeds = FakeSeedSource::new(4242);

    // Act
    let drawn = [seeds.draw(), seeds.draw()];

    // Assert
    assert_eq!(drawn, [4242, 4242]);
}

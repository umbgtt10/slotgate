// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::traits::seed_source::SeedSource;
use slotgate_cli::system::wall_clock_seed::WallClockSeed;

// Zero only for a clock set before the epoch, which is the one way the draw
// can fail.
#[test]
fn draw_reads_a_seed_off_a_clock_past_the_epoch() {
    // Arrange & Act
    let seed = WallClockSeed.draw();

    // Assert
    assert_ne!(seed, 0);
}

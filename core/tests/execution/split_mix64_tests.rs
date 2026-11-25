// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The generator behind `--random`, held to outputs published apart from this
// code: the reference implementation's for seed 0, and Rosetta Code's
// "Pseudo-random numbers/Splitmix64" task for seed 1234567. With the generator
// pinned here, the order's own tests pin only the walk.

use slotgate::execution::split_mix64::SplitMix64;

// The stream never ends: a shuffle draws once per job, however many there are.
#[test]
fn next_of_a_generator_drawn_many_times_still_returns_an_output() {
    // Arrange
    let mut generator = SplitMix64::new(42);

    // Act
    let drawn = (0..10_000).filter_map(|_| generator.next()).count();

    // Assert
    assert_eq!(drawn, 10_000);
}

#[test]
fn next_of_a_generator_seeded_1234567_returns_the_published_sequence() {
    // Arrange
    let mut generator = SplitMix64::new(1_234_567);

    // Act
    let outputs: Vec<Option<u64>> = (0..5).map(|_| generator.next()).collect();

    // Assert
    assert_eq!(
        outputs,
        [
            Some(6_457_827_717_110_365_317),
            Some(3_203_168_211_198_807_973),
            Some(9_817_491_932_198_370_423),
            Some(4_593_380_528_125_082_431),
            Some(16_408_922_859_458_223_821),
        ]
    );
}

#[test]
fn next_of_a_generator_seeded_zero_returns_the_reference_first_output() {
    // Arrange
    let mut generator = SplitMix64::new(0);

    // Act
    let first = generator.next();

    // Assert
    assert_eq!(first, Some(0xe220_a839_7b1d_cdaf));
}

#[test]
fn next_of_a_generator_seeded_zero_returns_the_reference_sequence() {
    // Arrange
    let mut generator = SplitMix64::new(0);

    // Act
    let outputs: Vec<Option<u64>> = (0..5).map(|_| generator.next()).collect();

    // Assert
    assert_eq!(
        outputs,
        [
            Some(0xe220_a839_7b1d_cdaf),
            Some(0x6e78_9e6a_a1b9_65f4),
            Some(0x06c4_5d18_8009_454f),
            Some(0xf88b_b8a8_724c_81ec),
            Some(0x1b39_896a_51a8_749b),
        ]
    );
}

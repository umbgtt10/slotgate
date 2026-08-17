// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use slotgate::port_range::PortRange;

#[test]
fn end_is_the_last_port_in_the_range_not_the_one_past_it() {
    // Arrange -- end() is compared against the next slot's base to prove the
    // ranges are disjoint. An exclusive end would make adjacent ranges look
    // like they overlap by one port and fail a check that should pass.
    let range = PortRange {
        base: 30000,
        count: 100,
    };

    // Act & Assert
    assert_eq!(range.end(), 30099);
}

#[test]
fn end_of_a_single_port_range_equals_its_base() {
    // Arrange -- count 1 is the smallest useful range. Off-by-one here would
    // report a range one port wider than the slot actually owns.
    let range = PortRange {
        base: 30000,
        count: 1,
    };

    // Act & Assert
    assert_eq!(range.end(), 30000);
}

#[test]
fn end_of_an_empty_range_equals_its_base() {
    // Arrange -- count 0 has no last port to report. The saturating subtraction
    // floors at zero rather than wrapping to 65535, which would otherwise claim
    // the entire port space for a range that owns nothing.
    let range = PortRange {
        base: 30000,
        count: 0,
    };

    // Act & Assert
    assert_eq!(range.end(), 30000);
}

#[test]
fn end_saturates_at_the_top_of_the_port_space_instead_of_wrapping() {
    // Arrange -- a high base with a wide count runs past u16. Wrapping would
    // produce an end below the base, making the range look inverted and every
    // disjointness comparison against it meaningless.
    let range = PortRange {
        base: 65_500,
        count: 100,
    };

    // Act
    let end = range.end();

    // Assert
    assert_eq!(end, u16::MAX);
    assert!(end >= range.base);
}

#[test]
fn end_never_falls_below_base_across_the_whole_port_space() {
    // Arrange -- the invariant every caller relies on, checked rather than
    // assumed: a range's end is never before its start, whatever the inputs.
    let cases = [
        (0u16, 0u16),
        (0, 1),
        (0, u16::MAX),
        (1, u16::MAX),
        (30_000, 100),
        (u16::MAX, 0),
        (u16::MAX, 1),
        (u16::MAX, u16::MAX),
    ];

    // Act & Assert
    for (base, count) in cases {
        let range = PortRange { base, count };
        assert!(
            range.end() >= range.base,
            "end {} fell below base {} for count {}",
            range.end(),
            base,
            count
        );
    }
}

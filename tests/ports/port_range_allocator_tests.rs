// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::ports::port_range_allocator::PortRangeAllocator;

#[test]
fn consecutive_slot_ranges_never_overlap() {
    // Arrange
    let allocator = PortRangeAllocator::new(30000, 100);

    // Act
    let slot_zero = allocator.range_for_slot(0);
    let slot_one = allocator.range_for_slot(1);

    // Assert
    assert!(slot_zero.end() < slot_one.base);
}

#[test]
fn many_slots_across_a_full_max_parallel_span_are_pairwise_disjoint() {
    // Arrange
    let allocator = PortRangeAllocator::new(30000, 50);
    let max_parallel = 10;

    // Act
    let ranges: Vec<_> = (0..max_parallel)
        .map(|slot| allocator.range_for_slot(slot))
        .collect();

    // Assert
    for i in 0..ranges.len() {
        for j in (i + 1)..ranges.len() {
            let a = &ranges[i];
            let b = &ranges[j];
            let disjoint = a.end() < b.base || b.end() < a.base;
            assert!(disjoint, "ranges for slot {i} and slot {j} overlap");
        }
    }
}

#[test]
fn range_end_is_base_plus_count_minus_one() {
    // Arrange
    let allocator = PortRangeAllocator::new(30000, 100);

    // Act
    let range = allocator.range_for_slot(0);

    // Assert
    assert_eq!(range.end(), 30099);
}

#[test]
fn range_for_slot_advances_by_range_size_per_slot() {
    // Arrange
    let allocator = PortRangeAllocator::new(30000, 100);

    // Act
    let slot_zero = allocator.range_for_slot(0);
    let slot_one = allocator.range_for_slot(1);
    let slot_two = allocator.range_for_slot(2);

    // Assert
    assert_eq!(slot_zero.base, 30000);
    assert_eq!(slot_one.base, 30100);
    assert_eq!(slot_two.base, 30200);
}

#[test]
fn range_for_slot_count_matches_configured_range_size() {
    // Arrange
    let allocator = PortRangeAllocator::new(30000, 100);

    // Act
    let range = allocator.range_for_slot(0);

    // Assert
    assert_eq!(range.count, 100);
}

#[test]
fn range_for_slot_does_not_overflow_on_large_slot_index() {
    // Arrange
    let allocator = PortRangeAllocator::new(60000, 100);

    // Act & Assert
    let range = allocator.range_for_slot(1000);
    assert_eq!(range.base, u16::MAX);
}

#[test]
fn range_for_slot_zero_starts_at_base_port() {
    // Arrange
    let allocator = PortRangeAllocator::new(30000, 100);

    // Act
    let range = allocator.range_for_slot(0);

    // Assert
    assert_eq!(range.base, 30000);
}

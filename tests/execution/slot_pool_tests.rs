// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use slotgate::execution::slot_pool::SlotPool;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn acquire_returns_a_slot_index_within_bounds() {
    // Arrange
    let pool = SlotPool::new(4);

    // Act
    let guard = pool.acquire().await;

    // Assert
    assert!(guard.slot_index() < 4);
}

#[tokio::test]
async fn acquiring_beyond_max_parallel_blocks_until_a_release() {
    // Arrange
    let pool = SlotPool::new(1);
    let first_guard = pool.acquire().await;

    // Act
    let second_attempt = timeout(Duration::from_millis(200), pool.acquire()).await;

    // Assert
    assert!(
        second_attempt.is_err(),
        "second acquire should still be blocked while max_parallel=1 slot is held"
    );

    drop(first_guard);
    let second_guard = timeout(Duration::from_millis(500), pool.acquire())
        .await
        .expect("acquire should succeed once the held slot is released");
    assert_eq!(second_guard.slot_index(), 0);
}

#[tokio::test]
async fn at_most_max_parallel_guards_are_held_simultaneously() {
    // Arrange
    let max_parallel = 3;
    let pool = Arc::new(SlotPool::new(max_parallel));
    let current = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    // Act
    for _ in 0..(max_parallel * 4) {
        let pool = Arc::clone(&pool);
        let current = Arc::clone(&current);
        let peak = Arc::clone(&peak);
        handles.push(tokio::spawn(async move {
            let _guard = pool.acquire().await;
            let now = current.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(now, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(20)).await;
            current.fetch_sub(1, Ordering::SeqCst);
        }));
    }
    for handle in handles {
        handle.await.expect("task panicked");
    }

    // Assert
    assert!(peak.load(Ordering::SeqCst) <= max_parallel);
}

#[tokio::test]
async fn slot_is_returned_to_the_pool_when_guard_drops() {
    // Arrange
    let pool = SlotPool::new(1);
    let first_slot = {
        let guard = pool.acquire().await;
        guard.slot_index()
    };

    // Act
    let second_guard = timeout(Duration::from_millis(500), pool.acquire())
        .await
        .expect("acquiring after release should not block");

    // Assert
    assert_eq!(second_guard.slot_index(), first_slot);
}

#[tokio::test]
async fn two_concurrent_acquires_get_different_slots() {
    // Arrange
    let pool = SlotPool::new(4);

    // Act
    let first = pool.acquire().await;
    let second = pool.acquire().await;

    // Assert
    assert_ne!(first.slot_index(), second.slot_index());
}

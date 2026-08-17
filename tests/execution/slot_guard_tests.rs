// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use slotgate::slot_pool::SlotPool;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn slot_index_reports_the_same_slot_on_every_call() {
    // Arrange -- the index is read once to build the port range and again to
    // label the log file. A guard that reported a different slot the second
    // time would write one job's output under another job's ports.
    let pool = SlotPool::new(4);
    let guard = pool.acquire().await;

    // Act
    let first_read = guard.slot_index();
    let second_read = guard.slot_index();

    // Assert
    assert_eq!(first_read, second_read);
}

#[tokio::test]
async fn drop_returns_the_guards_own_slot_and_not_merely_some_slot() {
    // Arrange -- three slots taken, so 0, 1 and 2 are all out. Releasing the
    // middle one is what distinguishes returning the right index from
    // returning any index: the pool is empty, so the next acquire can only be
    // served by the slot that was just handed back.
    let pool = SlotPool::new(3);
    let first = pool.acquire().await;
    let middle = pool.acquire().await;
    let last = pool.acquire().await;
    let middle_slot = middle.slot_index();

    // Act
    drop(middle);
    let replacement = timeout(Duration::from_millis(500), pool.acquire())
        .await
        .expect("the released slot should be immediately available");

    // Assert
    assert_eq!(replacement.slot_index(), middle_slot);
    assert_ne!(replacement.slot_index(), first.slot_index());
    assert_ne!(replacement.slot_index(), last.slot_index());
}

#[tokio::test]
async fn guards_dropped_out_of_order_each_return_their_own_slot() {
    // Arrange -- jobs finish in whatever order they finish, never the order
    // they started. If release were positional rather than by index, the pool
    // would start handing out slots that are still held.
    let pool = SlotPool::new(3);
    let first = pool.acquire().await;
    let middle = pool.acquire().await;
    let last = pool.acquire().await;
    let (first_slot, middle_slot, last_slot) =
        (first.slot_index(), middle.slot_index(), last.slot_index());

    // Act -- released last, first, middle
    drop(last);
    drop(first);
    drop(middle);
    let a = pool.acquire().await;
    let b = pool.acquire().await;
    let c = pool.acquire().await;

    // Assert -- every original slot is back, none duplicated
    let mut returned = [a.slot_index(), b.slot_index(), c.slot_index()];
    returned.sort_unstable();
    let mut expected = [first_slot, middle_slot, last_slot];
    expected.sort_unstable();
    assert_eq!(returned, expected);
}

#[tokio::test]
async fn a_held_guard_keeps_its_slot_out_of_circulation() {
    // Arrange -- the permit lives in the guard, so the slot must stay taken
    // for the guard's whole lifetime. Releasing early would let two jobs bind
    // the same port range, which is the collision slotgate exists to prevent.
    let pool = SlotPool::new(1);
    let guard = pool.acquire().await;

    // Act
    let while_held = timeout(Duration::from_millis(200), pool.acquire()).await;

    // Assert
    assert!(
        while_held.is_err(),
        "the only slot is held, so a second acquire must block"
    );
    drop(guard);
    assert!(
        timeout(Duration::from_millis(500), pool.acquire())
            .await
            .is_ok(),
        "the slot must become available once the guard drops"
    );
}

// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// The real clock is Tokio's, so these run on Tokio's paused clock: time moves
// only when every task is idle, by exactly as much as a deadline asks.

use slotgate::time::job_clock::JobClock;
use slotgate::time::tokio_job_clock::TokioJobClock;
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn deadline_resolves_once_its_time_has_passed_on_the_clock() {
    // Arrange
    let clock = TokioJobClock::new();
    let started = clock.now();

    // Act
    clock.deadline(Duration::from_secs(5)).await;

    // Assert
    assert_eq!(clock.now() - started, Duration::from_secs(5));
}

#[tokio::test(start_paused = true)]
async fn now_of_a_new_clock_starts_at_zero() {
    // Arrange
    let clock = TokioJobClock::new();

    // Act
    let first = clock.now();

    // Assert
    assert_eq!(first, Duration::ZERO);
}

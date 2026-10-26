// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::fake_job_clock::FakeJobClock;
use slotgate::traits::job_clock::JobClock;
use std::task::Context;
use std::task::Waker;
use std::time::Duration;

#[test]
fn deadline_of_a_fake_that_expires_is_up_at_once() {
    // Arrange
    let clock = FakeJobClock::new(&[], true);
    let mut deadline = clock.deadline();

    // Act
    let polled = deadline
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()));

    // Assert
    assert!(polled.is_ready());
}

#[test]
fn deadline_of_a_fake_that_never_expires_is_still_pending() {
    // Arrange
    let clock = FakeJobClock::new(&[], false);
    let mut deadline = clock.deadline();

    // Act
    let polled = deadline
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()));

    // Assert
    assert!(polled.is_pending());
}

#[test]
fn now_of_a_fake_reads_its_readings_in_turn() {
    // Arrange
    let clock = FakeJobClock::new(&[Duration::from_secs(10), Duration::from_secs(13)], false);

    // Act
    let readings = [clock.now(), clock.now(), clock.now()];

    // Assert
    assert_eq!(
        readings,
        [
            Duration::from_secs(10),
            Duration::from_secs(13),
            Duration::ZERO
        ]
    );
}

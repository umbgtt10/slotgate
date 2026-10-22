// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::time::job_clock::JobClock;
use std::future::Future;
use std::future::pending;
use std::future::ready;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::Context;
use std::task::Waker;
use std::time::Duration;

// The seam the job runner times a job through. Lives here, beside the trait it
// implements, so the runner's tests share one fake: a clock that reads what it
// is told to, in turn, and whose deadlines are up at once or never.
pub struct FakeJobClock {
    readings: Mutex<Vec<Duration>>,
    expires: bool,
}

impl FakeJobClock {
    pub fn new(readings: &[Duration], expires: bool) -> Self {
        Self {
            readings: Mutex::new(readings.iter().rev().copied().collect()),
            expires,
        }
    }
}

impl JobClock for FakeJobClock {
    fn deadline(&self, _after: Duration) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        if self.expires {
            Box::pin(ready(()))
        } else {
            Box::pin(pending())
        }
    }

    fn now(&self) -> Duration {
        self.readings
            .lock()
            .expect("the readings lock")
            .pop()
            .unwrap_or_default()
    }
}

#[test]
fn deadline_of_a_fake_that_expires_is_up_at_once() {
    // Arrange
    let clock = FakeJobClock::new(&[], true);
    let mut deadline = clock.deadline(Duration::from_secs(60));

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
    let mut deadline = clock.deadline(Duration::from_millis(1));

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

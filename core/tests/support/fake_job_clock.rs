// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::traits::job_clock::JobClock;
use std::future::pending;
use std::future::ready;
use std::pin::Pin;
use std::sync::Mutex;
use std::time::Duration;

// The seam the job runner times a job through, faked so the runner's tests
// share one clock: it reads what it is told to, in turn, and its deadlines are
// up at once or never.
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
    fn deadline(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
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

// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::time::job_clock::JobClock;
use std::future::Future;
use std::pin::Pin;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::time::Instant;
use tokio::time::sleep;

// The real clock, and Tokio's, so a test on Tokio's paused clock drives it too.
// A job's time is up once `limit` has passed. Its scale starts at its first
// reading, taken inside the port rather than when it is built: building one
// reads no time.
pub struct TokioJobClock {
    limit: Duration,
    origin: OnceLock<Instant>,
}

impl TokioJobClock {
    pub fn new(limit: Duration) -> Self {
        Self {
            limit,
            origin: OnceLock::new(),
        }
    }
}

impl JobClock for TokioJobClock {
    fn deadline(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(sleep(self.limit))
    }

    fn now(&self) -> Duration {
        self.origin.get_or_init(Instant::now).elapsed()
    }
}

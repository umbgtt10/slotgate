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
// Its scale starts at its first reading, taken inside the port rather than when
// it is built: building one reads no time.
pub struct TokioJobClock {
    origin: OnceLock<Instant>,
}

impl TokioJobClock {
    pub fn new() -> Self {
        Self {
            origin: OnceLock::new(),
        }
    }
}

impl Default for TokioJobClock {
    fn default() -> Self {
        Self::new()
    }
}

impl JobClock for TokioJobClock {
    fn deadline(&self, after: Duration) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(sleep(after))
    }

    fn now(&self) -> Duration {
        self.origin.get_or_init(Instant::now).elapsed()
    }
}

// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

// What a job is timed by: a reading on the clock's own monotonic scale, whose
// differences are durations, and a deadline that resolves once `after` has
// passed on it. A port, so a test can say how long a job took and when its time
// is up, rather than wait for either.
pub trait JobClock: Send + Sync {
    fn deadline(&self, after: Duration) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    fn now(&self) -> Duration;
}

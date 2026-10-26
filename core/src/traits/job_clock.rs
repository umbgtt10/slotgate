// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::time::Duration;

// What a job is timed by: a reading on the clock's own monotonic scale, whose
// differences are durations, and a deadline that resolves once a job's time is
// up. The limit is the clock's, set where the real one is built, so a runner
// handed a clock needs no timeout of its own. A port, so a test can say how
// long a job took and when its time is up, rather than wait for either.
pub trait JobClock: Send + Sync {
    fn deadline(&self) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    fn now(&self) -> Duration;
}

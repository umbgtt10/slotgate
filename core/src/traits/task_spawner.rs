// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::job_outcome::JobOutcome;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

// Runs a job's task beside the others and hands back a handle to its outcome.
// A port, so a test can run the tasks in an order of its own choosing.
pub trait TaskSpawner: Send + Sync {
    fn spawn(
        &self,
        task: Pin<Box<dyn Future<Output = JobOutcome> + Send>>,
    ) -> Pin<Box<dyn Future<Output = JobOutcome> + Send>>;
}

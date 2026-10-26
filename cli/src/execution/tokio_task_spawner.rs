// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::execution::job_outcome::JobOutcome;
use slotgate::traits::task_spawner::TaskSpawner;
use std::pin::Pin;
use tokio::spawn;

// A job's task on Tokio's runtime, beside the others. A task that panicked
// takes the run down with it, as it always has.
pub struct TokioTaskSpawner;

impl TaskSpawner for TokioTaskSpawner {
    fn spawn(
        &self,
        task: Pin<Box<dyn Future<Output = JobOutcome> + Send>>,
    ) -> Pin<Box<dyn Future<Output = JobOutcome> + Send>> {
        let handle = spawn(task);
        Box::pin(async move { handle.await.expect("job task panicked") })
    }
}

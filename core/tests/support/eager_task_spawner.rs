// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::blocking::Blocking;
use slotgate::execution::job_outcome::JobOutcome;
use slotgate::traits::task_spawner::TaskSpawner;
use std::future::ready;
use std::pin::Pin;

// Runs each task to its end the moment it is handed one -- every fake here
// answers at once -- so a task's slot is free again before the next is asked
// for.
pub struct EagerTaskSpawner;

impl TaskSpawner for EagerTaskSpawner {
    fn spawn(
        &self,
        task: Pin<Box<dyn Future<Output = JobOutcome> + Send>>,
    ) -> Pin<Box<dyn Future<Output = JobOutcome> + Send>> {
        Box::pin(ready(Blocking::on(task)))
    }
}

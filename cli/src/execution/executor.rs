// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::slot_pool::SlotPool;
use crate::execution::tokio_task_spawner::TokioTaskSpawner;
use slotgate::execution::job::Job;
use slotgate::execution::job_outcome::JobOutcome;
use slotgate::execution::job_runner::JobRunner;
use slotgate::execution::job_scheduler::JobScheduler;
use slotgate::ports::port_range_allocator::PortRangeAllocator;
use slotgate::traits::job_executor::JobExecutor;
use std::pin::Pin;
use std::sync::Arc;

// The core's scheduler on Tokio: its slots a semaphore of `max_parallel`
// permits, its tasks Tokio's.
pub struct Executor {
    scheduler: JobScheduler,
}

impl Executor {
    pub fn new(
        max_parallel: usize,
        port_allocator: PortRangeAllocator,
        job_runner: JobRunner,
    ) -> Self {
        Self {
            scheduler: JobScheduler::new(
                Arc::new(SlotPool::new(max_parallel)),
                Arc::new(TokioTaskSpawner),
                port_allocator,
                job_runner,
            ),
        }
    }

    pub async fn run_all<F>(&self, jobs: Vec<Job>, on_outcome: F) -> Vec<JobOutcome>
    where
        F: Fn(&JobOutcome) + Send + Sync + 'static,
    {
        self.scheduler.run_all(jobs, Arc::new(on_outcome)).await
    }
}

impl JobExecutor for Executor {
    fn execute(
        &self,
        jobs: Vec<Job>,
        on_outcome: Arc<dyn Fn(&JobOutcome) + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Vec<JobOutcome>> + Send + '_>> {
        Box::pin(self.scheduler.run_all(jobs, on_outcome))
    }
}

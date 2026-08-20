// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::execution::job::Job;
use crate::execution::job_outcome::JobOutcome;
use crate::execution::job_runner::JobRunner;
use crate::execution::slot_pool::SlotPool;
use crate::ports::port_range_allocator::PortRangeAllocator;
use std::sync::Arc;
use tokio::spawn;

pub struct Executor {
    slot_pool: Arc<SlotPool>,
    port_allocator: Arc<PortRangeAllocator>,
    job_runner: Arc<JobRunner>,
}

impl Executor {
    pub fn new(
        max_parallel: usize,
        port_allocator: PortRangeAllocator,
        job_runner: JobRunner,
    ) -> Self {
        Self {
            slot_pool: Arc::new(SlotPool::new(max_parallel)),
            port_allocator: Arc::new(port_allocator),
            job_runner: Arc::new(job_runner),
        }
    }

    pub async fn run_all<F>(&self, jobs: Vec<Job>, on_outcome: F) -> Vec<JobOutcome>
    where
        F: Fn(&JobOutcome) + Send + Sync + 'static,
    {
        let on_outcome = Arc::new(on_outcome);
        let mut handles = Vec::new();
        for job in jobs {
            let slot_pool = Arc::clone(&self.slot_pool);
            let port_allocator = Arc::clone(&self.port_allocator);
            let job_runner = Arc::clone(&self.job_runner);
            let on_outcome = Arc::clone(&on_outcome);
            handles.push(spawn(async move {
                let guard = slot_pool.acquire().await;
                let port_range = port_allocator.range_for_slot(guard.slot_index());
                let outcome = job_runner.run(&job, &port_range).await;
                on_outcome(&outcome);
                outcome
            }));
        }

        let mut outcomes = Vec::new();
        for handle in handles {
            outcomes.push(handle.await.expect("job task panicked"));
        }
        outcomes
    }
}

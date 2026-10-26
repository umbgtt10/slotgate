// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::job::Job;
use crate::execution::job_outcome::JobOutcome;
use crate::execution::job_runner::JobRunner;
use crate::ports::port_range_allocator::PortRangeAllocator;
use crate::traits::job_executor::JobExecutor;
use crate::traits::slots::Slots;
use crate::traits::task_spawner::TaskSpawner;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;

// Every job run, as many at once as there are slots, each started in the order
// it was handed over. The slots are taken here, one job at a time and in that
// order, and each job's process is started here too; only waiting for it is
// handed to a task. However the runtime orders its tasks, it cannot change which
// job starts before which: at one slot the run order is the order handed over,
// exactly, and above one only the jobs' overlap varies.
pub struct JobScheduler {
    slots: Arc<dyn Slots>,
    spawner: Arc<dyn TaskSpawner>,
    port_allocator: PortRangeAllocator,
    job_runner: Arc<JobRunner>,
}

impl JobScheduler {
    pub fn new(
        slots: Arc<dyn Slots>,
        spawner: Arc<dyn TaskSpawner>,
        port_allocator: PortRangeAllocator,
        job_runner: JobRunner,
    ) -> Self {
        Self {
            slots,
            spawner,
            port_allocator,
            job_runner: Arc::new(job_runner),
        }
    }

    // The outcomes come back in the order the jobs were handed over, whoever
    // finished first; `on_outcome` hears of each as it finishes.
    pub async fn run_all(
        &self,
        jobs: Vec<Job>,
        on_outcome: Arc<dyn Fn(&JobOutcome) + Send + Sync>,
    ) -> Vec<JobOutcome> {
        let mut handles = Vec::new();
        for job in jobs {
            let slot = self.slots.acquire_slot().await;
            let port_range = self.port_allocator.range_for_slot(slot.index());
            let started = self.job_runner.start(&job, &port_range);
            let job_runner = Arc::clone(&self.job_runner);
            let on_outcome = Arc::clone(&on_outcome);
            handles.push(self.spawner.spawn(Box::pin(async move {
                let _held = slot;
                let outcome = job_runner.finish(started).await;
                on_outcome(&outcome);
                outcome
            })));
        }

        let mut outcomes = Vec::new();
        for handle in handles {
            outcomes.push(handle.await);
        }
        outcomes
    }
}

impl JobExecutor for JobScheduler {
    fn execute(
        &self,
        jobs: Vec<Job>,
        on_outcome: Arc<dyn Fn(&JobOutcome) + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Vec<JobOutcome>> + Send + '_>> {
        Box::pin(self.run_all(jobs, on_outcome))
    }
}

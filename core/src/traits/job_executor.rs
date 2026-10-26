// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::job::Job;
use crate::execution::job_outcome::JobOutcome;
use core::future::Future;
use core::pin::Pin;
use std::sync::Arc;

// Runs every job, as many at once as there are slots, telling `on_outcome` of
// each as it finishes and handing back the outcomes in the order the jobs were
// given. The concurrency is the runtime's; a port, so a run can be tested with
// none.
pub trait JobExecutor: Send + Sync {
    fn execute(
        &self,
        jobs: Vec<Job>,
        on_outcome: Arc<dyn Fn(&JobOutcome) + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Vec<JobOutcome>> + Send + '_>>;
}

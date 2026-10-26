// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use slotgate::execution::job::Job;
use slotgate::execution::job_outcome::JobOutcome;
use slotgate::execution::job_status::JobStatus;
use slotgate::traits::job_executor::JobExecutor;
use std::future::ready;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

// An executor that runs nothing: every job ends with the status it is scripted
// to, in 1.5 seconds, each outcome told to the caller as it is made. It keeps
// every job it was handed, in order.
pub struct ScriptedJobExecutor {
    status: JobStatus,
    received: Mutex<Vec<Job>>,
}

impl ScriptedJobExecutor {
    pub fn new(status: JobStatus) -> Self {
        Self {
            status,
            received: Mutex::new(Vec::new()),
        }
    }

    pub fn received(&self) -> Vec<String> {
        self.received
            .lock()
            .expect("the received lock")
            .iter()
            .map(|job| format!("{} {}", job.program, job.args.join(" ")))
            .collect()
    }

    fn outcome_of(&self, job: &Job) -> JobOutcome {
        JobOutcome {
            job_name: job.name.clone(),
            status: self.status,
            duration: Duration::from_millis(1_500),
            stdout_path: format!("logs/{}/stdout.log", job.name),
            stderr_path: format!("logs/{}/stderr.log", job.name),
        }
    }
}

impl JobExecutor for ScriptedJobExecutor {
    fn execute(
        &self,
        jobs: Vec<Job>,
        on_outcome: Arc<dyn Fn(&JobOutcome) + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Vec<JobOutcome>> + Send + '_>> {
        let outcomes: Vec<JobOutcome> = jobs.iter().map(|job| self.outcome_of(job)).collect();
        outcomes.iter().for_each(|outcome| on_outcome(outcome));
        self.received
            .lock()
            .expect("the received lock")
            .extend(jobs);
        Box::pin(ready(outcomes))
    }
}

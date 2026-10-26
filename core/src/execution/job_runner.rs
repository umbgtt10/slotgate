// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::environment_variable::EnvironmentVariable;
use crate::execution::filesystem_safe_name::FilesystemSafeName;
use crate::execution::job::Job;
use crate::execution::job_outcome::JobOutcome;
use crate::execution::job_watch::JobWatch;
use crate::execution::launch_request::LaunchRequest;
use crate::execution::launched_job::LaunchedJob;
use crate::ports::port_range::PortRange;
use crate::traits::job_clock::JobClock;
use crate::traits::job_launcher::JobLauncher;
use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;

const JOB_NAME_ENV_VAR: &str = "SLOTGATE_JOB_NAME";

// One job, start to outcome: what it is handed, where its output goes, how long
// it took. The process is the launcher's and the time the clock's; how its
// ending is judged is `JobWatch`'s.
pub struct JobRunner {
    port_env_base_var: String,
    port_env_count_var: String,
    launcher: Arc<dyn JobLauncher>,
    clock: Arc<dyn JobClock>,
}

impl JobRunner {
    pub fn new(
        port_env_base_var: String,
        port_env_count_var: String,
        launcher: Arc<dyn JobLauncher>,
        clock: Arc<dyn JobClock>,
    ) -> Self {
        Self {
            port_env_base_var,
            port_env_count_var,
            launcher,
            clock,
        }
    }

    pub async fn run(&self, job: &Job, port_range: &PortRange) -> JobOutcome {
        let started = self.clock.now();
        let LaunchedJob {
            process,
            stdout_path,
            stderr_path,
        } = self.launcher.launch(self.request_for(job, port_range));
        let status = JobWatch::new(Arc::clone(&self.clock))
            .status_of(process)
            .await;

        JobOutcome {
            job_name: job.name.clone(),
            status,
            duration: self.clock.now().saturating_sub(started),
            stdout_path,
            stderr_path,
        }
    }

    // The port range under the names the job reads it by, and the job's own name
    // as it was given, before the log directory's sanitising.
    fn request_for(&self, job: &Job, port_range: &PortRange) -> LaunchRequest {
        LaunchRequest {
            directory: FilesystemSafeName::sanitize(&job.name),
            program: job.program.clone(),
            args: job.args.clone(),
            environment: vec![
                EnvironmentVariable {
                    name: self.port_env_base_var.clone(),
                    value: port_range.base().to_string(),
                },
                EnvironmentVariable {
                    name: self.port_env_count_var.clone(),
                    value: port_range.count().to_string(),
                },
                EnvironmentVariable {
                    name: String::from(JOB_NAME_ENV_VAR),
                    value: job.name.clone(),
                },
            ],
        }
    }
}

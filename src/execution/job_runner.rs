// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::filesystem_safe_name::FilesystemSafeName;
use crate::job::Job;
use crate::job_outcome::JobOutcome;
use crate::job_status::JobStatus;
use crate::port_range::PortRange;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;
use tokio::process::Command;

const JOB_LOG_DIR_ENV_VAR: &str = "SLOTGATE_JOB_LOG_DIR";
const JOB_NAME_ENV_VAR: &str = "SLOTGATE_JOB_NAME";

pub struct JobRunner {
    port_env_base_var: String,
    port_env_count_var: String,
    timeout: Duration,
    log_dir: PathBuf,
}

impl JobRunner {
    pub fn new(
        port_env_base_var: String,
        port_env_count_var: String,
        timeout: Duration,
        log_dir: PathBuf,
    ) -> Self {
        Self {
            port_env_base_var,
            port_env_count_var,
            timeout,
            log_dir,
        }
    }

    pub async fn run(&self, job: &Job, port_range: &PortRange) -> JobOutcome {
        let job_dir = self.log_dir.join(FilesystemSafeName::sanitize(&job.name));
        std::fs::create_dir_all(&job_dir).expect("failed to create job log directory");
        let stdout_path = job_dir.join("stdout.log");
        let stderr_path = job_dir.join("stderr.log");
        let stdout_file =
            std::fs::File::create(&stdout_path).expect("failed to create stdout log file");
        let stderr_file =
            std::fs::File::create(&stderr_path).expect("failed to create stderr log file");

        let mut command = Command::new(&job.program);
        command
            .args(&job.args)
            .env(&self.port_env_base_var, port_range.base.to_string())
            .env(&self.port_env_count_var, port_range.count.to_string())
            .env(JOB_LOG_DIR_ENV_VAR, &job_dir)
            .env(JOB_NAME_ENV_VAR, &job.name)
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file));

        let started = Instant::now();
        let mut child = command.spawn().expect("failed to spawn job process");

        let status = match tokio::time::timeout(self.timeout, child.wait()).await {
            Ok(Ok(exit_status)) if exit_status.success() => JobStatus::Passed,
            Ok(Ok(_)) => JobStatus::Failed,
            Ok(Err(_)) => JobStatus::Failed,
            Err(_) => {
                let _ = child.kill().await;
                JobStatus::TimedOut
            }
        };

        JobOutcome {
            job_name: job.name.clone(),
            status,
            duration: started.elapsed(),
            stdout_path,
            stderr_path,
        }
    }
}

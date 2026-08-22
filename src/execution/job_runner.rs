// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::filesystem_safe_name::FilesystemSafeName;
use crate::execution::job::Job;
use crate::execution::job_outcome::JobOutcome;
use crate::execution::job_status::JobStatus;
use crate::ports::port_range::PortRange;
use std::fs::File;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;
use tokio::process::Command;
use tokio::time::timeout;

const NO_TESTS_MARKER: &str = "running 0 tests";

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

    // A job that ran nothing did not pass.
    //
    // `cargo test --exact` given a name that matches no test runs zero of them
    // and exits 0, so a job naming a test that does not exist was reported as a
    // pass. That is the worst answer a gate can give, and it is reachable three
    // ways: a stale `--jobs` list, a `--jobs-file` whose first name carries a
    // byte order mark, and now a `--jobs-path` scan that reads a name the
    // compiled binary does not have. `etheram-ibft` hit the second and the
    // summary said 366 passed.
    //
    // The check is the libtest line rather than the exit code, because the exit
    // code is exactly what is wrong. This tool already assumes libtest wherever
    // it discovers a test binary for itself, so reading its output is no new
    // assumption -- and a program that never prints the line is unaffected.
    fn verdict_for(stdout_path: &Path) -> JobStatus {
        match read_to_string(stdout_path) {
            Ok(output) if output.contains(NO_TESTS_MARKER) => JobStatus::Failed,
            _ => JobStatus::Passed,
        }
    }

    pub async fn run(&self, job: &Job, port_range: &PortRange) -> JobOutcome {
        let job_dir = self.log_dir.join(FilesystemSafeName::sanitize(&job.name));
        create_dir_all(&job_dir).expect("failed to create job log directory");
        let stdout_path = job_dir.join("stdout.log");
        let stderr_path = job_dir.join("stderr.log");
        let stdout_file = File::create(&stdout_path).expect("failed to create stdout log file");
        let stderr_file = File::create(&stderr_path).expect("failed to create stderr log file");

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

        let status = match timeout(self.timeout, child.wait()).await {
            Ok(Ok(exit_status)) if exit_status.success() => Self::verdict_for(&stdout_path),
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

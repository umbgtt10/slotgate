// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::job_status::JobStatus;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

pub struct JobOutcome {
    job_name: String,
    status: JobStatus,
    duration: Duration,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

impl JobOutcome {
    pub fn new(
        job_name: String,
        status: JobStatus,
        duration: Duration,
        stdout_path: PathBuf,
        stderr_path: PathBuf,
    ) -> Self {
        Self {
            job_name,
            status,
            duration,
            stdout_path,
            stderr_path,
        }
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn job_name(&self) -> &str {
        &self.job_name
    }

    pub fn status(&self) -> JobStatus {
        self.status
    }

    pub fn stderr_path(&self) -> &Path {
        &self.stderr_path
    }

    pub fn stdout_path(&self) -> &Path {
        &self.stdout_path
    }
}

// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::execution::job_status::JobStatus;
use std::path::PathBuf;
use std::time::Duration;

pub struct JobOutcome {
    pub job_name: String,
    pub status: JobStatus,
    pub duration: Duration,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
}

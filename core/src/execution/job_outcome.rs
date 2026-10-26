// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::job_status::JobStatus;
use alloc::string::String;
use core::time::Duration;

pub struct JobOutcome {
    pub job_name: String,
    pub status: JobStatus,
    pub duration: Duration,
    pub stdout_path: String,
    pub stderr_path: String,
}

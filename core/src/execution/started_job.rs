// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::launched_job::LaunchedJob;
use alloc::string::String;
use core::time::Duration;

// A job whose process has been started, and when, on the clock's scale: what
// waiting for it needs.
pub struct StartedJob {
    pub job_name: String,
    pub launched: LaunchedJob,
    pub started_at: Duration,
}

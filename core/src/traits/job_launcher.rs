// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::execution::launch_request::LaunchRequest;
use crate::execution::launched_job::LaunchedJob;

// Starts one job's process, its output going to files under the run's log
// directory. A port, so what a job runner decides about a job can be tested
// without starting one.
pub trait JobLauncher: Send + Sync {
    fn launch(&self, request: LaunchRequest) -> LaunchedJob;
}
